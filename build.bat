@echo off
echo Building RepoDiff executable...
pyinstaller --onefile ^
  --hidden-import=tiktoken ^
  --hidden-import=tiktoken_ext.openai_public ^
  --hidden-import=tiktoken_ext ^
  repodiff_launcher.py ^
  --name repodiff

echo.
if %ERRORLEVEL% EQU 0 (
  echo Build successful! Executable is located in the dist folder.
  echo Path: %CD%\dist\repodiff.exe
) else (
  echo Build failed with error code %ERRORLEVEL%.
)
echo.