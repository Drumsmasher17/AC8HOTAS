@echo off
setlocal
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSROOT=%%i"
if not defined VSROOT (
  echo Visual Studio C++ Build Tools were not found.
  exit /b 1
)
call "%VSROOT%\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul
if errorlevel 1 exit /b 1
where cargo >nul 2>nul
if errorlevel 1 (
  echo Cargo was not found. Open a new terminal after installing Rust.
  exit /b 1
)
cd /d "%~dp0wizard"
cargo build --release -j 4
exit /b %errorlevel%
