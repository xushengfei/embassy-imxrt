# Get all files in the current directory
$files = Get-ChildItem -File -Filter *.rs

$testCounter = 1
$failed = $false
[string[]]$failedArr = @()

# Loop through each file and add its name without the .rs extension to the array
foreach ($file in $files) {
    $strippedName = $file.Name -replace '\.rs$', ''
    Write-Output "Executing test ${testCounter}: $strippedName"
    $testCounter++
    
    cargo run --bin $strippedName | test-parser.exe -s TEST-SUCCESS -f TEST-FAIL

    if ($LASTEXITCODE -eq 1) {
        Write-Output "The last command failed with exit code 1."
        $failedArr += $strippedName
        $failed = $true
    }
}

Write-Output "`r`n========END OF TESTS SUMMARY========"
if ($failed) {
    Write-Output "Some tests failed: "
    foreach($failure in $failedArr) {
        Write-Output "${failure}"
    }
    Exit 1
} else {
    Write-Output "All tests passed!"
    Exit 0
}

