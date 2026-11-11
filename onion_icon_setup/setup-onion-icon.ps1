$ErrorActionPreference = 'Stop'

# Run this script from anywhere; it locates the VS Code extension in your project.
$ProjectRoot = 'C:\Users\RIVAL\Downloads\Onion\Onion\onion-compiler'
$ExtensionDir = Join-Path $ProjectRoot 'vscode-onion'
$PackagePath = Join-Path $ExtensionDir 'package.json'
$ImageSource = Join-Path $PSScriptRoot 'onion.png'
$IconDir = Join-Path $ExtensionDir 'icons'
$IconTarget = Join-Path $IconDir 'onion.png'

if (-not (Test-Path $PackagePath)) { throw "Cannot find extension package.json: $PackagePath" }
if (-not (Test-Path $ImageSource)) { throw "Missing onion.png next to this script: $ImageSource" }

New-Item -ItemType Directory -Path $IconDir -Force | Out-Null
Copy-Item $ImageSource $IconTarget -Force

# Backup the existing extension manifest before modifying it.
Copy-Item $PackagePath "$PackagePath.bak" -Force
$pkg = Get-Content $PackagePath -Raw | ConvertFrom-Json
if (-not $pkg.contributes) { throw 'package.json is missing contributes.' }
if (-not $pkg.contributes.languages) { throw 'package.json is missing contributes.languages.' }

$language = @($pkg.contributes.languages | Where-Object { $_.id -eq 'onion' } | Select-Object -First 1)
if ($language.Count -eq 0) { throw 'Language ID onion not found in contributes.languages.' }
$language = $language[0]
$exts = @($language.extensions)
if ($exts -notcontains '.oni') { $exts += '.oni' }
if ($exts -notcontains '.onion') { $exts += '.onion' }
$language | Add-Member -NotePropertyName extensions -NotePropertyValue $exts -Force
$icon = [PSCustomObject]@{ light = './icons/onion.png'; dark = './icons/onion.png' }
$language | Add-Member -NotePropertyName icon -NotePropertyValue $icon -Force

# Raise no compiler changes: .oni and .onion both bind to the existing Onion language.
$pkg | ConvertTo-Json -Depth 100 | Set-Content $PackagePath -Encoding UTF8
Write-Host 'Added .onion extension and Onion logo to VS Code extension.' -ForegroundColor Green

Push-Location $ExtensionDir
try {
    if (-not (Get-Command npx -ErrorAction SilentlyContinue)) {
        throw 'npx is missing. Install Node.js, then rerun this script.'
    }
    & npx --yes @vscode/vsce package --no-dependencies
    if ($LASTEXITCODE -ne 0) { throw 'VSIX package failed.' }
    $vsix = Get-ChildItem -Path . -Filter '*.vsix' -File | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $vsix) { throw 'Could not find created .vsix file.' }
    if (-not (Get-Command code -ErrorAction SilentlyContinue)) { throw 'VS Code CLI command `code` not found.' }
    & code --install-extension $vsix.FullName --force
    if ($LASTEXITCODE -ne 0) { throw 'VS Code extension install failed.' }
}
finally { Pop-Location }

Write-Host ''
Write-Host 'Done. Reload VS Code (Ctrl+Shift+P -> Developer: Reload Window).' -ForegroundColor Green
Write-Host 'Create main.onion and check the Onion icon in Explorer.' -ForegroundColor Cyan
Write-Host 'If your active icon theme overrides language icons, try the built-in Seti theme.' -ForegroundColor Yellow
