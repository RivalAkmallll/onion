param(
    [string]$CompilerPath
)

$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent

if (-not $CompilerPath) {
    $CompilerPath = Join-Path $Root "onion-compiler/target/debug/onion.exe"
}

$CompilerPath = [System.IO.Path]::GetFullPath($CompilerPath)

if (-not (Test-Path $CompilerPath)) {
    throw "Onion compiler not found: $CompilerPath"
}

$Example = Join-Path $Root "examples/hello-world.onion"

Write-Host "Running Onion smoke test..." -ForegroundColor Cyan

$Output = & $CompilerPath run $Example 2>&1 | Out-String
$ExitCode = $LASTEXITCODE

Write-Host $Output

if ($ExitCode -ne 0) {
    throw "Onion compiler exited with code $ExitCode"
}

if ($Output -notmatch [regex]::Escape("Hello, Onion!")) {
    throw "Expected output was not found."
}

Write-Host "PASS: Hello World" -ForegroundColor Green
