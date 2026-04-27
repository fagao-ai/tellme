#!/usr/bin/env pwsh

$REPO = "fagao-ai/tellme"

$ErrorActionPreference = "Stop"

# 获取最新版本
$latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
$version = $latestRelease.tag_name
Write-Host "Installing tellme $version for Windows..."

# 下载
$downloadUrl = "https://github.com/$REPO/releases/download/$version/tellme-windows-amd64"
$exePath = "$env:TEMP\tellme.exe"

Write-Host "Downloading from $downloadUrl..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath

# 安装
$binDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

Write-Host "Installing to $binDir..."
Move-Item -Path $exePath -Destination "$binDir\tellme.exe" -Force

Write-Host "✓ tellme has been installed to $binDir\tellme.exe"
Write-Host ""
Write-Host "Add to PATH if needed:"
Write-Host "  [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$binDir', 'User')"
Write-Host ""
Write-Host "Then restart your terminal and verify installation:"
Write-Host "  tellme --version"
