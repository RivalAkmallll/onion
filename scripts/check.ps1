$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent
$CompilerDir = Join-Path $Root "onion-compiler"

Push-Location $CompilerDir

try {
    Write-Host "Checking Onion compiler..." -ForegroundColor Cyan

    & cargo check --all-targets

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo check failed."
    }

    Write-Host "Building Onion..." -ForegroundColor Cyan

    & cargo build --bins

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed."
    }

    Write-Host "Running Rust tests..." -ForegroundColor Cyan

    & cargo test

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo tests failed."
    }
}
finally {
    Pop-Location
}

$TestFile = Join-Path $Root "tests/smoke.ps1"

& $TestFile

if (-not $?) {
    throw "Onion smoke test failed."
}

Write-Host "All Onion checks passed!" -ForegroundColor Green
