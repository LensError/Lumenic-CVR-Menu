param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"
$tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }

Push-Location "$PSScriptRoot\native-ui"
try {
    cargo test
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }
} finally {
    Pop-Location
}

dotnet build "$PSScriptRoot\LumenicMenu.csproj" -c Release
if ($LASTEXITCODE -ne 0) { throw "dotnet build failed" }

$dll = "$PSScriptRoot\bin\Release\netstandard2.1\LumenicMenu.dll"
if (-not (Test-Path $dll)) { throw "Missing build output: $dll" }

git tag -f $tag
if ($LASTEXITCODE -ne 0) { throw "git tag failed" }

git push origin $tag --force
if ($LASTEXITCODE -ne 0) { throw "git push tag failed" }

if ($Notes) {
    gh release create $tag $dll --title $tag --notes $Notes
} else {
    gh release create $tag $dll --title $tag --generate-notes
}
if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }

Write-Host "Released $tag ($dll)"
