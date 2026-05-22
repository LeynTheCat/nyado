@echo off
setlocal enabledelayedexpansion

echo Nyado Uninstaller

set "BIN_DIR=%LOCALAPPDATA%\nyado\bin"

if exist "%BIN_DIR%" (
    echo Removing %BIN_DIR%
    rmdir /s /q "%BIN_DIR%"
) else (
    echo Binary directory not found, skipping.
)

echo Removing PATH entry...
set "PATH_KEY=HKCU\Environment"
for /f "skip=2 tokens=3*" %%a in ('reg query "%PATH_KEY%" /v PATH 2^>nul') do set "CURRENT_PATH=%%b"

if defined CURRENT_PATH (
    echo !CURRENT_PATH! | findstr /i "!BIN_DIR!" >nul
    if not errorlevel 1 (
        set "NEW_PATH=!CURRENT_PATH:%BIN_DIR%;=!"
        set "NEW_PATH=!NEW_PATH:%BIN_DIR%=!"
        if "!NEW_PATH:~-1!"==";" set "NEW_PATH=!NEW_PATH:~0,-1!"
        if "!NEW_PATH:~0,1!"==";" set "NEW_PATH=!NEW_PATH:~1!"
        setx PATH "!NEW_PATH!" >nul
        echo PATH updated. Restart your terminal to take effect.
    ) else (
        echo PATH does not contain %BIN_DIR%, nothing to remove.
    )
) else (
    echo PATH variable not found, nothing to remove.
)

echo Uninstall completed. You may delete %LOCALAPPDATA%\nyado manually if still present.
pause