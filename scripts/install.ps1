# Install the latest OpenCanon CLI binary from GitHub Releases.
# Usage: irm https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.ps1 | iex
& {
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = if ($env:OPENCANON_REPO) { $env:OPENCANON_REPO } else { 'keyonkerr/OpenCanon' }
$Bin = 'opencanon'
$DefaultBinDir = Join-Path $env:LOCALAPPDATA 'OpenCanon\bin'
$BinDir = if ($env:OPENCANON_INSTALL_DIR) { $env:OPENCANON_INSTALL_DIR } else { $DefaultBinDir }
$Release = if ($env:OPENCANON_RELEASE) { $env:OPENCANON_RELEASE } else { 'latest' }

function Write-Step([string]$Message) {
    Write-Host "==> $Message"
}

function Normalize-Release([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value -eq 'latest') {
        return 'latest'
    }
    if ($Value.StartsWith('v')) {
        return $Value
    }
    return "v$Value"
}

function Get-Target {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' { throw 'Windows ARM64 binaries are not published yet. Use cargo install --git, or run under x64 emulation.' }
        default { throw "unsupported Windows architecture: $($env:PROCESSOR_ARCHITECTURE)" }
    }
}

function Get-AssetUrl([string]$Asset) {
    if ($Release -eq 'latest') {
        return "https://github.com/$Repo/releases/latest/download/$Asset"
    }
    return "https://github.com/$Repo/releases/download/$Release/$Asset"
}

$Release = Normalize-Release $Release
$Target = Get-Target
$Archive = "$Bin-$Target.zip"
$Checksum = "$Bin-$Target.sha256"

Write-Step 'OpenCanon CLI'
Write-Step "platform: $Target"
Write-Step "release: $Release"

$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("opencanon-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
    $ArchivePath = Join-Path $Tmp $Archive
    $ChecksumPath = Join-Path $Tmp $Checksum

    try {
        Invoke-WebRequest -Uri (Get-AssetUrl $Archive) -OutFile $ArchivePath -UseBasicParsing
    } catch {
        throw "could not download $Archive from https://github.com/$Repo/releases (has a tagged GitHub Release been published?)"
    }
    try {
        Invoke-WebRequest -Uri (Get-AssetUrl $Checksum) -OutFile $ChecksumPath -UseBasicParsing
    } catch {
        throw "could not download $Checksum (release is missing checksums)"
    }

    $Expected = ((Get-Content -Path $ChecksumPath -TotalCount 1) -split '\s+')[0].ToLowerInvariant()
    $Actual = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Expected -ne $Actual) {
        throw "SHA-256 mismatch for $Archive"
    }

    Expand-Archive -Path $ArchivePath -DestinationPath $Tmp -Force
    $Exe = Join-Path $Tmp "$Bin.exe"
    if (-not (Test-Path $Exe)) {
        throw "archive did not contain $Bin.exe"
    }

    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    Copy-Item -Path $Exe -Destination (Join-Path $BinDir "$Bin.exe") -Force

    Write-Step "installed $(Join-Path $BinDir "$Bin.exe")"

    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $UserPath) { $UserPath = '' }
    $Parts = $UserPath -split ';' | Where-Object { $_ -ne '' }
    if ($Parts -notcontains $BinDir) {
        $NewPath = if ($UserPath.Trim() -eq '') { $BinDir } else { "$UserPath;$BinDir" }
        [Environment]::SetEnvironmentVariable('Path', $NewPath, 'User')
        Write-Step "added $BinDir to the user PATH (open a new terminal)"
    } else {
        Write-Step "$BinDir is already on PATH"
    }
    $env:Path = "$BinDir;$env:Path"

    Write-Step 'run: opencanon help'
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
}
