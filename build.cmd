@echo off
setlocal
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSROOT=%%i"
if not defined VSROOT exit /b 1
call "%VSROOT%\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul
if errorlevel 1 exit /b 1
cd /d "%~dp0"
if not exist build mkdir build
if not exist mod\Scripts mkdir mod\Scripts
cd build
cl /nologo /std:c++17 /O2 /MT /W4 /EHsc /LD ..\src\devices.cpp /link /OUT:ac8_hotas_devices_001.dll
if errorlevel 1 exit /b 1
cl /nologo /std:c++17 /O2 /MT /W4 /EHsc /DHOTAS_CONSOLE ..\src\devices.cpp /link /OUT:scan-devices.exe
if errorlevel 1 exit /b 1
copy /y ac8_hotas_devices_001.dll ..\mod\Scripts\ >nul
exit /b %errorlevel%
