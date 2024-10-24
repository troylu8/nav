param($other, [Switch]$h, [Switch]$help)

if ($h -or $help) {
    echo "help page"
}
elseif ($other) {
    echo "dont understand"
}
else {
    Clear-Host
    .\target\debug\map.exe
    Clear-Host
    cd (Get-Content -Path .\output.txt -TotalCount 1)
}