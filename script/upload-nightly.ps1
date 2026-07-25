[CmdletBinding()]
Param(
    [Parameter()][string]$Architecture
)

# Based on the template in: https://docs.digitalocean.com/reference/api/spaces-api/
$ErrorActionPreference = "Stop"
. "$PSScriptRoot\lib\blob-store.ps1"
. "$PSScriptRoot\lib\workspace.ps1"

ParseVelaWorkspace
Write-Host "Uploading nightly for target: $target"

$bucketName = "vela-nightly-host"
$releaseVersion = & "$PSScriptRoot\get-crate-version.ps1" vela
$version = "$releaseVersion+nightly.$env:GITHUB_RUN_NUMBER.$env:GITHUB_SHA"

$remoteServerFiles = Get-ChildItem -Path "target" -Filter "vela-remote-server-windows-*.zip" -Recurse -File -ErrorAction SilentlyContinue

foreach ($file in $remoteServerFiles) {
    UploadToBlobStore -BucketName $bucketName -FileToUpload $file.FullName -BlobStoreKey "nightly/$($file.Name)"
    UploadToBlobStore -BucketName $bucketName -FileToUpload $file.FullName -BlobStoreKey "$version/$($file.Name)"
    Remove-Item -Path $file.FullName -ErrorAction SilentlyContinue
}

UploadToBlobStore -BucketName $bucketName -FileToUpload "target/Vela-$Architecture.exe" -BlobStoreKey "nightly/Vela-$Architecture.exe"
UploadToBlobStore -BucketName $bucketName -FileToUpload "target/Vela-$Architecture.exe" -BlobStoreKey "$version/Vela-$Architecture.exe"

Remove-Item -Path "target/Vela-$Architecture.exe" -ErrorAction SilentlyContinue

$version | Out-File -FilePath "target/latest-sha" -NoNewline
UploadToBlobStore -BucketName $bucketName -FileToUpload "target/latest-sha" -BlobStoreKey "nightly/latest-sha-windows"
Remove-Item -Path "target/latest-sha" -ErrorAction SilentlyContinue
