# clipmap — Windows Task Scheduler setup
# Run once as Administrator in PowerShell

$BinaryPath = "C:\Program Files\clipmap\clipmap.exe"
$TaskName   = "clipmap"

# Remove existing task if present
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue

$Action  = New-ScheduledTaskAction -Execute $BinaryPath
$Trigger = New-ScheduledTaskTrigger -AtLogOn
$Settings = New-ScheduledTaskSettingsSet `
    -ExecutionTimeLimit 0 `
    -RestartCount 3 `
    -RestartInterval (New-TimeSpan -Minutes 1)

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action   $Action `
    -Trigger  $Trigger `
    -Settings $Settings `
    -RunLevel Highest `
    -Force

Write-Host "clipmap registered. It will start at next login."
Write-Host "To start now: Start-ScheduledTask -TaskName clipmap"
