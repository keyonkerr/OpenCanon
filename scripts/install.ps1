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
$InheritedPath = $env:Path

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

function Test-PathContains([string]$PathValue, [string]$Entry) {
    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return $false
    }
    $needle = $Entry.TrimEnd('\')
    foreach ($segment in $PathValue.Split(';', [System.StringSplitOptions]::RemoveEmptyEntries)) {
        if ($segment.TrimEnd('\') -ieq $needle) {
            return $true
        }
    }
    return $false
}

function Prepend-PathEntry([string]$PathValue, [string]$Entry) {
    $needle = $Entry.TrimEnd('\')
    $segments = @($Entry)
    if (-not [string]::IsNullOrWhiteSpace($PathValue)) {
        $segments += $PathValue.Split(';', [System.StringSplitOptions]::RemoveEmptyEntries) |
            Where-Object { $_.TrimEnd('\') -ine $needle }
    }
    return ($segments -join ';')
}

function Notify-EnvironmentChange {
    try {
        if (-not ('OpenCanon.NativeMethods' -as [type])) {
            Add-Type -Namespace OpenCanon -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@
        }
        $result = [UIntPtr]::Zero
        [void][OpenCanon.NativeMethods]::SendMessageTimeout(
            [IntPtr]0xffff,
            0x1A,
            [UIntPtr]::Zero,
            'Environment',
            2,
            5000,
            [ref]$result
        )
    } catch {
        # User PATH is already written; explorer will pick it up on the next login if notify fails.
    }
}

function Refresh-SessionPath {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ([string]::IsNullOrWhiteSpace($machine)) {
        $env:Path = $user
    } elseif ([string]::IsNullOrWhiteSpace($user)) {
        $env:Path = $machine
    } else {
        $env:Path = "$machine;$user"
    }
}

function Install-VisibleShim([string]$ExePath) {
    $exeDir = [System.IO.Path]::GetDirectoryName($ExePath)
    $candidates = @(
        (Join-Path $env:USERPROFILE '.local\bin'),
        (Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps')
    )
    foreach ($dir in $candidates) {
        if (-not (Test-PathContains -PathValue $InheritedPath -Entry $dir)) {
            continue
        }
        if ($dir.TrimEnd('\') -ieq $exeDir.TrimEnd('\')) {
            continue
        }
        if (-not (Test-Path -LiteralPath $dir)) {
            continue
        }
        $shim = Join-Path $dir "$Bin.exe"
        Copy-Item -LiteralPath $ExePath -Destination $shim -Force
        Write-Step "also installed $shim (already on this terminal PATH)"
        return
    }
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
    $Installed = Join-Path $BinDir "$Bin.exe"
    Copy-Item -Path $Exe -Destination $Installed -Force

    Write-Step "installed $Installed"

    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $newUserPath = Prepend-PathEntry -PathValue $UserPath -Entry $BinDir
    if ($newUserPath -cne $UserPath) {
        [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
        Notify-EnvironmentChange
        Write-Step "PATH updated for future PowerShell sessions."
    } elseif (Test-PathContains -PathValue $InheritedPath -Entry $BinDir) {
        Write-Step "$BinDir is already on PATH."
    } else {
        Write-Step "PATH is already configured for future PowerShell sessions."
    }

    Refresh-SessionPath
    if (-not (Test-PathContains -PathValue $env:Path -Entry $BinDir)) {
        $env:Path = Prepend-PathEntry -PathValue $env:Path -Entry $BinDir
    }

    Install-VisibleShim -ExePath $Installed

    Write-Step "Current PowerShell session: $Bin"
    Write-Step "Future PowerShell windows: open a new PowerShell window and run: $Bin"
    Write-Step "run: opencanon help"
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
}
