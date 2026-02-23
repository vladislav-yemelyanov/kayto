$ErrorActionPreference = "Stop"

$Repo = "vladislav-yemelyanov/kayto"
$Binary = "kayto.exe"
$InstallDir = if ($env:KAYTO_INSTALL_DIR) { $env:KAYTO_INSTALL_DIR } else { "$env:USERPROFILE\\bin" }
$Version = $env:KAYTO_VERSION

if (-not $Version) {
  $Latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
  $Version = $Latest.tag_name
}

if (-not $Version) {
  throw "Failed to resolve release version. Set KAYTO_VERSION manually, e.g. v0.1.14"
}

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($Arch.ToString()) {
  "X64" { $TargetArch = "x86_64" }
  default { throw "Unsupported Windows architecture: $Arch. Supported: X64" }
}

$Target = "$TargetArch-pc-windows-gnu"
$Archive = "kayto-$Version-$Target.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("kayto-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
  $ZipPath = Join-Path $TmpDir $Archive
  Write-Host "Downloading kayto $Version for $Target..."
  Invoke-WebRequest -Uri $Url -OutFile $ZipPath

  Write-Host "Extracting..."
  Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

  $BinaryPath = Join-Path $TmpDir $Binary
  if (-not (Test-Path $BinaryPath)) {
    $NestedBinary = Join-Path (Join-Path $TmpDir "tmp") $Binary
    if (Test-Path $NestedBinary) {
      $BinaryPath = $NestedBinary
    } else {
      $Found = Get-ChildItem -Path $TmpDir -Filter $Binary -File -Recurse | Select-Object -First 1
      if (-not $Found) {
        throw "Binary not found in archive"
      }
      $BinaryPath = $Found.FullName
    }
  }

  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  $Destination = Join-Path $InstallDir $Binary
  Move-Item -Path $BinaryPath -Destination $Destination -Force

  $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if (-not $UserPath.Split(';').Contains($InstallDir)) {
    [Environment]::SetEnvironmentVariable("Path", ($UserPath.TrimEnd(';') + ";" + $InstallDir), "User")
    Write-Host "Added $InstallDir to user PATH"
  }

  Write-Host "Installed: $Destination"
  Write-Host "Open a new terminal and run: kayto --help"
}
finally {
  Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
