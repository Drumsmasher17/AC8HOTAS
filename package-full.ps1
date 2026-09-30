param(
    [Parameter(Mandatory=$true)][string]$GameBinDir
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$dist = Join-Path $PSScriptRoot 'dist'
$bundlePath = Join-Path $dist 'AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip'
if (-not (Test-Path -LiteralPath $bundlePath)) { throw 'Run package.ps1 first.' }
$pinned = @{
    'dwmapi.dll'='C5D2AB9F9B89BD94460B0A283EEFB113085105014011CAC961F36787376DB744'
    'UE4SS\UE4SS.dll'='3FC1CD877007499E8C4F2152E12F76098ABA8E267B7EE65846EDC17CF7A0D1A4'
}
foreach ($item in $pinned.GetEnumerator()) {
    if ((Get-FileHash -LiteralPath (Join-Path $GameBinDir $item.Key) -Algorithm SHA256).Hash -ne $item.Value) {
        throw "Unexpected loader binary: $($item.Key). Review provenance before changing the pin."
    }
}
$name = 'AC8HOTAS-0.1.0-full-install.zip'
$output = Join-Path $dist $name
$prefix = 'Game/Binaries/Win64/'
$bundle = [IO.Compression.ZipFile]::OpenRead($bundlePath)
$stream = [IO.File]::Open($output,[IO.FileMode]::Create)
$zip = [IO.Compression.ZipArchive]::new($stream,[IO.Compression.ZipArchiveMode]::Create)
function Add-Text($Name,$Text) {
    $entry=$zip.CreateEntry($Name)
    $writer=[IO.StreamWriter]::new($entry.Open(),[Text.UTF8Encoding]::new($false))
    try { $writer.Write($Text) } finally { $writer.Dispose() }
}
try {
    foreach ($entry in $bundle.Entries) {
        if ($entry.FullName.StartsWith('AC8HOTAS/') -or $entry.FullName.StartsWith('AC8AnalogYaw/')) {
            $target=$zip.CreateEntry($prefix+'UE4SS/Mods/'+$entry.FullName,[IO.Compression.CompressionLevel]::Optimal)
            $sourceStream=$entry.Open(); $targetStream=$target.Open()
            try { $sourceStream.CopyTo($targetStream) } finally { $sourceStream.Dispose(); $targetStream.Dispose() }
        }
    }
    $files=[ordered]@{}
    $files[$prefix+'dwmapi.dll']=Join-Path $GameBinDir 'dwmapi.dll'
    $files[$prefix+'UE4SS/UE4SS.dll']=Join-Path $GameBinDir 'UE4SS\UE4SS.dll'
    $files[$prefix+'UE4SS/LICENSE']=Join-Path $PSScriptRoot 'full-install\UE4SS-LICENSE.txt'
    $files[$prefix+'UE4SS/UE4SS-settings.ini']=Join-Path $PSScriptRoot 'full-install\UE4SS-settings.ini'
    $files['INSTALL-FULL.md']=Join-Path $PSScriptRoot 'full-install\INSTALL-FULL.md'
    $files['LOADER-PROVENANCE.md']=Join-Path $PSScriptRoot 'full-install\LOADER-PROVENANCE.md'
    foreach ($file in $files.GetEnumerator()) {
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip,$file.Value,$file.Key,[IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
    Add-Text ($prefix+'override.txt') "UE4SS`r`n"
    Add-Text ($prefix+'UE4SS/Mods/mods.txt') "AC8HOTAS : 1`r`nAC8AnalogYaw : 1`r`n"
    Add-Text ($prefix+'UE4SS/Mods/mods.json') '[{"mod_name":"AC8HOTAS","mod_enabled":true},{"mod_name":"AC8AnalogYaw","mod_enabled":true}]'
} finally { $zip.Dispose(); $stream.Dispose(); $bundle.Dispose() }
$sumPath=Join-Path $dist 'SHA256SUMS.txt'
$lines=@(Get-Content -LiteralPath $sumPath | Where-Object { -not $_.EndsWith('  '+$name) })
$lines += (Get-FileHash -LiteralPath $output -Algorithm SHA256).Hash+'  '+$name
[IO.File]::WriteAllLines($sumPath,$lines,[Text.Encoding]::ASCII)
Write-Output $output
