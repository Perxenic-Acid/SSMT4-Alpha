# Build helper binary: wuwa_settings.exe
param(
    [switch]$Release = $false
)

$ErrorActionPreference = "Stop"
$RepoRoot = $PSScriptRoot
$CargoPath = if ($Env:CARGO) { $Env:CARGO } else { "cargo.exe" }
$HelperManifest = Join-Path $RepoRoot "src-tauri" "Cargo.toml"
$HelperProfileArgs = if ($Release) { @("--release") } else { @() }
$HelperProfileDir = if ($Release) { "release" } else { "debug" }
$HelperTargetDir = Join-Path $RepoRoot "src-tauri" "resources"

function Build-HelperBinary {
    param(
        [string]$BinName
    )

    Write-Host "`nBuilding $BinName..." -ForegroundColor Cyan

    $HelperExecutableName = "$BinName.exe"
    $HelperArgs = @("build", "--manifest-path", $HelperManifest, "--bin", $BinName) + $HelperProfileArgs

    $Process = Start-Process -FilePath $CargoPath -ArgumentList $HelperArgs -NoNewWindow -Wait -PassThru -WorkingDirectory $RepoRoot

    if ($Process.ExitCode -ne 0) {
        Write-Host "Failed to build $BinName" -ForegroundColor Red
        exit $Process.ExitCode
    }

    $HelperSource = Join-Path $RepoRoot "src-tauri" "target" $HelperProfileDir $HelperExecutableName
    $HelperTarget = Join-Path $HelperTargetDir $HelperExecutableName

    if (-not (Test-Path $HelperSource)) {
        Write-Host "$BinName binary not found at $HelperSource" -ForegroundColor Red
        exit 1
    }

    if (-not (Test-Path $HelperTargetDir)) {
        New-Item -ItemType Directory -Path $HelperTargetDir -Force | Out-Null
    }

    Copy-Item -Path $HelperSource -Destination $HelperTarget -Force
    Write-Host "Copied $BinName to $HelperTarget" -ForegroundColor Green
}

# Build the helper
Build-HelperBinary "wuwa_settings"

Write-Host "`nSuccessfully built helper binary!" -ForegroundColor Green
