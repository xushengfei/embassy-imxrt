$sshcommand = "ssh hwrunner@172.25.110.34 ./run_tests.sh"

$output = Invoke-Expression $sshcommand

# Determine the success or failure based on the output
if ($output -match "All tests passed!") {
    exit 0
}
else {
    exit 1
}