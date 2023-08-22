param(
    [Parameter(Mandatory=$true)][string]$version
)

function Remove-OldInstallation {
    Write-Host "Removing old installation..."
    $null = Remove-Item -LiteralPath $INSTALL_DIR -Force -Recurse
}

function Install-TagTrack {
    param(
        [Parameter(Mandatory=$true)][string]$version
    )

    $null =  New-Item -ItemType Directory -Force -Path "${INSTALL_DIR}\bin"

    Write-Host "Downloading version ${version}..."
    try {
        $null =  Invoke-WebRequest -URI "${DOWNLOAD_URL}/${version}/${BINARY_NAME}" -OutFile "${INSTALL_DIR}\bin\tag-track.exe"
    } catch {
        Write-Host "Failed to install tag-track"
        exit 1
    }
}

$INSTALL_DIR = "${env:localappdata}\tag-track"
$BINARY_NAME = "tag-track_x86_64-pc-windows-msvc.exe"
$DOWNLOAD_URL = "https://github.com/dloez/tag-track/releases/download"

if (Test-Path -Path $INSTALL_DIR) {
    Remove-OldInstallation
}

Install-TagTrack -version $version
echo "Done! To start using tag-track add the directory '${INSTALL_DIR}\bin' to the system/user path environment variable"
