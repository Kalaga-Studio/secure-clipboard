param(
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$toolsDir = Join-Path $root 'tools\wix314'
$zipPath = Join-Path $toolsDir 'wix314-binaries.zip'
$candlePath = Join-Path $toolsDir 'candle.exe'

Write-Host "==> Using repo root: $root"

if (-not (Test-Path $candlePath)) {
    Write-Host "==> WiX 3.14 not found at $toolsDir, downloading..."
    New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

    $uri = 'https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip'
    Write-Host "==> Downloading WiX binaries from $uri"
    Invoke-WebRequest -Uri $uri -OutFile $zipPath

    Write-Host "==> Extracting WiX binaries..."
    Expand-Archive -Force -Path $zipPath -DestinationPath $toolsDir
}
else {
    Write-Host "==> Found existing WiX binaries at $toolsDir"
}

if ($SkipBuild) {
    Write-Host "==> SkipBuild specified; not running cargo wix."
    return
}

if (-not (Get-Command cargo-wix -ErrorAction SilentlyContinue)) {
    Write-Host "==> cargo-wix not found; installing..."
    cargo install cargo-wix
}

Write-Host "==> Building MSI with cargo wix..."
Push-Location $root
try {
    cargo wix -b $toolsDir
}
finally {
    Pop-Location
}

