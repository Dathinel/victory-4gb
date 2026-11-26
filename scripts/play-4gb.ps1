# Starts the title with the settings measured on a 4 GB card (README.md).
#   scripts\play-4gb.ps1 -Exe <path of your own copy's executable>
# -Build    target\diag (default; the ray-tracing skip list is a diagnostics switch) or target\release
# -Scale    the title's internal render scale (default 0.5; 0.6667 is the title's own)
# -More     other settings: -More @{VICTORY_TEXTURE_BUDGET_MB='2048'}
param(
    [Parameter(Mandatory)][string]$Exe,
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [string]$Scale = '0.5',
    [hashtable]$More = @{}
)
$settings = @{
    # fxceless/victory-fork's measured set ("run 253")
    VICTORY_LEVELS_ON_DEMAND = '1'; VICTORY_ASYNC_SUBMIT = '1'; VICTORY_QUEUE2 = '1'; VICTORY_INDIRECT_WINDOW = '1'
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_MERGE_GAP = '64'; VICTORY_MERGE_RUNS = '4096'; VICTORY_STAGING_POOL = '0'
    VICTORY_COPY_PLAIN = '1'; VICTORY_JOIN_PARTS = '1'; VICTORY_BACK_OWN_VIEW = '1'; VICTORY_INDIRECT_GATHER = '1'
    VICTORY_CARD_REHASH_RUNS = '1'
    # 4 GB: texture copies capped; the card is over its budget without it
    VICTORY_TEXTURE_BUDGET_MB = '1536'
    # picture
    VICTORY_BLEND_INT_OFF = '1'; VICTORY_BVH_NO_HIT = '1'; VICTORY_WAVE_LDS_LOOSE = '0'
    VICTORY_AMBIENT = '0.8'; VICTORY_AMBIENT_AIR = '1.5'; VICTORY_PIPELINE_WAIT_MS = '10000'; VICTORY_BAKE_SOURCES = '1'
    VICTORY_RENDER_SCALE = $Scale
    # a framed window with the frame rate in its title bar
    VICTORY_FULLSCREEN = '1'
}
# Without the ray-tracing structure builder every ray misses, and the passes that turn misses into
# light go white and glitter: they are skipped (the list is trustno1-coder/VIctory-NVIDIA's).
$rt = Join-Path $PSScriptRoot 'rt_off_hashes.txt'
if ($Build -eq 'diag' -and (Test-Path $rt)) { $settings.VICTORY_SKIP_PIPELINES = (Get-Content $rt -Raw).Trim() }
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = Join-Path $PSScriptRoot "..\target\$Build\victory.exe"
if (-not (Test-Path $victory)) { Write-Host "No $victory. Build it first (README.md)."; return }
if (-not (Test-Path $Exe)) { Write-Host "No $Exe."; return }
Get-ChildItem Env:VICTORY_* | ForEach-Object { Remove-Item "Env:$($_.Name)" }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
& $victory run (Resolve-Path $Exe).Path --console series-s --execute
