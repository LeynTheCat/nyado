@echo off
setlocal enabledelayedexpansion

echo Nyado Installer

where curl >nul 2>nul
if errorlevel 1 (
    echo Error: curl not found. Please install curl and try again.
    pause
    exit /b 1
)

set "ARCH=x86_64"
if "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "ARCH=aarch64"

set "BIN_DIR=%LOCALAPPDATA%\nyado\bin"
set "EXE_NAME=nyado.exe"
set "REPO=LeynTheCat/nyado"
set "BIN_NAME=nyado-%ARCH%-pc-windows-gnu.exe"

for /f "tokens=* usebackq" %%i in (`curl -s "https://api.github.com/repos/%REPO%/releases/latest" ^| findstr "tag_name"`) do (
    set "line=%%i"
    set "tag=!line:*"tag_name": "=!
    set "tag=!tag:",=!
)

if "%tag%"=="" (
    echo Could not detect latest tag, using 'latest'
    set "BIN_URL=https://github.com/%REPO%/releases/latest/download/%BIN_NAME%"
) else (
    set "BIN_URL=https://github.com/%REPO%/releases/download/%tag%/%BIN_NAME%"
)

echo Downloading nyado %tag% for %ARCH% from %BIN_URL%

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"

curl -L --progress-bar -o "%BIN_DIR%\%EXE_NAME%" "%BIN_URL%"
if errorlevel 1 (
    echo Failed to download binary.
    pause
    exit /b 1
)

echo Adding %BIN_DIR% to user PATH...
set "PATH_KEY=HKCU\Environment"
for /f "skip=2 tokens=3*" %%a in ('reg query "%PATH_KEY%" /v PATH 2^>nul') do set "CURRENT_PATH=%%b"
echo !CURRENT_PATH! | findstr /i "!BIN_DIR!" >nul
if errorlevel 1 (
    set "NEW_PATH=!CURRENT_PATH!;!BIN_DIR!"
    setx PATH "!NEW_PATH!" >nul
    echo Added to PATH. Changes will apply in new terminal windows.
) else (
    echo PATH already contains %BIN_DIR%
)

echo Nyado installed successfully!
echo Run 'nyado' from a new Command Prompt or PowerShell window.
pause