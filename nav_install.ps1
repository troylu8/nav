
$zipName = "nav_cli.zip"
$dir = $PWD.Path + "\nav_cli"

Write-Output "downloading zip.."
Invoke-WebRequest "https://www.dropbox.com/scl/fi/j9e6vxgba0amfocavr1hw/nav_cli.zip?rlkey=bsawm942hfi4j7p0ljrdyfu4n&st=yjzv9h0r&dl=1" -OutFile $zipName

Write-Output "unzipping.."
Expand-Archive -Path $zipName
Remove-Item -Path $zipName

Write-Output "setting environment variables.."
[Environment]::SetEnvironmentVariable(
    "PATH", 
    [Environment]::GetEnvironmentVariable("PATH", "User") + ";" + $dir, 
    "User"
)
[Environment]::SetEnvironmentVariable("NAV_HOME", $dir, "User")

Write-Output "done!"
Pause

Remove-Item $PSCommandPath -Force