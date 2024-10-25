param($other, [Switch]$h, [Switch]$help, [Switch]$uninstall)

$NAV_HOME = [Environment]::GetEnvironmentVariable("NAV_HOME", "User")

if ($h -or $help) {
    Get-Content ($NAV_HOME + "\help.txt")
}
elseif ($uninstall) {
    $NAV_HOME =  [Environment]::GetEnvironmentVariable("NAV_HOME", "User")
    $path = [Environment]::GetEnvironmentVariable("PATH", "User");

    Write-Output "removing environment variables.."
    [System.Environment]::SetEnvironmentVariable(
        "PATH",
        ($path.Split(';') | Where-Object { $_ -ne $NAV_HOME }) -join ';',
        "User"
    )
    [Environment]::SetEnvironmentVariable('NAV_HOME', [NullString]::Value, "User")

    Write-Output "deleting.."
    Remove-Item -Path $NAV_HOME -Recurse -Force

    Write-Output "done!"
    pause
}
elseif ($other) {
    Write-Output "usage: nav [-h | -help] [-uninstall]"
}
else {
    Clear-Host
    & ($NAV_HOME + "\map\map.exe")
    Clear-Host
    Set-Location (Get-Content -Path ($NAV_HOME + "\map\map_dest.txt") -TotalCount 1)
}