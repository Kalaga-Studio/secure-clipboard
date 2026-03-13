param(
    [switch]$WithMsi = $true
)

$ErrorActionPreference = "Stop"

Write-Host "Building secure-clipboard (release)..."
cargo build --release

if ($WithMsi) {
    Write-Host "Packaging MSI with cargo-wix..."
    cargo wix
}

Write-Host "Done."
