@ECHO off
cls
.\target\debug\map.exe
cls
for /f %%i in (output.txt) do cd %%i