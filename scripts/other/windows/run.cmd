@echo off
rem Starts the FerriteOS ISO in QEMU using the OVMF files from .\ovmf.
rem Usage: run.cmd [path\to\ferrite_os.iso]
rem Default: ferrite_os-release.iso, then the first ferrite_os-*.iso next to this script,
rem then ferrite_os.iso, then <repo>\build\ferrite_os.iso.
setlocal EnableExtensions EnableDelayedExpansion

set "DIR=%~dp0"
set "OVMF_DIR=%DIR%ovmf"
set "RC=0"
set "VARS="

set "ISO=%~1"
if not defined ISO (
    if exist "%DIR%ferrite_os-release.iso" set "ISO=%DIR%ferrite_os-release.iso"
)
if not defined ISO (
    rem Quoted patterns are not expanded by FOR, so glob from inside the folder.
    pushd "%DIR%"
    for %%F in (ferrite_os-*.iso) do if not defined ISO set "ISO=%%~fF"
    popd
)
if not defined ISO if exist "%DIR%ferrite_os.iso" set "ISO=%DIR%ferrite_os.iso"
if not defined ISO if exist "%DIR%..\..\..\build\ferrite_os.iso" set "ISO=%DIR%..\..\..\build\ferrite_os.iso"
if not defined ISO (
    echo [error] No ISO found next to this script. Pass the path as argument.
    set "RC=1"
    goto :done
)
if not exist "!ISO!" (
    echo [error] ISO not found: !ISO!
    set "RC=1"
    goto :done
)

set "QEMU="
for /f "delims=" %%Q in ('where qemu-system-x86_64 2^>nul') do if not defined QEMU set "QEMU=%%Q"
if not defined QEMU if exist "%ProgramFiles%\qemu\qemu-system-x86_64.exe" set "QEMU=%ProgramFiles%\qemu\qemu-system-x86_64.exe"
if not defined QEMU (
    echo [error] QEMU missing, run install-deps.cmd
    set "RC=1"
    goto :done
)

if not exist "%OVMF_DIR%\code.fd" goto :no_ovmf
if not exist "%OVMF_DIR%\vars.fd" goto :no_ovmf
goto :have_ovmf
:no_ovmf
echo [error] OVMF missing, run install-deps.cmd
set "RC=1"
goto :done
:have_ovmf

rem Work on a copy of vars.fd so the pristine template stays untouched.
set "VARS=%TEMP%\ferrite_vars_%RANDOM%.fd"
copy /y "%OVMF_DIR%\vars.fd" "!VARS!" >nul

rem QEMU splits -drive options on commas, so commas in paths must be doubled.
set "CODE_Q=!OVMF_DIR:,=,,!"
set "VARS_Q=!VARS:,=,,!"

"!QEMU!" -cdrom "!ISO!" -m 4G -vga std -serial stdio -drive "if=pflash,format=raw,readonly=on,file=!CODE_Q!\code.fd" -drive "if=pflash,format=raw,file=!VARS_Q!"
set "RC=!errorlevel!"

:done
if defined VARS if exist "!VARS!" del "!VARS!"
call :pause_if_dblclick
exit /b !RC!

rem When started by double-click, the console closes on exit; keep it open.
:pause_if_dblclick
set "CL=!cmdcmdline:"=!"
if not "!CL:/c =!"=="!CL!" pause
exit /b 0
