#define DIRECTINPUT_VERSION 0x0800
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dinput.h>
#include <objbase.h>
#include <cstdio>
#include <cassert>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <filesystem>
#pragma comment(lib,"dinput8.lib")
#pragma comment(lib,"dxguid.lib")
#pragma comment(lib,"ole32.lib")
#pragma comment(lib,"user32.lib")

struct Edges {
    unsigned previous=0; bool primed=false;
    unsigned sample(unsigned down) {
        unsigned result=primed ? down & ~previous : 0;
        previous=down; primed=true; return result;
    }
    void reset() { previous=0; primed=false; }
};
struct Binding { unsigned slot,type,index; GUID id; };
struct Reader {
    GUID id; IDirectInputDevice8W* device=nullptr;
    ULONGLONG retry=0; Edges edges;
};
static std::vector<Binding> active,staged;
static std::vector<Reader> readers;
static IDirectInput8W* input=nullptr;
static HWND window=nullptr;
static HRESULT lastResult=E_PENDING;

static bool parse(std::istream& stream,std::vector<Binding>& output) {
    std::vector<Binding> result; unsigned used=0;
    std::string line;
    while(std::getline(stream,line)) {
        std::istringstream row(line); Binding b{}; std::string guid,extra;
        if(!(row>>b.slot>>guid>>b.type>>b.index) || row>>extra) return false;
        if(b.slot<1 || b.slot>3 || (used&(1u<<b.slot))) return false;
        if(!((b.type==12 && b.index>=1 && b.index<=128) || (b.type>=8 && b.type<=11 && b.index==0))) return false;
        if(guid.size()!=38 || guid.front()!='{' || guid.back()!='}') return false;
        std::wstring wide(guid.begin(),guid.end());
        if(FAILED(CLSIDFromString(wide.c_str(),&b.id))) return false;
        used|=1u<<b.slot; result.push_back(b);
    }
    output=std::move(result); return true;
}
static void releaseDevices() {
    for(auto& r:readers) if(r.device) { r.device->Unacquire(); r.device->Release(); }
    readers.clear();
    if(input) { input->Release(); input=nullptr; }
}
static void commit() {
    releaseDevices(); active=std::move(staged);
    for(const auto& b:active) {
        bool exists=false;
        for(const auto& r:readers) if(IsEqualGUID(r.id,b.id)) exists=true;
        if(!exists) readers.push_back(Reader{b.id});
    }
}
static bool pressed(const Binding& b,const DIJOYSTATE2& state) {
    if(b.type==12) return (state.rgbButtons[b.index-1]&0x80)!=0;
    const DWORD pov=state.rgdwPOV[0];
    if(LOWORD(pov)==0xffff) return false;
    if(b.type==8) return pov>=31500 || pov<=4500;
    if(b.type==9) return pov>=13500 && pov<=22500;
    if(b.type==10) return pov>=22500 && pov<=31500;
    return pov>=4500 && pov<=13500;
}
static bool connect(Reader& r) {
    if(!window) window=CreateWindowExW(0,L"STATIC",L"AC8HOTAS input",WS_OVERLAPPED,
        0,0,0,0,nullptr,nullptr,GetModuleHandleW(nullptr),nullptr);
    if(!window) return false;
    if(!input) {
        lastResult=DirectInput8Create(GetModuleHandleW(nullptr),DIRECTINPUT_VERSION,IID_IDirectInput8W,
            reinterpret_cast<void**>(&input),nullptr);
        if(FAILED(lastResult)) return false;
    }
    lastResult=input->CreateDevice(r.id,&r.device,nullptr);
    if(SUCCEEDED(lastResult)) lastResult=r.device->SetDataFormat(&c_dfDIJoystick2);
    if(SUCCEEDED(lastResult)) lastResult=r.device->SetCooperativeLevel(window,DISCL_BACKGROUND|DISCL_NONEXCLUSIVE);
    if(SUCCEEDED(lastResult)) lastResult=r.device->Acquire();
    if(FAILED(lastResult)) {
        if(r.device) { r.device->Release(); r.device=nullptr; } return false;
    }
    return true;
}
static unsigned poll(bool requireFocus) {
    DWORD foregroundProcess=0; unsigned result=0;
    GetWindowThreadProcessId(GetForegroundWindow(),&foregroundProcess);
    if(requireFocus && foregroundProcess!=GetCurrentProcessId()) {
        for(auto& r:readers) r.edges.reset(); return 0;
    }
    for(auto& r:readers) {
        if(!r.device) {
            if(GetTickCount64()<r.retry) continue;
            r.retry=GetTickCount64()+2000;
            if(!connect(r)) continue;
        }
        DIJOYSTATE2 state{};
        lastResult=r.device->Poll();
        if(SUCCEEDED(lastResult)) lastResult=r.device->GetDeviceState(sizeof(state),&state);
        if(FAILED(lastResult)) {
            r.device->Unacquire(); r.device->Release(); r.device=nullptr;
            r.edges.reset(); r.retry=GetTickCount64()+2000; continue;
        }
        unsigned down=0;
        for(const auto& b:active) if(IsEqualGUID(b.id,r.id) && pressed(b,state)) down|=1u<<(b.slot-1);
        result|=r.edges.sample(down);
    }
    return result;
}
extern "C" __declspec(dllexport) int ac8_weapon_prepare(void*) {
    HMODULE module=nullptr; wchar_t path[32768]{};
    if(!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS|GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&ac8_weapon_prepare),&module) || !GetModuleFileNameW(module,path,32768)) return 0;
    std::ifstream stream(std::filesystem::path(path).parent_path()/L"weapon-bindings.internal.txt");
    // Called with one sentinel argument; success returns that existing argument.
    return stream && parse(stream,staged) ? 1 : 0;
}
extern "C" __declspec(dllexport) int ac8_weapon_commit(void*) { commit(); return 0; }
// Called with seven sentinels; returning the top N arguments communicates mask N.
// No private lua_State layout, Lua symbols, keyboard events or per-frame file I/O.
extern "C" __declspec(dllexport) int ac8_weapon_poll(void*) { return static_cast<int>(poll(true)); }

#ifdef WEAPON_CONSOLE
int main(int argc,char** argv) {
    Edges e;
    assert(e.sample(7)==0); assert(e.sample(0)==0); assert(e.sample(1)==1);
    assert(e.sample(1)==0); assert(e.sample(3)==2); assert(e.sample(7)==4);
    e.reset(); assert(e.sample(7)==0);
    std::vector<Binding> parsed;
    const char* guid="{11111111-2222-3333-4444-555555555555}";
    std::istringstream valid(std::string("1 ")+guid+" 12 14\n2 "+guid+" 8 0\n");
    assert(parse(valid,parsed) && parsed.size()==2);
    DIJOYSTATE2 state{}; state.rgdwPOV[0]=0xffffffff;
    state.rgbButtons[13]=0x80; assert(pressed(parsed[0],state));
    assert(!pressed(parsed[1],state)); state.rgdwPOV[0]=4500;
    assert(pressed(parsed[1],state)); state.rgdwPOV[0]=9000; assert(!pressed(parsed[1],state));
    for(const auto& text:{std::string("1 ")+guid+" 12 0",std::string("4 ")+guid+" 12 14",
        std::string("1 ")+guid+" 12 14\n1 "+guid+" 12 15",std::string("1 invalid 12 14")}) {
        std::istringstream invalid(text); assert(!parse(invalid,parsed));
    }
    puts("PASS: config parsing, bounds, duplicates, button/POV mapping and edge reset");
    if(argc>1) {
        std::ifstream file(argv[1]); assert(file && parse(file,staged)); commit(); poll(false);
        bool ok=true;
        for(const auto& r:readers) { printf("Device acquisition=%s downMask=%u HRESULT=0x%08lX\n",
            r.device?"OK":"FAILED",r.edges.previous,static_cast<unsigned long>(lastResult)); ok &= r.device!=nullptr; }
        releaseDevices(); if(window) DestroyWindow(window); return ok?0:1;
    }
    return 0;
}
#endif
