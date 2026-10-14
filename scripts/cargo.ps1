param([Parameter(ValueFromRemainingArguments = $true)][string[]]$CargoArguments)
$ErrorActionPreference = 'Stop'
$cargoCommand = Get-Command cargo.exe -CommandType Application -ErrorAction Stop
& $cargoCommand.Source @CargoArguments
exit $LASTEXITCODE
