
function ParseVelaWorkspace {
    $metadata = cargo metadata --no-deps --offline | ConvertFrom-Json
    $env:VELA_WORKSPACE = $metadata.workspace_root
    $env:RELEASE_VERSION = $metadata.packages | Where-Object { $_.name -eq "vela" } | Select-Object -ExpandProperty version
}
