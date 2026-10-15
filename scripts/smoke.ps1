#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Clipper end-to-end smoke test.

.DESCRIPTION
    Runs the release binary fully isolated under a temporary CLIPPER_HOME with a
    faked Claude provider, and asserts the whole pipeline end to end:

      * config + Obsidian-vault auto-detect fallback
      * the REST API (healthz, items, today, stats)
      * the phone-pairing endpoints (pair, pair.png, save-without-token)
      * queue -> drainer -> vault -> index

    Builds the release binary first if it is missing. The temp home is left in
    place so the generated notes can be inspected.

    Prints "SMOKE OK" and exits 0 on success, or exits non-zero with a diff.

.EXAMPLE
    pwsh -File scripts/smoke.ps1
#>

$ErrorActionPreference = 'Stop'

$repo        = Split-Path -Parent $PSScriptRoot
$exe         = Join-Path $repo 'src-tauri\target\release\clipper.exe'
$stamp       = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
$clipperHome = Join-Path $env:TEMP "clipper-smoke-$stamp"
$vault       = Join-Path $clipperHome 'ClipperVault'
$queueDir    = Join-Path $clipperHome 'queue'
$staging     = Join-Path $clipperHome 'staging'
$config      = Join-Path $clipperHome 'config.toml'
$urls        = @('https://example.com/a', 'https://example.org/b', 'https://example.net/c')

$script:checks = 0

function Write-Step([string]$text) {
    Write-Host ''
    Write-Host "== $text" -ForegroundColor Cyan
}

function Stop-App {
    Get-Process clipper -ErrorAction SilentlyContinue |
        Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 900
}

function Show-Log([string]$path) {
    if (Test-Path $path) { return (Get-Content $path -Raw) }
    return '(no log)'
}

function Check([string]$what, [bool]$ok, [string]$detail = '') {
    if ($ok) {
        $script:checks++
        Write-Host "  ok   $what"
        return
    }
    Write-Host ''
    Write-Host "SMOKE FAILED: $what" -ForegroundColor Red
    if ($detail) { Write-Host $detail -ForegroundColor DarkGray }
    Stop-App
    exit 1
}

function Get-Text($response) {
    if ($response.Content -is [byte[]]) {
        return [System.Text.Encoding]::UTF8.GetString($response.Content)
    }
    return [string]$response.Content
}

# Any unexpected terminating error must still leave no stray Clipper process,
# or the leftover app keeps this script's output handle open forever.
trap {
    Stop-App
    Write-Host ''
    Write-Host "SMOKE FAILED: $_" -ForegroundColor Red
    exit 1
}

Stop-App

# --------------------------------------------------------------- build

Write-Step 'release binary'
if (-not (Test-Path $exe)) {
    Write-Host '  building (npx tauri build --no-bundle)…'
    Push-Location $repo
    try {
        & npx tauri build --no-bundle
        if ($LASTEXITCODE -ne 0) { throw "tauri build exited $LASTEXITCODE" }
    }
    finally { Pop-Location }
}
Check 'release binary exists' (Test-Path $exe) $exe

# ------------------------------------------------- isolated home + first run

Write-Step 'isolated CLIPPER_HOME'
Stop-App
Remove-Item $clipperHome -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $clipperHome | Out-Null
$env:CLIPPER_HOME = $clipperHome
Write-Host "  CLIPPER_HOME=$clipperHome"

Write-Step 'first run: config + vault auto-detect fallback'
$bootLog = Join-Path $clipperHome 'boot.log'
Start-Process -FilePath $exe -PassThru `
    -RedirectStandardOutput $bootLog `
    -RedirectStandardError (Join-Path $clipperHome 'boot.err') | Out-Null
Start-Sleep -Seconds 8
Stop-App

Check 'config.toml written' (Test-Path $config) (Show-Log $bootLog)
Check 'vault fallback created at <CLIPPER_HOME>/ClipperVault' `
    (Test-Path (Join-Path $vault '.obsidian')) `
    "expected $(Join-Path $vault '.obsidian')"

$vaultPath = ''
$pathMatch = Select-String -Path $config -Pattern "^path\s*=\s*'(.+)'" | Select-Object -First 1
if ($pathMatch) { $vaultPath = $pathMatch.Matches[0].Groups[1].Value }
Check 'config vault path resolves inside CLIPPER_HOME' `
    ($vaultPath.StartsWith($clipperHome)) `
    "vault.path = $vaultPath"

foreach ($part in @('soul', 'user', 'memory')) {
    $partPath = Join-Path $clipperHome "memory\$part.md"
    Check "memory\$part.md seeded on first run" (Test-Path $partPath) $partPath
}

# ------------------------------------------- test config + fake provider

Write-Step 'configuring the run: port 0, faked provider'
New-Item -ItemType Directory -Force -Path $staging | Out-Null
New-Item -ItemType Directory -Force -Path $queueDir | Out-Null

$notePaths = @()
for ($i = 0; $i -lt 3; $i++) {
    $notePath = Join-Path $staging "fake-$($i + 1).md"
    $notePaths += $notePath
    $body = @"
---
type: article
source: example
url: $($urls[$i])
saved_at: 2026-01-0$(($i + 1))T00:00:00Z
title: Smoke note $($i + 1)
summary: Written by the smoke test.
tags: [smoke]
---

# Smoke note $($i + 1)
"@
    Set-Content -Path $notePath -Value $body -NoNewline
}
$listFile = Join-Path $staging 'fake-paths.txt'
Set-Content -Path $listFile -Value ($notePaths -join "`r`n") -NoNewline

$text = Get-Content $config -Raw
$text = $text -replace '(?m)^port\s*=\s*\d+', 'port = 0'
$text = $text -replace '(?m)^command\s*=\s*"claude"', "command = 'cmd'"
$text = $text -replace '(?m)^args\s*=\s*\["-p"\]', "args = ['/c', 'type', '$listFile']"
Set-Content -Path $config -Value $text -NoNewline

Check 'server port set to 0' ($text -match '(?m)^port = 0') $text
Check 'provider.command faked to cmd' ($text -match "command = 'cmd'") $text

# ------------------------------------------------------------- start server

Write-Step 'starting the server'
$runLog = Join-Path $clipperHome 'run.log'
Start-Process -FilePath $exe -PassThru `
    -RedirectStandardOutput $runLog `
    -RedirectStandardError (Join-Path $clipperHome 'run.err') | Out-Null

$port = 0
$deadline = (Get-Date).AddSeconds(30)
while ((Get-Date) -lt $deadline -and $port -eq 0) {
    Start-Sleep -Milliseconds 500
    $found = Select-String -Path $runLog -ErrorAction SilentlyContinue `
        -Pattern 'server: listening on http://127\.0\.0\.1:(\d+)' | Select-Object -First 1
    if ($found) { $port = [int]$found.Matches[0].Groups[1].Value }
}
Check 'server reported its port within 30s' ($port -gt 0) (Show-Log $runLog)
Write-Host "  port=$port"
$base = "http://127.0.0.1:$port"

# --------------------------------------------------------- API + pairing

Write-Step 'REST + pairing endpoints'
$health = Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 10
Check "GET /healthz -> 200 'ok'" `
    ($health.StatusCode -eq 200 -and (Get-Text $health).Trim() -eq 'ok') `
    "$($health.StatusCode) $(Get-Text $health)"

$items = Invoke-WebRequest "$base/api/items" -UseBasicParsing -TimeoutSec 10
$itemsJson = (Get-Text $items) | ConvertFrom-Json
Check 'GET /api/items -> empty to start' `
    ($itemsJson.total -eq 0 -and @($itemsJson.items).Count -eq 0) `
    (Get-Text $items)

$qr = Invoke-WebRequest "$base/pair.png" -UseBasicParsing -TimeoutSec 10
$qrBytes = @($qr.Content)
Check 'GET /pair.png -> image/png, non-empty' `
    ($qr.StatusCode -eq 200 -and "$($qr.Headers['Content-Type'])" -like 'image/png*' -and $qrBytes.Count -gt 100) `
    "type=$($qr.Headers['Content-Type']) bytes=$($qrBytes.Count)"

$pair = Invoke-WebRequest "$base/pair" -UseBasicParsing -TimeoutSec 10
$pairBody = Get-Text $pair
Check 'GET /pair -> the save page' `
    ($pair.StatusCode -eq 200 -and ($pairBody -match '<input' -or $pairBody -match 'type="url"')) `
    $pairBody.Substring(0, [Math]::Min(200, $pairBody.Length))

$unauthorised = 0
try {
    Invoke-WebRequest "$base/save" -Method POST -UseBasicParsing -TimeoutSec 10 `
        -ContentType 'application/json' -Body '{"url":"https://example.com/nope"}' | Out-Null
}
catch { $unauthorised = [int]$_.Exception.Response.StatusCode }
Check 'POST /save without a token -> 401' ($unauthorised -eq 401) "got $unauthorised"

# ------------------------------------------------- queue -> drainer -> vault

Write-Step 'queueing 3 URLs, letting the drainer run'
$lines = for ($i = 0; $i -lt 3; $i++) {
    [pscustomobject]@{
        id       = "smoke-$($i + 1)"
        url      = $urls[$i]
        source   = 'clipboard'
        type     = 'url'
        saved_at = "2026-01-0$($i + 1)T00:00:00Z"
    } | ConvertTo-Json -Compress
}
Set-Content -Path (Join-Path $queueDir 'inbox.jsonl') -Value (($lines -join "`n") + "`n") -NoNewline

$inbox = Join-Path $vault 'Inbox'
$filed = @()
$deadline = (Get-Date).AddSeconds(30)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 700
    $filed = @(Get-ChildItem $inbox -Filter *.md -ErrorAction SilentlyContinue)
    if ($filed.Count -ge 3) { break }
}
Check '3 notes filed in the vault within 30s' ($filed.Count -eq 3) `
    "found $($filed.Count) in $inbox`n$(Show-Log $runLog)"

# ------------------------------------------------------------ assertions

Write-Step 'pipeline assertions'
# The indexer debounces 500ms, so give it a moment to catch up with the drainer.
$deadline = (Get-Date).AddSeconds(30)
$itemsJson = $null
while ((Get-Date) -lt $deadline) {
    $itemsJson = (Get-Text (Invoke-WebRequest "$base/api/items" -UseBasicParsing -TimeoutSec 10)) |
        ConvertFrom-Json
    if ($itemsJson.total -eq 3) { break }
    Start-Sleep -Milliseconds 600
}
Check 'GET /api/items -> 3 results' `
    ($itemsJson.total -eq 3 -and @($itemsJson.items).Count -eq 3) `
    ($itemsJson | ConvertTo-Json -Compress)

$today = Invoke-WebRequest "$base/api/today" -UseBasicParsing -TimeoutSec 10
$todayJson = (Get-Text $today) | ConvertFrom-Json
Check 'GET /api/today -> 3 or fewer, not empty' `
    (@($todayJson.items).Count -le 3 -and @($todayJson.items).Count -gt 0) `
    (Get-Text $today)

$left = @(Get-Content (Join-Path $queueDir 'inbox.jsonl') -ErrorAction SilentlyContinue |
    Where-Object { $_.Trim() -ne '' })
Check 'queue\inbox.jsonl is drained' ($left.Count -eq 0) "$($left.Count) lines left"

$archived = @(Get-Content (Join-Path $queueDir 'archive.jsonl') -ErrorAction SilentlyContinue |
    Where-Object { $_.Trim() -ne '' })
Check 'queue\archive.jsonl has 3 entries' ($archived.Count -eq 3) "found $($archived.Count)"

$dead = @(Get-Content (Join-Path $queueDir 'deadletter.jsonl') -ErrorAction SilentlyContinue |
    Where-Object { $_.Trim() -ne '' })
Check 'no deadletter' ($dead.Count -eq 0) ($dead -join "`n")

# --------------------------------------------------------------- shutdown

Write-Step 'shutting down'
Stop-App
Check 'no clipper process left behind' `
    (@(Get-Process clipper -ErrorAction SilentlyContinue).Count -eq 0)

Write-Host ''
Write-Host "SMOKE OK ($script:checks checks)" -ForegroundColor Green
Write-Host "  home:  $clipperHome"
Write-Host "  vault: $vault"
exit 0
