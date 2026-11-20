param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$exe = (Resolve-Path -LiteralPath $ExePath).Path

function Assert-Exit([int]$Expected) {
    if ($LASTEXITCODE -ne $Expected) {
        throw "CLI exit code $LASTEXITCODE, expected $Expected"
    }
}

$capabilities = & $exe describe | ConvertFrom-Json
Assert-Exit 0
if ($capabilities.schema_version -ne 1) {
    throw 'CLI capability schema version is missing.'
}

$single = & $exe lookup '+86 132 1300 0000' '123' | ConvertFrom-Json
Assert-Exit 0
if ($single.schema_version -ne 1 -or
    $single.results.Count -ne 2 -or
    $single.results[0].phone -ne '13213000000' -or
    $single.results[0].status -ne 'found' -or
    $single.results[0].carrier -ne '中国联通' -or
    $single.results[0].province -ne '河南省' -or
    $single.results[0].city -ne '郑州市' -or
    $single.results[1].status -ne 'invalid' -or
    $null -ne $single.results[1].carrier) {
    throw 'CLI single lookup returned unexpected JSON.'
}

$fromStdin = '["13213000000","13900200000"]' | & $exe lookup --stdin | ConvertFrom-Json
Assert-Exit 0
if ($fromStdin.results.Count -ne 2 -or
    $fromStdin.results[0].carrier -ne '中国联通' -or
    $fromStdin.results[1].carrier -ne '中国移动') {
    throw 'CLI standard input lookup returned unexpected JSON.'
}

$invalidCall = & $exe lookup 2>&1 | Out-String
Assert-Exit 2
if ($invalidCall -notmatch 'invalid_request') {
    throw 'CLI did not report a structured invocation error.'
}

$work = Join-Path ([System.IO.Path]::GetTempPath()) ('PhoneRegion CLI smoke ' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $source = (Resolve-Path -LiteralPath ./docs/example.csv).Path
    foreach ($format in @('csv', 'xlsx')) {
        $output = Join-Path $work "results.$format"
        $batch = & $exe batch $source $output | ConvertFrom-Json
        Assert-Exit 0
        if ($batch.schema_version -ne 1 -or $batch.rows -ne 3 -or
            -not (Test-Path -LiteralPath $output)) {
            throw "CLI batch lookup failed for $format."
        }
        if ($format -eq 'csv') {
            $bytes = [System.IO.File]::ReadAllBytes($output)
            $contents = [System.IO.File]::ReadAllText($output)
            if ($bytes.Length -lt 3 -or
                $bytes[0] -ne 0xEF -or
                $bytes[1] -ne 0xBB -or
                $bytes[2] -ne 0xBF -or
                -not $contents.Contains('13213000000,中国联通,河南省,郑州市')) {
                throw 'CLI CSV output differs from the expected result.'
            }
        }
        $duplicate = & $exe batch $source $output 2>&1 | Out-String
        Assert-Exit 1
        if ($duplicate -notmatch 'operation_failed') {
            throw 'CLI did not protect an existing export.'
        }
    }
}
finally {
    Remove-Item -LiteralPath $work -Recurse -Force
}

Write-Host "CLI smoke passed: $exe"
exit 0
