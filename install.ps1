# Rune VCS PowerShell Installation Script
# Usage: iwr -useb https://raw.githubusercontent.com/CaptainOtto/rune-vcs/main/install.ps1 | iex

param(
    [switch]$DryRun,
    [string]$InstallDir = "",
    [switch]$Force
)

# Configuration
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Colors and styling (limited in PowerShell)
function Write-ColorText {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Write-Header {
    Write-Host ""
    Write-ColorText "🔮 Rune VCS Installation Script" "Blue"
    Write-ColorText "================================" "Blue"
    Write-Host ""
}

function Write-Success {
    param([string]$Message)
    Write-ColorText "✓ $Message" "Green"
}

function Write-Info {
    param([string]$Message)
    Write-ColorText "ℹ $Message" "Cyan"
}

function Write-Warning {
    param([string]$Message)
    Write-ColorText "⚠ $Message" "Yellow"
}

function Write-Error {
    param([string]$Message)
    Write-ColorText "✗ $Message" "Red"
}

function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { 
            Write-Error "Unsupported architecture: $arch"
            exit 1
        }
    }
}

function Get-LatestVersion {
    # For now, return current version
    # In production, this would query GitHub API
    return "v0.0.1"
}

function Test-AdminRights {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-InstallDirectory {
    param([string]$PreferredDir)
    
    if ($PreferredDir -ne "") {
        return $PreferredDir
    }
    
    # Try various install locations
    $candidates = @(
        "$env:LOCALAPPDATA\Programs\Rune",
        "$env:USERPROFILE\.local\bin",
        "$env:USERPROFILE\bin"
    )
    
    # If running as admin, prefer system-wide location
    if (Test-AdminRights) {
        $candidates = @("$env:ProgramFiles\Rune") + $candidates
    }
    
    foreach ($dir in $candidates) {
        if (Test-Path $dir -PathType Container) {
            return $dir
        }
    }
    
    # Default to user programs directory
    return "$env:LOCALAPPDATA\Programs\Rune"
}

function Install-Rune {
    param(
        [string]$Version,
        [string]$Architecture,
        [string]$InstallPath
    )
    
    $target = "$Architecture-pc-windows-msvc"
    $downloadUrl = "https://github.com/CaptainOtto/rune-vcs/releases/download/$Version/rune-$Version-$target.zip"
    
    Write-Info "Downloading Rune $Version for $target..."
    
    $tempDir = [System.IO.Path]::GetTempPath()
    $archivePath = Join-Path $tempDir "rune.zip"
    $extractPath = Join-Path $tempDir "rune-extract"
    
    try {
        # Download the archive
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
        Write-Success "Downloaded successfully"
    }
    catch {
        Write-Error "Failed to download Rune from $downloadUrl"
        Write-Info "Note: This is a demo script. Actual releases may not be available yet."
        Write-Info "For now, please build from source:"
        Write-Info "  1. git clone https://github.com/CaptainOtto/rune-vcs.git"
        Write-Info "  2. cd rune-vcs"
        Write-Info "  3. cargo build --release"
        Write-Info "  4. Copy target\release\rune-cli.exe to your desired location"
        exit 1
    }
    
    # Create install directory
    if (-not (Test-Path $InstallPath)) {
        Write-Info "Creating install directory: $InstallPath"
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    }
    
    # Extract and install
    Write-Info "Extracting archive..."
    Expand-Archive -Path $archivePath -DestinationPath $extractPath -Force
    
    $binaryPath = Join-Path $extractPath "rune.exe"
    $targetPath = Join-Path $InstallPath "rune.exe"
    
    Copy-Item $binaryPath $targetPath -Force
    Write-Success "Rune installed to $targetPath"
    
    # Cleanup
    Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
    Remove-Item $extractPath -Recurse -Force -ErrorAction SilentlyContinue
    
    return $InstallPath
}

function Update-Path {
    param([string]$Directory)
    
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    
    if ($currentPath -notlike "*$Directory*") {
        Write-Info "Adding $Directory to PATH..."
        $newPath = "$Directory;$currentPath"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Success "PATH updated (restart terminal to take effect)"
        
        # Update current session PATH
        $env:PATH = "$Directory;$env:PATH"
    } else {
        Write-Info "Directory already in PATH"
    }
}

function Install-Completions {
    param([string]$BinaryPath)
    
    Write-Info "Setting up PowerShell completions..."
    
    try {
        $completionScript = & $BinaryPath completions powershell 2>$null
        if ($completionScript) {
            $profileDir = Split-Path $PROFILE -Parent
            if (-not (Test-Path $profileDir)) {
                New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
            }
            
            $completionFile = Join-Path $profileDir "rune-completion.ps1"
            $completionScript | Out-File -FilePath $completionFile -Encoding UTF8
            
            # Add to profile if not already there
            if (Test-Path $PROFILE) {
                $profileContent = Get-Content $PROFILE -Raw
                if ($profileContent -notlike "*rune-completion.ps1*") {
                    ". `"$completionFile`"" | Add-Content $PROFILE
                }
            } else {
                ". `"$completionFile`"" | Out-File $PROFILE -Encoding UTF8
            }
            
            Write-Success "PowerShell completions installed"
        }
    }
    catch {
        Write-Warning "Could not install PowerShell completions"
    }
}

function Test-Installation {
    param([string]$BinaryPath)
    
    Write-Info "Verifying installation..."
    
    try {
        $version = & $BinaryPath version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Rune is installed and accessible"
            Write-Info "Version check passed"
            
            # Run doctor command
            Write-Info "Running installation verification..."
            & $BinaryPath doctor >$null 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Installation verification passed"
            } else {
                Write-Warning "Installation verification had some issues"
                Write-Info "Run 'rune doctor' for detailed diagnostics"
            }
        } else {
            Write-Error "Rune installation failed - unable to run binary"
            exit 1
        }
    }
    catch {
        Write-Error "Rune installation failed - $($_.Exception.Message)"
        exit 1
    }
}

function Main {
    Write-Header
    
    if ($DryRun) {
        Write-Info "DRY RUN MODE - No actual installation will be performed"
        Write-Host ""
    }
    
    Write-Info "Detecting system architecture..."
    $arch = Get-Architecture
    Write-Success "Architecture detected: $arch"
    
    Write-Info "Getting latest version..."
    $version = Get-LatestVersion
    Write-Success "Latest version: $version"
    
    $installDir = Get-InstallDirectory -PreferredDir $InstallDir
    Write-Info "Install directory: $installDir"
    
    # Check if already installed
    $existingBinary = Join-Path $installDir "rune.exe"
    if ((Test-Path $existingBinary) -and (-not $Force)) {
        Write-Warning "Rune is already installed at $existingBinary"
        $response = Read-Host "Do you want to reinstall? (y/N)"
        if ($response -notmatch "^[Yy]") {
            Write-Info "Installation cancelled"
            exit 0
        }
    }
    
    if ($DryRun) {
        Write-Info "Would download and install Rune $version for $arch to $installDir"
        Write-Info "Would update PATH if necessary"
        Write-Info "Would install PowerShell completions"
        Write-Success "Dry run completed"
        exit 0
    }
    
    # Demo warning
    Write-Warning "This is a demonstration installation script"
    Write-Info "For the current version, please build from source:"
    Write-Host ""
    Write-Info "Building from source:"
    Write-Info "  1. git clone https://github.com/CaptainOtto/rune-vcs.git"
    Write-Info "  2. cd rune-vcs"
    Write-Info "  3. cargo build --release"
    Write-Info "  4. Copy target\release\rune-cli.exe to your desired location"
    Write-Host ""
    Write-Info "Alternative installation methods:"
    Write-Info "  • Cargo: cargo install --git https://github.com/CaptainOtto/rune-vcs rune-cli"
    Write-Host ""
    
    $response = Read-Host "Continue with demo installation? (y/N)"
    if ($response -match "^[Yy]") {
        Write-Info "Proceeding with demo installation..."
        # This would normally download and install
        # $actualInstallDir = Install-Rune -Version $version -Architecture $arch -InstallPath $installDir
        Write-Info "Demo installation completed"
        Write-Info "Please follow the build-from-source instructions above"
    } else {
        Write-Info "Installation cancelled"
    }
    
    Write-Host ""
    Write-Success "Thank you for trying Rune VCS!"
    Write-Info "Visit https://github.com/CaptainOtto/rune-vcs for documentation"
}

# Run the installation
Main
