@echo off
setlocal EnableDelayedExpansion

REM Go to the folder where this .bat is located
cd /d "%~dp0"

:CHECK_PATH
REM Check if FOLDER_PATH is empty in python file
findstr /C:"FOLDER_PATH = \"\"" Android_Serve.py >nul
if %errorlevel% equ 0 goto ASK_PATH

findstr /C:"FOLDER_PATH = r\"\"" Android_Serve.py >nul
if %errorlevel% equ 0 goto ASK_PATH

:RUN_PY
REM Run the Python script
python Android_Serve.py

REM 5 is the exit code for Ctrl+E
if %errorlevel% equ 5 (
    goto ASK_PATH
)

REM Keep window open so you can see URL & QR
echo.
echo Press any key to close...
pause >nul
exit /b

:ASK_PATH
echo.
set /p NEW_PATH="Enter new FOLDER_PATH: "

REM Replace FOLDER_PATH in python file securely using Python
python -c "import re, sys; f=open('Android_Serve.py', 'r', encoding='utf-8'); c=f.read(); f.close(); c=re.sub(r'FOLDER_PATH\s*=\s*[rR]?\x22[^\x22]*\x22', lambda m: 'FOLDER_PATH = r\x22' + sys.argv[1].replace('\x22', '') + '\x22', c, count=1); f=open('Android_Serve.py', 'w', encoding='utf-8'); f.write(c); f.close()" "%NEW_PATH%"

goto RUN_PY
