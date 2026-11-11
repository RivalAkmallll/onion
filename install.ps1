
# ================================================
# ONION PROGRAMMING LANGUAGE - WINDOWS INSTALLER
# ================================================

$ErrorActionPreference = "Stop"

$Root = $PSScriptRoot
$InstallDir = Join-Path $env:LOCALAPPDATA "Onion\bin"
$CompilerDir = Join-Path $Root "onion-compiler"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "       ONION LANGUAGE INSTALLER" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# --------------------------------
# 1. CHECK REQUIRED TOOLS
# --------------------------------

Write-Host "[1/6] Checking dependencies..." -ForegroundColor Yellow

foreach ($tool in @("cargo", "rustc")) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "$tool is not installed or not available in PATH."
    }
}

Write-Host "Rust and Cargo found." -ForegroundColor Green

# --------------------------------
# 2. FIND COMPILER PROJECT
# --------------------------------

Write-Host "[2/6] Locating Onion compiler..." -ForegroundColor Yellow

if (-not (Test-Path (Join-Path $CompilerDir "Cargo.toml"))) {
    throw "Cannot find onion-compiler/Cargo.toml. Put install.ps1 in your Onion project root."
}

Write-Host "Compiler: $CompilerDir" -ForegroundColor Green

# --------------------------------
# 3. BUILD ONION LANGUAGE SERVER
# --------------------------------

Write-Host "[3/6] Building Onion LSP..." -ForegroundColor Yellow

Push-Location $CompilerDir

try {
    & cargo build --bin onion-lsp

    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed."
    }
}
finally {
    Pop-Location
}

$TargetDir = Join-Path $CompilerDir "target\debug"
$LspSource = Join-Path $TargetDir "onion-lsp.exe"

if (-not (Test-Path $LspSource)) {
    throw "onion-lsp.exe was not found after building."
}

Write-Host "Onion LSP built successfully." -ForegroundColor Green

# --------------------------------
# 4. INSTALL EXECUTABLES
# --------------------------------

Write-Host "[4/6] Installing Onion executables..." -ForegroundColor Yellow

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

$LspDestination = Join-Path $InstallDir "onion-lsp.exe"

Copy-Item -Path $LspSource -Destination $LspDestination -Force

# Install Onion compiler CLI if available.
$CompilerExe = Join-Path $TargetDir "onion.exe"

if (Test-Path $CompilerExe) {
    Copy-Item $CompilerExe (Join-Path $InstallDir "onion.exe") -Force
    Write-Host "Installed onion.exe" -ForegroundColor Green
}
else {
    Write-Host "onion.exe not found; only LSP installed." -ForegroundColor DarkYellow
}

Write-Host "Installed onion-lsp.exe" -ForegroundColor Green

# --------------------------------
# 5. ADD ONION TO USER PATH
# --------------------------------

Write-Host "[5/6] Configuring PATH..." -ForegroundColor Yellow

$CurrentUserPath = [Environment]::GetEnvironmentVariable(
    "Path",
    "User"
)

$PathEntries = @(
    $CurrentUserPath -split ";" |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
)

if ($PathEntries -notcontains $InstallDir) {

    $NewPath = (
        @($PathEntries) + @($InstallDir)
    ) -join ";"

    [Environment]::SetEnvironmentVariable(
        "Path",
        $NewPath,
        "User"
    )

    Write-Host "Onion added to user PATH." -ForegroundColor Green
}
else {
    Write-Host "Onion already exists in PATH." -ForegroundColor Green
}

if (($env:Path -split ";") -notcontains $InstallDir) {
    $env:Path += ";$InstallDir"
}

# --------------------------------
# 6. CONFIGURE VSCODE
# --------------------------------

Write-Host "[6/6] Configuring VS Code..." -ForegroundColor Yellow

$SettingsDir = Join-Path $env:APPDATA "Code\User"
$SettingsFile = Join-Path $SettingsDir "settings.json"

New-Item -ItemType Directory -Path $SettingsDir -Force | Out-Null

# Convert the path into a JSON-safe string.
$JsonPath = ConvertTo-Json -InputObject $LspDestination -Compress

if (-not (Test-Path $SettingsFile)) {
    Set-Content -Path $SettingsFile -Value "{}" -Encoding UTF8
}

$SettingsText = Get-Content $SettingsFile -Raw

$SettingPattern = '("onion\.serverPath"\s*:\s*)"(?:\\.|[^"\\])*"'

if ([regex]::IsMatch($SettingsText, $SettingPattern)) {

    $SettingsText = [regex]::Replace(
        $SettingsText,
        $SettingPattern,
        [System.Text.RegularExpressions.MatchEvaluator]{
            param($match)
            return $match.Groups[1].Value + $JsonPath
        }
    )
}
else {
    # Add setting near the beginning of the root object.
    $OpenBrace = $SettingsText.IndexOf("{")

    if ($OpenBrace -lt 0) {
        throw "VS Code settings.json does not contain a JSON object."
    }

    $AfterBrace = $SettingsText.Substring($OpenBrace + 1)

    # Empty settings object needs no comma.
    $IsEmpty = $AfterBrace -match '^\s*\}\s*$'

    $Comma = if ($IsEmpty) { "" } else { "," }

    $SettingLine = "`r`n    `"onion.serverPath`": $JsonPath$Comma"

    $SettingsText = $SettingsText.Insert(
        $OpenBrace + 1,
        $SettingLine
    )
}

Set-Content -Path $SettingsFile -Value $SettingsText -Encoding UTF8

Write-Host "Onion LSP path configured." -ForegroundColor Green

# --------------------------------
# 7. FIND AND INSTALL EXTENSION
# --------------------------------

Write-Host ""
Write-Host "Searching for Onion VS Code extension..." -ForegroundColor Yellow

$VSCode = Get-Command "code" -ErrorAction SilentlyContinue

if ($VSCode) {

    $ExtensionDir = $null

    $Packages = Get-ChildItem `
        -Path $Root `
        -Filter "package.json" `
        -Recurse `
        -File `
        -ErrorAction SilentlyContinue |
        Where-Object {
            $_.FullName -notmatch '[\\/](node_modules|target)[\\/]'
        }

    foreach ($Package in $Packages) {

        try {
            $PackageJson = Get-Content $Package.FullName -Raw |
                ConvertFrom-Json

            $IsOnion = $false

            if ($PackageJson.contributes.languages) {
                foreach ($Language in $PackageJson.contributes.languages) {
                    if ($Language.id -eq "onion") {
                        $IsOnion = $true
                    }
                }
            }

            if ($IsOnion) {
                $ExtensionDir = $Package.DirectoryName
                break
            }
        }
        catch {
            continue
        }
    }

    if ($ExtensionDir) {

        Write-Host "Extension found: $ExtensionDir" -ForegroundColor Green

        $VSIX = Get-ChildItem `
            -Path $ExtensionDir `
            -Filter "*.vsix" `
            -File |
            Select-Object -First 1

        if (-not $VSIX) {

            $Npm = Get-Command npm -ErrorAction SilentlyContinue
            $Npx = Get-Command npx -ErrorAction SilentlyContinue

            if ($Npm -and $Npx) {

                Push-Location $ExtensionDir

                try {
                    if (Test-Path "package-lock.json") {
                        & npm ci
                    }
                    else {
                        & npm install
                    }

                    if ($LASTEXITCODE -ne 0) {
                        throw "npm dependency installation failed."
                    }

                    & npx --yes @vscode/vsce package --no-dependencies

                    if ($LASTEXITCODE -ne 0) {
                        throw "VS Code extension packaging failed."
                    }
                }
                finally {
                    Pop-Location
                }

                $VSIX = Get-ChildItem `
                    -Path $ExtensionDir `
                    -Filter "*.vsix" `
                    -File |
                    Sort-Object LastWriteTime -Descending |
                    Select-Object -First 1
            }
            else {
                Write-Warning "Node.js/npm/npx not found. Cannot package extension."
            }
        }

        if ($VSIX) {

            & code --install-extension $VSIX.FullName --force

            if ($LASTEXITCODE -eq 0) {
                Write-Host "Onion VS Code extension installed!" -ForegroundColor Green
            }
            else {
                Write-Warning "VS Code extension installation failed."
            }
        }
        else {
            Write-Warning "No VSIX file was generated."
        }
    }
    else {
        Write-Warning "Onion VS Code extension folder not found."
    }
}
else {
    Write-Warning "VS Code CLI 'code' is not available in PATH."
}

# --------------------------------
# INSTALLATION COMPLETED
# --------------------------------

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "       ONION INSTALL FINISHED" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installation directory:"
Write-Host $InstallDir -ForegroundColor Cyan
Write-Host ""
Write-Host "Language server:"
Write-Host $LspDestination -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Restart VS Code."
Write-Host "2. Open test.oni."
Write-Host "3. Check syntax highlighting and completion."
Write-Host "4. Open errors.oni and check diagnostics."
Write-Host ""
