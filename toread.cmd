@ECHO off

IF "%~1"=="" (
    cls
    .\target\debug\map.exe
    cls
    for /f "delims=" %%i in (output.txt) do cd %%i
    goto end
)
IF "%~1"=="-h" GOTO help
IF "%~1"=="-help" GOTO help

echo dont understand

GOTO end

:help
ECHO helppage

:end