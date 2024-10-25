@ECHO off

IF "%~1"=="" (
    CLS
    %NAV_HOME%\map\map.exe
    CLS
    FOR /F "delims=" %%i IN (%NAV_HOME%\map\map_dest.txt) DO CD %%i
    GOTO end
)
IF "%~1"=="-h" GOTO help
IF "%~1"=="-help" GOTO help
IF "%~1"=="-uninstall" GOTO uninstall

ECHO usage: nav [-h ^| -help] [-uninstall]

GOTO end

:help
more %NAV_HOME%\help.txt
GOTO end

:uninstall
powershell nav.ps1

:end