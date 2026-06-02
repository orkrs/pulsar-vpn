# ─────────────────────────────────────────────────────────────────────────
#  build.ps1 — обратно-совместимая обёртка над `cargo xtask build`.
#
#  Новый единый вход для сборки: `cargo xtask build [--release]`
#  См. BUILD.md или `cargo xtask help`.
# ─────────────────────────────────────────────────────────────────────────

[CmdletBinding()]
param(
    [switch]$Release = $false,
    [switch]$Dev     = $false,
    [switch]$Clean   = $false,
    [switch]$Lint    = $false,
    [switch]$Info    = $false,
    [switch]$Help    = $false
)

$ErrorActionPreference = "Stop"

$xtaskArgs = @()
if     ($Release) { $xtaskArgs += "--release" }
if     ($Dev)     { $xtaskArgs = @("dev");     $Release = $false }
elseif ($Clean)   { $xtaskArgs = @("clean") }
elseif ($Lint)    { $xtaskArgs = @("lint") }
elseif ($Info)    { $xtaskArgs = @("info") }
elseif ($Help)    { $xtaskArgs = @("help") }
else              { $xtaskArgs = @("build") }

Write-Host ""
Write-Host "  Pulsar VPN build (через cargo xtask)" -ForegroundColor Cyan
Write-Host "  $ cargo xtask $($xtaskArgs -join ' ')" -ForegroundColor Gray
Write-Host ""

& cargo xtask @xtaskArgs
exit $LASTEXITCODE
