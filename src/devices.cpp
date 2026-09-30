#define DIRECTINPUT_VERSION 0x0800
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dinput.h>
#include <objbase.h>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#pragma comment(lib,"dinput8.lib")
#pragma comment(lib,"dxguid.lib")
#pragma comment(lib,"ole32.lib")

static std::string utf8(const wchar_t* s) {
    int n=WideCharToMultiByte(CP_UTF8,0,s,-1,nullptr,0,nullptr,nullptr);
    std::string r(n,'\0');
    if(n) WideCharToMultiByte(CP_UTF8,0,s,-1,r.data(),n,nullptr,nullptr);
    if(!r.empty()) r.pop_back(); return r;
}
static std::string quote(const std::string& s) {
    std::string r="\"";
    for(unsigned char c:s) {
        if(c=='"'||c=='\\') { r+='\\'; r+=c; }
        else if(c<32||c==127) { char b[5]; sprintf_s(b,"\\%03u",c); r+=b; }
        else r+=c;
    }
    return r+'"';
}
static std::string guid(const GUID& g) { wchar_t b[40]{}; StringFromGUID2(g,b,40); return utf8(b); }
struct Scan { IDirectInput8W* di; std::ostringstream out; bool ok=true; };
static BOOL CALLBACK device(const DIDEVICEINSTANCEW* info,void* context) {
    auto& s=*static_cast<Scan*>(context);
    IDirectInputDevice8W* d=nullptr;
    if(FAILED(s.di->CreateDevice(info->guidInstance,&d,nullptr))) { s.ok=false; return DIENUM_CONTINUE; }
    DIDEVCAPS caps{}; caps.dwSize=sizeof(caps);
    if(FAILED(d->GetCapabilities(&caps))||FAILED(d->SetDataFormat(&c_dfDIJoystick2))) {
        d->Release(); s.ok=false; return DIENUM_CONTINUE;
    }
    char id[16]; sprintf_s(id,"0x%08lX",info->guidProduct.Data1);
    s.out<<"{ id="<<quote(guid(info->guidInstance))<<", product="<<quote(id)
         <<", name="<<quote(utf8(info->tszProductName))<<", axes="<<caps.dwAxes
         <<", buttons="<<caps.dwButtons<<", povs="<<caps.dwPOVs<<", inputs={\n";
    const DWORD offsets[]={DIJOFS_X,DIJOFS_Y,DIJOFS_Z,DIJOFS_RX,DIJOFS_RY,DIJOFS_RZ,DIJOFS_SLIDER(0),DIJOFS_SLIDER(1)};
    const char* names[]={"X","Y","Z","Rx","Ry","Rz","Slider1","Slider2"};
    auto object=[&](DWORD offset,const std::string& token,bool supported) {
        DIDEVICEOBJECTINSTANCEW o{}; o.dwSize=sizeof(o);
        if(SUCCEEDED(d->GetObjectInfo(&o,offset,DIPH_BYOFFSET)))
            s.out<<"{token="<<quote(token)<<",name="<<quote(utf8(o.tszName))<<",supported="<<(supported?"true":"false")<<"},\n";
    };
    for(int i=0;i<8;++i) object(offsets[i],names[i],i<7);
    for(int i=0;i<128;++i) object(DIJOFS_BUTTON(i),"Button"+std::to_string(i+1),false);
    for(int i=0;i<4;++i) object(DIJOFS_POV(i),"POV"+std::to_string(i+1),false);
    s.out<<"}},\n"; d->Release(); return DIENUM_CONTINUE;
}
static bool scan(const std::filesystem::path& output) {
    IDirectInput8W* di=nullptr;
    HRESULT hr=DirectInput8Create(GetModuleHandleW(nullptr),DIRECTINPUT_VERSION,IID_IDirectInput8W,reinterpret_cast<void**>(&di),nullptr);
    Scan s{di};
    if(SUCCEEDED(hr)) { hr=di->EnumDevices(DI8DEVCLASS_GAMECTRL,device,&s,DIEDFL_ATTACHEDONLY); di->Release(); }
    bool ok=SUCCEEDED(hr)&&s.ok;
    auto temp=output; temp+=".tmp";
    { std::ofstream f(temp,std::ios::binary|std::ios::trunc);
      f<<"return {ok="<<(ok?"true":"false")<<",devices={\n"<<s.out.str()<<"}}\n";
      f.close(); if(!f) return false; }
    return MoveFileExW(temp.c_str(),output.c_str(),MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH)&&ok;
}
extern "C" __declspec(dllexport) int ac8_hotas_scan(void*) {
    try {
        HMODULE module=nullptr; wchar_t path[32768]{};
        if(GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS|GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            reinterpret_cast<LPCWSTR>(&ac8_hotas_scan),&module)&&GetModuleFileNameW(module,path,32768))
            scan(std::filesystem::path(path).parent_path()/L"devices.lua");
    } catch(...) {} // Never propagate C++ exceptions across the Lua C boundary.
    return 0;
}
#ifdef HOTAS_CONSOLE
int wmain(int argc,wchar_t** argv) {
    if(argc!=2) return 2;
    try { return scan(argv[1])?0:1; } catch(...) { return 1; }
}
#endif
