

$MAP_HOME =  [Environment]::GetEnvironmentVariable("MAP_HOME", "User")
$path = [Environment]::GetEnvironmentVariable("PATH", "User");

[System.Environment]::SetEnvironmentVariable(
    "PATH",
    ($path.Split(';') | Where-Object { $_ -ne $MAP_HOME }) -join ';',
    "User"
)

Remove-Item -Path $MAP_HOME -Recurse
[Environment]::SetEnvironmentVariable('MAP_HOME', [NullString]::Value, "User")

pause