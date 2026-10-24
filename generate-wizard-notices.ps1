param([string]$NoticeArchive)
# Maintainer command: regenerate the single embedded notice document after
# changing Cargo.lock or the Rust toolchain. Not needed to run the wizard.
$ErrorActionPreference = 'Stop'
$wizard = Join-Path $PSScriptRoot 'wizard'
Push-Location $wizard
try {
    $metadata = ((& cargo metadata --format-version 1 --filter-platform x86_64-pc-windows-msvc) -join "`n") | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed' }
    $tree = & cargo tree --target x86_64-pc-windows-msvc --edges normal --prefix none --format '{p}'
    if ($LASTEXITCODE -ne 0) { throw 'Cargo dependency tree failed' }
    $sysroot = (& rustc --print sysroot).Trim()
    $rustVersion = (& rustc --version).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read Rust toolchain' }
} finally { Pop-Location }
function Escape-Html([string]$Text) { [Net.WebUtility]::HtmlEncode($Text) }
$html = [Text.StringBuilder]::new()
[void]$html.AppendLine('<!doctype html><html lang="en"><head><meta charset="utf-8">')
[void]$html.AppendLine('<meta name="cargo-lock-sha256" content="' + (Get-FileHash -LiteralPath (Join-Path $wizard 'Cargo.lock')).Hash.ToLowerInvariant() + '">')
[void]$html.AppendLine('<meta name="rust-version" content="' + (Escape-Html $rustVersion) + '">')
[void]$html.AppendLine('<title>AC8HOTAS Setup - Third-party notices</title><style>body{font:16px system-ui,sans-serif;max-width:1000px;margin:40px auto;padding:0 24px;background:#101925;color:#d4e5f5}a{color:#83bdfa}details{margin:16px 0;border:1px solid #3a526b;padding:14px;border-radius:6px}summary{cursor:pointer;font-weight:600}pre{white-space:pre-wrap;overflow-wrap:anywhere;font:13px ui-monospace,monospace;line-height:1.5}iframe{width:100%;height:600px;border:0;background:white}small{color:#a5b9cc}</style></head><body><h1>AC8HOTAS Setup</h1><p>Third-party notices embedded in this executable. Slint is used under Royalty-free 2.0; other listed alternatives are retained for reference.</p>')
function Add-Notice([string]$Name, [string]$Text) {
    [void]$html.AppendLine('<details><summary>' + (Escape-Html $Name) + '</summary>')
    if ($Name.EndsWith('.html')) {
        [void]$html.AppendLine('<iframe title="' + (Escape-Html $Name) + '" sandbox="" loading="lazy" srcdoc="' + (Escape-Html $Text) + '"></iframe>')
    } else {
        [void]$html.AppendLine('<pre>' + (Escape-Html $Text) + '</pre>')
    }
    [void]$html.AppendLine('</details>')
}
Add-Notice 'AC8HOTAS - CC0' ([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'LICENSE')))
$seen = [Collections.Generic.HashSet[string]]::new()
foreach ($line in $tree) {
    if ($line -notmatch '^(\S+) v(\S+)') { continue }
    $name = $Matches[1]; $version = $Matches[2]
    if ($name -eq 'ac8hotas-setup' -or -not $seen.Add("$name/$version")) { continue }
    $matching = @($metadata.packages | Where-Object { $_.name -eq $name -and $_.version -eq $version })
    if ($matching.Count -ne 1) { throw "Ambiguous dependency: $line" }
    $p = $matching[0]; $packageRoot = Split-Path -Parent $p.manifest_path
    $texts = @(Get-ChildItem -LiteralPath $packageRoot -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)([._-]|$)' })
    foreach ($directory in (Get-ChildItem -LiteralPath $packageRoot -Directory | Where-Object { $_.Name -match '^(licenses|licences)$' })) {
        $texts += Get-ChildItem -LiteralPath $directory.FullName -File -Recurse
    }
    if ($p.license_file) { $texts += Get-Item -LiteralPath (Join-Path $packageRoot $p.license_file) }
    $upstream = @{}
    if ($texts.Count -eq 0) {
        # Some crates omit their monorepo's license files. Fetch only from the
        # exact upstream revision recorded in the published Cargo package.
        if ($p.repository -notmatch '^https://github.com/([^/]+/[^/]+?)(?:\.git)?/?$') { throw "No license texts for $name" }
        $repo = $Matches[1]
        $vcs = Get-Content -LiteralPath (Join-Path $packageRoot '.cargo_vcs_info.json') -Raw | ConvertFrom-Json
        $sha = $vcs.git.sha1
        if ($sha -notmatch '^[a-fA-F0-9]{40}$') { throw "Missing pinned revision for $name" }
        $cache = Join-Path $wizard ("target/notice-sources/$name-$version")
        New-Item -ItemType Directory -Force -Path $cache | Out-Null
        if ($NoticeArchive) {
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            $archive = [IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $NoticeArchive).Path)
            try {
                $prefix = "AC8HOTAS-Setup/licenses/$name-$version/"
                foreach ($entry in $archive.Entries) {
                    if (-not $entry.FullName.StartsWith($prefix) -or $entry.Name -notmatch '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)([._-]|$)') { continue }
                    [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $cache $entry.Name), $true)
                }
            } finally { $archive.Dispose() }
        }
        # Reviewed repository-root notice files for crates that omit them.
        # Raw, revision-pinned downloads avoid GitHub API rate limits.
        $rootNotices = @{
                'AccessKit/accesskit' = @('LICENSE-APACHE','LICENSE-MIT','LICENSE.chromium')
                'nical/lyon' = @('LICENSE-APACHE','LICENSE-MIT')
                'DioxusLabs/taffy' = @('LICENSE.md')
                'mondeja/rspolib' = @('LICENSE')
                'DoumanAsh/clipboard-win' = @('LICENSE')
        }
        if (-not $rootNotices.ContainsKey($repo)) { throw "Review upstream notice filenames for $name ($repo)" }
        $texts = @()
        foreach ($fileName in $rootNotices[$repo]) {
            $destination = Join-Path $cache $fileName
            if (-not (Test-Path -LiteralPath $destination) -or (Get-Item -LiteralPath $destination).Length -eq 0) {
                $url = "https://raw.githubusercontent.com/$repo/$sha/$fileName"
                try { Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $destination }
                catch { throw "Cannot download required notice $url : $($_.Exception.Message)" }
            }
            $texts += Get-Item -LiteralPath $destination
        }
        if ($texts.Count -eq 0) { throw "No pinned upstream notices for $name" }
        foreach ($file in $texts) { $upstream[$file.Name] = "https://raw.githubusercontent.com/$repo/$sha/$($file.Name)" }
    }
    [void]$html.AppendLine('<section data-crate="' + (Escape-Html "$name/$version") + '"><h2>' + (Escape-Html "$name $version") + '</h2><p>License: ' + (Escape-Html $p.license) + '</p><p><a href="https://crates.io/crates/' + (Escape-Html "$name/$version") + '">Source package</a></p>')
    foreach ($file in ($texts | Sort-Object FullName -Unique)) {
        if ($upstream.ContainsKey($file.Name)) { [void]$html.AppendLine('<p><small>Original: ' + (Escape-Html $upstream[$file.Name]) + '</small></p>') }
        Add-Notice $file.Name ([IO.File]::ReadAllText($file.FullName))
    }
    [void]$html.AppendLine('</section>')
}
[void]$html.AppendLine('<h2>Rust standard library notices</h2><p>' + (Escape-Html $rustVersion) + '</p>')
$rustDocs = Join-Path $sysroot 'share/doc/rust'
foreach ($name in @('COPYRIGHT-library.html')) {
    Add-Notice $name ([IO.File]::ReadAllText((Join-Path $rustDocs $name)))
}
foreach ($file in (Get-ChildItem -LiteralPath (Join-Path $rustDocs 'licenses') -File | Sort-Object Name)) {
    Add-Notice $file.Name ([IO.File]::ReadAllText($file.FullName))
}
[void]$html.AppendLine('</body></html>')
$output = Join-Path $wizard 'notices.html'
$document = $html.ToString().Replace(([string][char]13 + [char]10), [string][char]10)
if (-not (Test-Path -LiteralPath $output) -or -not [string]::Equals([IO.File]::ReadAllText($output), $document, [StringComparison]::Ordinal)) {
    [IO.File]::WriteAllText($output, $document, [Text.UTF8Encoding]::new($false))
}
Write-Output "Embedded notices: $output ($($seen.Count) dependencies)"
