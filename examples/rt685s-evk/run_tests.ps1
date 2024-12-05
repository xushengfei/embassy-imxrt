param (
    [Parameter(Mandatory = $true)]
    [string]$Arg
)

$sshcommand = "ssh hwrunner@172.25.110.34 ./run_tests.sh " + $Arg
Write-Output $sshcommand

Restore-VMCheckpoint -VMName "hwrunner" -Name "fresh" -Confirm:$false 

# Pause for 30 seconds
Start-Sleep -Seconds 5

$output = Invoke-Expression $sshcommand

# Output the result
Write-Output $output

Restore-VMCheckpoint -VMName "hwrunner" -Name "fresh" -Confirm:$false 

# Determine the success or failure based on the output
if ($output -match "Some tests failed:") {
    exit 1
}
elseif ($output -match "All tests passed!") {
    exit 0
}

exit 1
