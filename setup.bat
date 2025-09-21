@echo off
echo MultiRichPresence Setup
echo ========================

echo.
echo Step 1: Checking Rust installation...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust is not installed!
    echo Please install Rust from https://rustup.rs/
    echo Then run this script again.
    pause
    exit /b 1
)
echo ✓ Rust is installed

echo.
echo Step 2: Building the application...
echo This may take a few minutes on first run...
cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    echo Check the error messages above.
    pause
    exit /b 1
)
echo ✓ Build completed successfully

echo.
echo Step 3: Setup instructions
echo ==========================
echo.
echo Before running the app, you need to:
echo 1. Go to https://discord.com/developers/applications
echo 2. Create a new application
echo 3. Copy the Application ID
echo 4. Replace APP_ID in main.rs with your Application ID
echo 5. Rebuild with: cargo build --release
echo.
echo The executable is located at: target\release\multi-rich-presence.exe
echo.

set /p choice="Do you want to run the app now? (y/n): "
if /i "%choice%"=="y" (
    echo.
    echo Starting MultiRichPresence...
    target\release\multi-rich-presence.exe
) else (
    echo.
    echo You can run the app later by executing:
    echo target\release\multi-rich-presence.exe
)

echo.
echo Setup complete! Don't forget to configure your Discord Application ID.
pause