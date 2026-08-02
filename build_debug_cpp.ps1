[CmdletBinding()]
param(
    [string]$Architecture = "x64",
    [string]$Generator = "",
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$SourceDir = Join-Path $RepoRoot "cpp"
$BuildDir = Join-Path $RepoRoot "build\cpp-debug"
$OutputDir = Join-Path $RepoRoot "x64\debug"
$TauriResourcesDir = Join-Path $RepoRoot "src-tauri\resources"
$TauriRunExe = Join-Path $TauriResourcesDir "Run.exe"

if ($Clean -and (Test-Path $BuildDir)) {
    Remove-Item -LiteralPath $BuildDir -Recurse -Force
}

$configureArgs = @("-S", $SourceDir, "-B", $BuildDir)

if ($Generator) {
    $configureArgs += @("-G", $Generator)
} elseif (-not $env:CMAKE_GENERATOR) {
    $cmakeHelp = cmake --help
    if ($cmakeHelp -match "Visual Studio 18 2026") {
        $configureArgs += @("-G", "Visual Studio 18 2026")
    } elseif ($cmakeHelp -match "Visual Studio 17 2022") {
        $configureArgs += @("-G", "Visual Studio 17 2022")
    }
}

if (($configureArgs -contains "Visual Studio 18 2026") -or ($configureArgs -contains "Visual Studio 17 2022")) {
    $configureArgs += @("-A", $Architecture)
} else {
    $configureArgs += "-DCMAKE_BUILD_TYPE=Debug"
}

cmake @configureArgs
cmake --build $BuildDir --config Debug --parallel

if (Test-Path $OutputDir) {
    Remove-Item -LiteralPath $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$artifactPatterns = @(
    "3Dmigoto-Injector-V2\Debug\Run.*",
    "MessageBoxTestDll\Debug\MessageBoxTestDll.*"
)

foreach ($pattern in $artifactPatterns) {
    Get-ChildItem -Path (Join-Path $BuildDir $pattern) -File -ErrorAction SilentlyContinue |
        Copy-Item -Destination $OutputDir -Force
}

New-Item -ItemType Directory -Force -Path $TauriResourcesDir | Out-Null
if (Test-Path $TauriRunExe) {
    Remove-Item -LiteralPath $TauriRunExe -Force
}
Copy-Item -LiteralPath (Join-Path $OutputDir "Run.exe") -Destination $TauriRunExe -Force
