
$zipName = "downloaded.zip"
$dir = $PWD.Path + "\downloaded"

Invoke-WebRequest "https://www.dropbox.com/scl/fi/82b8kpgthrr4mlip6co0c/sample.zip?rlkey=wa1yq19g0jnpj9sylo69nluan&st=334ezaa7&dl=1" -OutFile $zipName

Expand-Archive -Path $zipName
Remove-Item -Path $zipName

[Environment]::SetEnvironmentVariable(
    "PATH", 
    [Environment]::GetEnvironmentVariable("PATH", "User") + ";" + $dir, 
    "User"
)

New-Item -Path ($dir + "\output.txt")
[Environment]::SetEnvironmentVariable("MAP_HOME", $dir, "User")

pause

Remove-Item $PSCommandPath -Force