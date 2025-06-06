[Console]::OutputEncoding = [Text.Encoding]::UTF8

$originalDir = Get-Location
Set-Location $PSScriptRoot

fontforge -script ./boldify.pe

Set-Location $originalDir
