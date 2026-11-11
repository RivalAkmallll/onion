# Phase 16 reference/lifetime acceptance tests.
# Intentionally RED until the compiler implements references end-to-end.
param([string]$Compiler = "")
$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ([string]::IsNullOrWhiteSpace($Compiler)) {
    $Compiler = Join-Path $Repo 'onion-compiler/target/debug/onion.exe'
}
if (-not (Test-Path $Compiler)) {
    throw "Compiler not found: $Compiler . Run 'cargo build --bins' in onion-compiler."
}
$cases = @(
    @{ File='imm_borrow_valid.onion'; Want=0 },
    @{ File='mut_borrow_valid.onion'; Want=0 },
    @{ File='borrow_conflict.onion'; Want=1 },
    @{ File='move_while_borrowed.onion'; Want=1 },
    @{ File='dangling_scope.onion'; Want=1 },
    @{ File='deref_non_reference.onion'; Want=1 },
    @{ File='copy_value_valid.onion'; Want=0 },
    @{ File='old_move_fail.onion'; Want=1 }
)
$failures = 0
foreach ($case in $cases) {
    $path = Join-Path $Repo ('tests/phase16/' + $case.File)
   $result = cmd /c "`"$Compiler`" check `"$path`" 2>&1" | Out-String
    $actual = $LASTEXITCODE
    $pass = if ($case.Want -eq 0) { $actual -eq 0 } else { $actual -ne 0 }
    if ($pass) {
        Write-Host ('PASS ' + $case.File) -ForegroundColor Green
    } else {
        $failures++
        Write-Host ('FAIL ' + $case.File + ' (exit ' + $actual + ')') -ForegroundColor Red
        Write-Host $result
    }
}
if ($failures -gt 0) {
    Write-Host "$failures Phase 16 test(s) not yet passing." -ForegroundColor Yellow
    exit 1
}
Write-Host 'All Phase 16 check-level acceptance tests passed.' -ForegroundColor Green
Write-Host 'NEXT: verify run/build produce matching behavior and use-after-scope is impossible.'
