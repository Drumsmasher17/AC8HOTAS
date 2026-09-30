param(
    [string]$AnalogYawZip = (Join-Path $PSScriptRoot '..\AC8AnalogYaw\dist\AC8AnalogYaw-0.1.3.zip')
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$dist = Join-Path $PSScriptRoot 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$scripts = @('main.lua','mapper.lua','actions.lua','session.lua','ac8_hotas_devices_001.dll')
$hotas = [ordered]@{}
foreach ($file in $scripts) { $hotas['AC8HOTAS/Scripts/'+$file] = Join-Path $PSScriptRoot ('mod\Scripts\'+$file) }
$hotas['AC8HOTAS/bindings.lua'] = Join-Path $PSScriptRoot 'mod\bindings.lua'
foreach ($file in @('README.md','LICENSE','INSTALL.md','THIRD_PARTY_NOTICES.md')) {
    $hotas['AC8HOTAS/'+$file] = Join-Path $PSScriptRoot $file
}
$hotas['INSTALL.md'] = Join-Path $PSScriptRoot 'INSTALL.md'
$hotas['RELEASE_NOTES.md'] = Join-Path $PSScriptRoot 'RELEASE_NOTES.md'
foreach ($path in $hotas.Values) { if (-not (Test-Path -LiteralPath $path)) { throw "Missing release input: $path. Run build.cmd first." } }

function Write-Package($Name, $Files, $ExtraEntries) {
    $path = Join-Path $dist $Name
    $stream = [IO.File]::Open($path,[IO.FileMode]::Create)
    $zip = [IO.Compression.ZipArchive]::new($stream,[IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($entry in $Files.GetEnumerator()) {
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip,$entry.Value,$entry.Key,[IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
        foreach ($entry in $ExtraEntries) {
            $target = $zip.CreateEntry($entry.FullName,[IO.Compression.CompressionLevel]::Optimal)
            $sourceStream = $entry.Open(); $targetStream = $target.Open()
            try { $sourceStream.CopyTo($targetStream) } finally { $sourceStream.Dispose(); $targetStream.Dispose() }
        }
    } finally { $zip.Dispose(); $stream.Dispose() }
    Write-Output $path
}

Write-Package 'AC8HOTAS-0.1.0.zip' $hotas @()
# Dependency files are copied unchanged from a separately versioned release.
# Do not embed game files, personal config, UE4SS settings, or a global mod list.
$yawNames = @('AC8AnalogYaw/Scripts/main.lua','AC8AnalogYaw/Scripts/ac8_analog_yaw_013.dll',
    'AC8AnalogYaw/README.md','AC8AnalogYaw/LICENSE','AC8AnalogYaw/THIRD_PARTY_NOTICES.md','AC8AnalogYaw/MinHook-LICENSE.txt')
if (-not (Test-Path -LiteralPath $AnalogYawZip)) { throw 'Supply -AnalogYawZip with the AC8AnalogYaw 0.1.3 release ZIP.' }
$yaw = [IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $AnalogYawZip).Path)
try {
    $extras = @()
    foreach ($name in $yawNames) {
        $entry = $yaw.GetEntry($name)
        if (-not $entry) { throw "Analog yaw archive missing $name" }
        $extras += $entry
    }
    Write-Package 'AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip' $hotas $extras
} finally { $yaw.Dispose() }

$source = [ordered]@{}
$sourceFiles = @('README.md','INSTALL.md','RELEASE_NOTES.md','PUBLISHING.md','VALIDATION.md','LICENSE',
    'THIRD_PARTY_NOTICES.md','.gitignore','build.cmd','package.ps1','src/devices.cpp',
    'tests/test_mapper.py','tests/prepare_catalog.py','tests/test_package.py','mod/bindings.lua')
foreach ($file in $scripts) { if ($file.EndsWith('.lua')) { $sourceFiles += 'mod/Scripts/'+$file } }
foreach ($file in $sourceFiles) { $source['AC8HOTAS/'+$file] = Join-Path $PSScriptRoot $file }
Write-Package 'AC8HOTAS-0.1.0-source.zip' $source @()
$names = @('AC8HOTAS-0.1.0.zip','AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip','AC8HOTAS-0.1.0-source.zip')
$hashes = foreach ($name in $names) { (Get-FileHash -LiteralPath (Join-Path $dist $name) -Algorithm SHA256).Hash+'  '+$name }
[IO.File]::WriteAllLines((Join-Path $dist 'SHA256SUMS.txt'),$hashes,[Text.Encoding]::ASCII)
