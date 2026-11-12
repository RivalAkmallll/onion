# Onion Phase 18: strict reference acceptance tests.
# Requires PowerShell 5.1+; uses the existing tests/phase16 fixture files.
# Unlike the old runner, a semantic rejection cannot pass on a syntax/unsupported error.
param([string]$Compiler = "")
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $Compiler) { $Compiler = Join-Path $repo 'onion-compiler\target\release\onion.exe' }
if (-not (Test-Path -LiteralPath $Compiler)) { throw "Compiler not found: $Compiler. Run cargo build --release --bins." }
$Compiler = (Resolve-Path -LiteralPath $Compiler).Path
$tests = @(
    @{ Name='imm_borrow_valid.onion'; Success=$true; ExpectedOutput='Hello Onion' },
    @{ Name='mut_borrow_valid.onion'; Success=$true; ExpectedOutput='25' },
    @{ Name='borrow_conflict.onion'; Success=$false; Reason='borrow|already borrowed|conflict|mutable reference' },
    @{ Name='move_while_borrowed.onion'; Success=$false; Reason='borrowed|move.*borrow|cannot move' },
    @{ Name='dangling_scope.onion'; Success=$false; Reason='lifetime|outlive|dangling|does not live long enough|scope' },
    @{ Name='deref_non_reference.onion'; Success=$false; Reason='dereference|dereferenced|not a reference|cannot deref|type .*\*' },
    @{ Name='copy_value_valid.onion'; Success=$true; ExpectedOutput='10 10' },
    @{ Name='old_move_fail.onion'; Success=$false; Reason='moved value' }
)
function Invoke-Onion([string] $command, [string] $path) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Compiler
    $psi.Arguments = ('{0} "{1}"' -f $command, $path)
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $psi
    try {
        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEnd()
        $stderr = $process.StandardError.ReadToEnd()
        if (-not $process.WaitForExit(5000)) {
            try { $process.Kill() } catch {}
            return @{ Exit=-999; Text='TIMEOUT'; Stdout='' }
        }
        return @{ Exit=$process.ExitCode; Text=($stdout + "`n" + $stderr); Stdout=$stdout }
    } finally { $process.Dispose() }
}
$failed = 0
foreach ($t in $tests) {
    $path = Join-Path $repo ('tests\phase16\' + $t.Name)
    if (-not (Test-Path $path)) { throw "Missing fixture: $path" }
    $result = Invoke-Onion 'check' $path
    $ok = $false
    $details = ''
    if ($t.Success) {
        if ($result.Exit -eq 0) {
            $run = Invoke-Onion 'run' $path
            $ok = ($run.Exit -eq 0 -and $run.Stdout -match [regex]::Escape($t.ExpectedOutput))
            if (-not $ok) { $details = "Run exit $($run.Exit): $($run.Text)" }
        } else { $details = "Check exit $($result.Exit): $($result.Text)" }
    } else {
        $notImplemented = $result.Text -match 'not implemented|not supported|expected a statement|expected an expression|unrecognized character'
        $semanticError = $result.Text -match $t.Reason
        $ok = ($result.Exit -ne 0 -and $result.Exit -ne -999 -and $semanticError -and -not $notImplemented)
        if (-not $ok) { $details = "Expected a borrow/type-specific diagnostic; got exit $($result.Exit): $($result.Text)" }
    }
    if ($ok) { Write-Host "PASS $($t.Name)" -ForegroundColor Green }
    else { $failed++; Write-Host "FAIL $($t.Name)" -ForegroundColor Red; Write-Host $details }
}
Write-Host "Strict reference results: $($tests.Count-$failed)/$($tests.Count) passed."
if ($failed) { exit 1 }
