$ErrorActionPreference = 'Stop'
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$packageArgs = @{
    packageName   = 'tiny-settlements'
    fileType      = 'exe'
    url           = 'https://github.com/mlm-games/tiny-settlements/releases/latest'
    softwareName  = 'tiny-settlements'
    checksum      = ''
    checksumType  = 'sha256'
}
Install-ChocolateyPackage @packageArgs
