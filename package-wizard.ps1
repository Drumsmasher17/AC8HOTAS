param([switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$root = $PSScriptRoot
$wizard = Join-Path $root 'wizard'
if (-not $SkipBuild) {
    & (Join-Path $root 'build-wizard.cmd')
    if ($LASTEXITCODE -ne 0) { throw 'Wizard build failed' }
}
$noticePath = Join-Path $wizard 'notices.html'
if (-not (Test-Path -LiteralPath $noticePath)) { throw 'Run build-wizard.cmd first to generate and embed notices.' }
$notices = [IO.File]::ReadAllText($noticePath)
$lockHash = (Get-FileHash -LiteralPath (Join-Path $wizard 'Cargo.lock')).Hash.ToLowerInvariant()
if (-not $notices.Contains('<meta name="cargo-lock-sha256" content="'+$lockHash+'">')) {
    throw 'Embedded notices are stale. Run generate-wizard-notices.ps1 and rebuild.'
}
Push-Location $wizard
try {
    $tree = & cargo tree --target x86_64-pc-windows-msvc --edges normal --prefix none --format '{p}'
    if ($LASTEXITCODE -ne 0) { throw 'Cargo dependency tree failed' }
    $rustVersion = (& rustc --version).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect Rust toolchain' }
} finally { Pop-Location }
if (-not $notices.Contains('<meta name="rust-version" content="'+[Net.WebUtility]::HtmlEncode($rustVersion)+'">')) {
    throw 'Rust toolchain changed. Regenerate embedded notices and rebuild.'
}
foreach ($line in $tree) {
    if ($line -match '^(\S+) v(\S+)' -and $Matches[1] -ne 'ac8hotas-setup') {
        if (-not $notices.Contains('data-crate="'+$Matches[1]+'/'+$Matches[2]+'"')) { throw "Missing dependency notice: $line" }
    }
}
$exe = Join-Path $wizard 'target/release/AC8HOTAS-Setup.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw 'Run build-wizard.cmd first.' }
# SkipBuild must not accidentally package an executable from before the notices
# changed. The complete UTF-8 document is an embedded Rust string constant.
if (-not [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($exe)).Contains($notices)) {
    throw 'Executable does not contain the current notices. Rebuild the wizard.'
}
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$stage = Join-Path $dist ('wizard-stage-' + [guid]::NewGuid().ToString('N'))
$binary = Join-Path $stage 'binary'
$source = Join-Path $stage 'source/AC8HOTAS'
New-Item -ItemType Directory -Force -Path $binary,$source | Out-Null
# The portable download contains exactly one executable. In mod bundles it goes
# directly beside bindings.lua, enabling automatic installation detection.
Copy-Item -LiteralPath $exe -Destination $binary
$portable = Join-Path $dist 'AC8HOTAS-Setup.exe'
Copy-Item -LiteralPath $exe -Destination $portable -Force
$sourceFiles = @('LICENSE','THIRD_PARTY_NOTICES.md','.gitattributes','build-wizard.cmd','package-wizard.ps1',
    'generate-wizard-notices.ps1','wizard/Cargo.toml','wizard/Cargo.lock','wizard/README.md',
    'wizard/build.rs','wizard/.cargo/config.toml','mod/bindings.lua','mod/Scripts/actions.lua',
    'mod/Scripts/mapper.lua','tests/test_wizard_mapper.py','tests/test_wizard_package.py')
foreach ($directory in @('wizard/src','wizard/ui','wizard/assets','wizard/screenshots')) {
    $sourceFiles += Get-ChildItem -LiteralPath (Join-Path $root $directory) -File -Recurse |
        ForEach-Object { $_.FullName.Substring($root.Length + 1) }
}
foreach ($relative in $sourceFiles) {
    $destination = Join-Path $source $relative
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
    Copy-Item -LiteralPath (Join-Path $root $relative) -Destination $destination
}
$outputs = @($portable)
foreach ($kind in @('binary','source')) {
    $suffix = if ($kind -eq 'source') { '-source' } else { '' }
    $zip = Join-Path $dist ('AC8HOTAS-Setup-0.1.0' + $suffix + '.zip')
    $temp = $zip + '.' + [guid]::NewGuid().ToString('N') + '.tmp'
    [IO.Compression.ZipFile]::CreateFromDirectory((Join-Path $stage $kind), $temp, [IO.Compression.CompressionLevel]::Optimal, $false)
    Move-Item -LiteralPath $temp -Destination $zip -Force
    $outputs += $zip
}
$hashes = $outputs | ForEach-Object { (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash+'  '+(Split-Path -Leaf $_) }
[IO.File]::WriteAllLines((Join-Path $dist 'WIZARD-SHA256SUMS.txt'), $hashes, [Text.Encoding]::ASCII)
Write-Output $outputs
