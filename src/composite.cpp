// AC8HOTAS composite-device support.
//
// Some controllers expose several DirectInput collections that share one
// product ID (for example a wheel base and a throttle panel behind one USB
// cable). The game's flight-stick profiles are matched by product ID, so it
// cannot tell those collections apart.
//
// This module rewrites the product ID of every game-controller collection after
// the first one inside the game process only. Instance GUIDs are left untouched,
// so opening the physical device still works. No driver, registry, vJoy or
// system-wide change is made.
#define DIRECTINPUT_VERSION 0x0800
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dinput.h>
#include <algorithm>
#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <intrin.h>
#include <filesystem>
#include <map>
#include <mutex>
#include <set>
#include <string>
#include <vector>
#pragma comment(lib,"dinput8.lib")
#pragma comment(lib,"dxguid.lib")
#pragma comment(lib,"ole32.lib")

namespace {

SRWLOCK g_logLock = SRWLOCK_INIT;
FILE* g_log = nullptr;
int g_lines = 0;
const int kMaxLines = 1000;
std::recursive_mutex g_enumerateLock;

using EnumDevicesW_t = HRESULT (WINAPI*)(IDirectInput8W*, DWORD, LPDIENUMDEVICESCALLBACKW, LPVOID, DWORD);
using CreateDeviceW_t = HRESULT (WINAPI*)(IDirectInput8W*, const GUID&, IDirectInputDevice8W**, LPUNKNOWN);
using GetDeviceInfoW_t = HRESULT (WINAPI*)(IDirectInputDevice8W*, DIDEVICEINSTANCEW*);

EnumDevicesW_t g_enumW = nullptr;
CreateDeviceW_t g_createW = nullptr;
void* g_wVtable = nullptr;
// Enumeration rewrites are only visible in the copies handed to the callback.
// The game also asks each device it opens for its product ID, so keep the same
// mapping available to GetDeviceInfo. The original pointer is stored per vtable:
// Steam's overlay wraps devices with its own vtable and a single global original
// would call one vtable's function with another device's `this`.
std::map<void*, GetDeviceInfoW_t> g_infoOriginals;
std::set<void*> g_patchedDeviceVtables;
std::map<std::string, DWORD> g_overrides;
std::set<std::string> g_loggedInfo;

std::wstring ModulePath(HMODULE mod) {
    wchar_t path[32768]{};
    DWORD n = GetModuleFileNameW(mod, path, 32768);
    return std::wstring(path, n);
}

std::string U8(const wchar_t* s) {
    if (!s) return "";
    int n = WideCharToMultiByte(CP_UTF8, 0, s, -1, nullptr, 0, nullptr, nullptr);
    if (n <= 0) return "";
    std::string r(n, '\0');
    WideCharToMultiByte(CP_UTF8, 0, s, -1, r.data(), n, nullptr, nullptr);
    if (!r.empty()) r.pop_back();
    return r;
}

std::wstring GuidStr(const GUID& g) {
    wchar_t b[40]{};
    StringFromGUID2(g, b, 40);
    return b;
}

std::string GuidKey(const GUID& g) {
    return std::string(reinterpret_cast<const char*>(&g), sizeof(g));
}

void FormatCaller(void* address, char* out, size_t size) {
    HMODULE mod = nullptr;
    if (GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            reinterpret_cast<LPCWSTR>(address), &mod) && mod) {
        std::wstring path = ModulePath(mod);
        size_t slash = path.find_last_of(L"\\/");
        std::wstring base = slash == std::wstring::npos ? path : path.substr(slash + 1);
        unsigned long long offset = reinterpret_cast<unsigned char*>(address) -
            reinterpret_cast<unsigned char*>(mod);
        snprintf(out, size, "%s+0x%llX", U8(base.c_str()).c_str(), offset);
    } else {
        snprintf(out, size, "%p", address);
    }
}

void LogRaw(const char* fmt, ...) {
    if (!g_log) return;
    AcquireSRWLockExclusive(&g_logLock);
    if (g_lines < kMaxLines) {
        SYSTEMTIME st{};
        GetLocalTime(&st);
        fprintf(g_log, "[%02d:%02d:%02d.%03d] ", st.wHour, st.wMinute, st.wSecond, st.wMilliseconds);
        va_list ap;
        va_start(ap, fmt);
        vfprintf(g_log, fmt, ap);
        va_end(ap);
        fputc('\n', g_log);
        fflush(g_log);
        ++g_lines;
        if (g_lines == kMaxLines) {
            fprintf(g_log, "[log capacity reached; further lines suppressed]\n");
            fflush(g_log);
        }
    }
    ReleaseSRWLockExclusive(&g_logLock);
}

bool PatchSlot(void** vtbl, int index, void* replacement, void** original) {
    DWORD oldProtect = 0;
    if (!VirtualProtect(&vtbl[index], sizeof(void*), PAGE_READWRITE, &oldProtect)) return false;
    if (original) *original = vtbl[index];
    vtbl[index] = replacement;
    DWORD ignored = 0;
    VirtualProtect(&vtbl[index], sizeof(void*), oldProtect, &ignored);
    return true;
}

// Keep the first collection's real product ID and give later collections of the
// same product a unique ID that preserves the vendor half. Sorting by instance
// GUID makes the choice stable across enumerations.
void ComputeOverrides(const std::vector<DIDEVICEINSTANCEW>& devices,
        std::map<std::string, DWORD>& overrides) {
    std::map<DWORD, std::vector<std::string>> groups;
    std::set<DWORD> used;
    for (const auto& d : devices) {
        groups[d.guidProduct.Data1].push_back(GuidKey(d.guidInstance));
        used.insert(d.guidProduct.Data1);
    }
    for (const auto& entry : groups) {
        DWORD product = entry.first;
        std::vector<std::string> instances = entry.second;
        if (instances.size() < 2) continue;
        std::sort(instances.begin(), instances.end());
        unsigned pid = (product >> 16) & 0xFFFF;
        for (size_t i = 1; i < instances.size(); ++i) {
            DWORD fake = 0;
            for (int tries = 0; tries < 0xFFFF; ++tries) {
                pid = (pid + 1) & 0xFFFF;
                if (pid == 0) continue;
                DWORD candidate = (static_cast<DWORD>(pid) << 16) | (product & 0xFFFF);
                if (used.count(candidate)) continue;
                fake = candidate;
                break;
            }
            if (!fake) continue;
            used.insert(fake);
            overrides[instances[i]] = fake;
            GUID instance{};
            memcpy(&instance, instances[i].data(), sizeof(instance));
            LogRaw("split product=0x%08lX instance=%s -> 0x%08lX", product,
                U8(GuidStr(instance).c_str()).c_str(), fake);
        }
    }
}

struct CollectCtxW {
    std::vector<DIDEVICEINSTANCEW>* devices;
};

BOOL CALLBACK CollectThunkW(LPCDIDEVICEINSTANCEW info, LPVOID pv) {
    auto* ctx = static_cast<CollectCtxW*>(pv);
    if (ctx && ctx->devices && info) ctx->devices->push_back(*info);
    return DIENUM_CONTINUE;
}

// Patch the device object's own vtable so later GetDeviceInfo calls report the
// same product ID as the enumeration. IDirectInputDevice8A and W share this
// early structure layout, and only guidProduct.Data1 is changed.
HRESULT WINAPI HookGetDeviceInfoW(IDirectInputDevice8W* self, DIDEVICEINSTANCEW* info) {
    GetDeviceInfoW_t original = nullptr;
    {
        std::lock_guard<std::recursive_mutex> lock(g_enumerateLock);
        auto it = g_infoOriginals.find(*reinterpret_cast<void***>(self));
        if (it != g_infoOriginals.end()) original = it->second;
    }
    if (!original) return E_FAIL;
    HRESULT hr = original(self, info);
    if (SUCCEEDED(hr) && info) {
        std::lock_guard<std::recursive_mutex> lock(g_enumerateLock);
        auto it = g_overrides.find(GuidKey(info->guidInstance));
        if (it != g_overrides.end()) {
            info->guidProduct.Data1 = it->second;
            char caller[300]{};
            FormatCaller(_ReturnAddress(), caller, sizeof(caller));
            if (g_loggedInfo.insert(std::string(caller) + GuidKey(info->guidInstance)).second)
                LogRaw("GetDeviceInfo caller=%s instance=%s -> 0x%08lX", caller,
                    U8(GuidStr(info->guidInstance).c_str()).c_str(), it->second);
        }
    }
    return hr;
}

void PatchDeviceVtable(IDirectInputDevice8W* device) {
    if (!device) return;
    void** vtbl = *reinterpret_cast<void***>(device);
    std::lock_guard<std::recursive_mutex> lock(g_enumerateLock);
    if (g_patchedDeviceVtables.count(vtbl)) return;
    void* original = nullptr;
    if (!PatchSlot(vtbl, 15, reinterpret_cast<void*>(&HookGetDeviceInfoW), &original)) return;
    g_patchedDeviceVtables.insert(vtbl);
    g_infoOriginals[vtbl] = reinterpret_cast<GetDeviceInfoW_t>(original);
    char at[300]{};
    FormatCaller(original, at, sizeof(at));
    LogRaw("device vtable %p GetDeviceInfo=%s hooked", vtbl, at);
}

HRESULT WINAPI HookCreateDeviceW(IDirectInput8W* self, const GUID& guid,
        IDirectInputDevice8W** device, LPUNKNOWN outer) {
    if (!g_createW) return E_FAIL;
    HRESULT hr = g_createW(self, guid, device, outer);
    if (SUCCEEDED(hr)) PatchDeviceVtable(device ? *device : nullptr);
    return hr;
}

HRESULT WINAPI HookEnumW(IDirectInput8W* self, DWORD type, LPDIENUMDEVICESCALLBACKW cb, LPVOID ref, DWORD flags) {
    if (!g_enumW) return E_FAIL;
    // Only game controllers are split. Rewriting every HID collection in the
    // process would touch devices this mod has no reason to change.
    if (!cb || type != DI8DEVCLASS_GAMECTRL)
        return g_enumW(self, type, cb, ref, flags);

    std::lock_guard<std::recursive_mutex> lock(g_enumerateLock);
    std::vector<DIDEVICEINSTANCEW> devices;
    CollectCtxW collect{&devices};
    HRESULT hr = g_enumW(self, type, CollectThunkW, &collect, flags);
    if (FAILED(hr)) return g_enumW(self, type, cb, ref, flags);

    std::map<std::string, DWORD> overrides;
    ComputeOverrides(devices, overrides);
    if (!devices.empty()) {
        g_overrides.clear();
        for (const auto& entry : overrides) g_overrides[entry.first] = entry.second;
    }
    if (!overrides.empty()) {
        char caller[300]{};
        FormatCaller(_ReturnAddress(), caller, sizeof(caller));
        LogRaw("EnumDevicesW caller=%s devices=%d splits=%d", caller,
            static_cast<int>(devices.size()), static_cast<int>(overrides.size()));
    }
    for (const auto& device : devices) {
        DIDEVICEINSTANCEW copy = device;
        auto it = overrides.find(GuidKey(device.guidInstance));
        if (it != overrides.end()) copy.guidProduct.Data1 = it->second;
        if (cb(&copy, ref) == DIENUM_STOP) break;
    }
    return hr;
}

bool g_installed = false;

} // namespace

extern "C" __declspec(dllexport) int ac8_hotas_composite_install(void*) {
    if (g_installed) return 1;
    g_installed = true;

    HMODULE self = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(&ac8_hotas_composite_install), &self);
    std::filesystem::path dir = std::filesystem::path(ModulePath(self)).parent_path();
    if (_wcsicmp(dir.filename().c_str(), L"Scripts") == 0) dir = dir.parent_path();
    std::wstring logPath = (dir / L"COMPOSITE.log").wstring();

    _wfopen_s(&g_log, logPath.c_str(), L"ab");
    if (!g_log) return 0;
    LogRaw("=== AC8HOTAS composite support installed ===");
    LogRaw("module=%s log=%s", U8(ModulePath(self).c_str()).c_str(), U8(logPath.c_str()).c_str());

    IDirectInput8W* di = nullptr;
    HRESULT hr = DirectInput8Create(GetModuleHandleW(nullptr), DIRECTINPUT_VERSION, IID_IDirectInput8W,
        reinterpret_cast<void**>(&di), nullptr);
    if (FAILED(hr) || !di) {
        LogRaw("DirectInput8Create failed hr=0x%08lX", static_cast<unsigned long>(hr));
        return 0;
    }
    void** vtbl = *reinterpret_cast<void***>(di);
    di->Release();
    void* original = nullptr;
    if (!PatchSlot(vtbl, 4, reinterpret_cast<void*>(&HookEnumW), &original)) {
        LogRaw("EnumDevicesW patch failed");
        return 0;
    }
    g_enumW = reinterpret_cast<EnumDevicesW_t>(original);
    void* createOriginal = nullptr;
    if (PatchSlot(vtbl, 3, reinterpret_cast<void*>(&HookCreateDeviceW), &createOriginal))
        g_createW = reinterpret_cast<CreateDeviceW_t>(createOriginal);
    else
        LogRaw("CreateDeviceW patch failed; GetDeviceInfo rewrites disabled");
    g_wVtable = vtbl;
    char where[300]{};
    FormatCaller(reinterpret_cast<void*>(g_enumW), where, sizeof(where));
    LogRaw("input vtable W=%p EnumDevices=%s", vtbl, where);
    LogRaw("=== install complete ===");
    return 1;
}

BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) DisableThreadLibraryCalls(instance);
    return TRUE;
}
