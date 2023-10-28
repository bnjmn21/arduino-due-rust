@echo off

set port=COM4
set flashsize=524288

cargo objcopy --release -- -O binary app.bin

call :getsize app.bin

if %size% GEQ %flashsize% (
    echo ----BINARY TOO LARGE----
    echo %size% of %flashsize% bytes
    exit 1
) else (
    echo using %size% of %flashsize% bytes
)

C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -noprofile -command "$port = new-Object System.IO.Ports.SerialPort %port%,1200,None,8,one; $port.Open(); $port.Close(); .\\bossac.exe -p %port% -e -w -v -b -R app.bin"
goto :eof


:getsize
set size=%~z1
goto :eof