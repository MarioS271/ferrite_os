@echo off
rem Installs everything needed to run the FerriteOS ISO:
rem   - QEMU (system-wide, via winget)
rem   - OVMF (code.fd / vars.fd), downloaded next to this script into .\ovmf
setlocal EnableExtensions EnableDelayedExpansion

set "DIR=%~dp0"
set "OVMF_DIR=%DIR%ovmf"
set "OVMF_API=https://api.github.com/repos/rust-osdev/ovmf-prebuilt/releases/latest"
set "RC=0"
set "TMP_DIR="

call :install_qemu
if errorlevel 1 (
    set "RC=1"
    goto :done
)

call :install_ovmf
if errorlevel 1 (
    set "RC=1"
    goto :done
)

echo.
echo Done. Start the ISO with: "%DIR%run.cmd" [path\to\ferrite_os.iso]

:done
if defined TMP_DIR if exist "!TMP_DIR!" rmdir /s /q "!TMP_DIR!"
call :pause_if_dblclick
exit /b !RC!

rem --- helpers ---

:install_qemu
where qemu-system-x86_64 >nul 2>&1 && (
    echo [ok] qemu-system-x86_64 already installed
    exit /b 0
)
if exist "%ProgramFiles%\qemu\qemu-system-x86_64.exe" (
    echo [ok] qemu-system-x86_64 already installed in "%ProgramFiles%\qemu"
    exit /b 0
)
where winget >nul 2>&1 || (
    echo [error] winget not found. Install QEMU manually from https://www.qemu.org/download/#windows
    exit /b 1
)
echo ^>^> Installing QEMU system-wide
winget install -e --id SoftwareFreedomConservancy.QEMU --accept-package-agreements --accept-source-agreements
if errorlevel 1 (
    echo [error] QEMU installation failed
    exit /b 1
)
exit /b 0

:install_ovmf
if exist "%OVMF_DIR%\code.fd" if exist "%OVMF_DIR%\vars.fd" (
    echo [ok] OVMF already present in "%OVMF_DIR%"
    exit /b 0
)
for %%T in (curl tar powershell) do (
    where %%T >nul 2>&1 || (
        echo [error] '%%T' is required
        exit /b 1
    )
)

echo ^>^> Downloading OVMF (rust-osdev/ovmf-prebuilt)
set "URL="
for /f "usebackq delims=" %%U in (`powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = 'Tls12'; (Invoke-RestMethod '%OVMF_API%').assets.Where({ $_.name -like '*-bin.tar.xz' })[0].browser_download_url"`) do set "URL=%%U"
if not defined URL (
    echo [error] Could not find OVMF release asset
    exit /b 1
)

set "TMP_DIR=%TEMP%\ferrite_ovmf_%RANDOM%"
mkdir "!TMP_DIR!" || exit /b 1
curl.exe -fsSL -o "!TMP_DIR!\ovmf.tar.xz" "!URL!" || (
    echo [error] Download failed
    exit /b 1
)
tar -xf "!TMP_DIR!\ovmf.tar.xz" -C "!TMP_DIR!" || (
    echo [error] Extraction failed
    exit /b 1
)

if not exist "%OVMF_DIR%" mkdir "%OVMF_DIR%"
for /d %%D in ("!TMP_DIR!\*") do (
    if exist "%%D\x64\code.fd" (
        copy /y "%%D\x64\code.fd" "%OVMF_DIR%\" >nul
        copy /y "%%D\x64\vars.fd" "%OVMF_DIR%\" >nul
    )
)
if not exist "%OVMF_DIR%\code.fd" (
    echo [error] OVMF files not found in the downloaded archive
    exit /b 1
)
echo [ok] OVMF installed to "%OVMF_DIR%"
exit /b 0

rem When started by double-click, the console closes on exit; keep it open.
:pause_if_dblclick
set "CL=!cmdcmdline:"=!"
if not "!CL:/c =!"=="!CL!" pause
exit /b 0
