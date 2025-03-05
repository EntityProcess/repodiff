@echo off
echo Building RepoDiff...

REM Check if Rust is installed
where rustc >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Rust is not installed. Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Build the project
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Build failed.
    exit /b 1
)

echo Build successful!
echo Executable is located at: target\release\repodiff.exe

REM Copy config.json to the release directory
copy config.json target\release\
echo Copied config.json to target\release\

echo.
echo To run RepoDiff, use: .\target\release\repodiff.exe [options]
echo For help, use: .\target\release\repodiff.exe --help

exit /b 0 