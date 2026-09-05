# Run only on the disposable Windows Actions runner. No City or model calls.
$ErrorActionPreference = 'Stop'
if ($env:RUNNER_OS -ne 'Windows' -or $env:GITHUB_ACTIONS -ne 'true') {
    throw 'Use the disposable Windows workflow; this check must not use an employee profile.'
}
$root = Split-Path $PSScriptRoot -Parent
$artifacts = Join-Path $root 'dist-local-agent/windows-x64'
$receipt = Get-Content (Join-Path $artifacts 'package-receipt.json') -Raw | ConvertFrom-Json
if ($receipt.app_identifier -ne 'city.daarion.edge.local-agent-pilot' -or
    $receipt.installer -ne 'DAARION-Edge-Local-Pilot-windows-x64-setup.exe') { throw 'Unexpected artifact identity' }
$installer = Join-Path $artifacts $receipt.installer
if ((Get-FileHash $installer -Algorithm SHA256).Hash.ToLowerInvariant() -ne $receipt.sha256) { throw 'Installer checksum mismatch' }
$pilotProfile = Join-Path $env:APPDATA 'city.daarion.edge.local-agent-pilot'
$localProfile = Join-Path $env:LOCALAPPDATA 'city.daarion.edge.local-agent-pilot'
if ((Test-Path $pilotProfile) -or (Test-Path $localProfile)) { throw 'Existing pilot profile: refusing smoke test' }
$installDir = Join-Path $env:RUNNER_TEMP ('edge-pilot-install-' + [guid]::NewGuid().ToString('N'))
$process = $null
$installed = $false
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$scope = [System.Windows.Automation.TreeScope]::Descendants

function Find-Element([string] $name, [int] $timeout = 25) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt $timeout) {
        $process.Refresh()
        if ($process.HasExited) { throw 'Pilot exited before the UI check completed' }
        if ($process.MainWindowHandle -ne 0) {
            $window = [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
            $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty, $name)
            $element = $window.FindFirst($scope, $condition)
            if ($null -ne $element) { return $element }
        }
        Start-Sleep -Milliseconds 400
    }
    throw "Pilot UI element unavailable: $name"
}
function Click-Element([string] $name) {
    $element = Find-Element $name
    $pattern = $element.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $pattern.Invoke()
}
function Close-Pilot {
    if ($null -ne $process -and -not $process.HasExited) {
        if (-not $process.CloseMainWindow()) { throw 'Could not close pilot window' }
        if (-not $process.WaitForExit(15000)) { throw 'Pilot did not exit after closing its window' }
    }
}
try {
    # NSIS requires /D to be the final argument and without embedded quotes.
    $setup = Start-Process -FilePath $installer -ArgumentList "/S /D=$installDir" -PassThru
    if (-not $setup.WaitForExit(180000)) { $setup.Kill(); throw 'Setup timeout' }
    if ($setup.ExitCode -ne 0) { throw 'Setup failed' }
    $installed = $true
    $exe = Join-Path $installDir 'edge-local-agent.exe'
    if ((Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $receipt.executable_sha256) { throw 'Installed executable differs from built pilot' }
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--force-renderer-accessibility'
    $process = Start-Process -FilePath $exe -PassThru
    Click-Element 'Перевірити пристрій'
    $null = Find-Element 'Базову перевірку завершено'
    $agentNameInput = Find-Element 'Ім’я агента'
    $agentNameInput.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue('Windows installer check')
    Click-Element 'Створити власного агента'
    $null = Find-Element 'Windows installer check'
    Close-Pilot
    if (-not (Test-Path (Join-Path $pilotProfile 'pilot.sqlite3'))) { throw 'Pilot store missing after close' }
    $process = Start-Process -FilePath $exe -PassThru
    $null = Find-Element 'Windows installer check'
    Close-Pilot
    $uninstaller = Join-Path $installDir 'uninstall.exe'
    $remove = Start-Process -FilePath $uninstaller -ArgumentList '/S' -PassThru
    if (-not $remove.WaitForExit(60000)) { $remove.Kill(); throw 'Uninstaller timeout' }
    if ($remove.ExitCode -ne 0) { throw 'Uninstaller failed' }
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while ((Test-Path $exe) -and $timer.Elapsed.TotalSeconds -lt 15) { Start-Sleep -Milliseconds 400 }
    if (Test-Path $exe) { throw 'Executable remains after uninstall' }
    [ordered]@{
        installer_sha256 = $receipt.sha256
        install = 'PASS'; installed_binary_hash = 'PASS'; native_ui = 'PASS'
        basic_device_scan = 'PASS'; create_agent = 'PASS'; agent_survives_restart = 'PASS'; uninstall = 'PASS'
        folder_task_ui = 'NOT_RUN'; model_install = 'NOT_RUN'; city_connection = 'NOT_RUN'; employee_pc = 'NOT_RUN'
    } | ConvertTo-Json | Set-Content (Join-Path $artifacts 'windows-smoke.json') -Encoding utf8
    Write-Output 'Windows installer, native scan, agent creation, restart and uninstall: PASS'
} finally {
    if ($null -ne $process -and -not $process.HasExited) { Stop-Process -Id $process.Id }
    # The runner is disposable. Only remove profiles that this test created.
    if ($installed) {
        if (Test-Path $pilotProfile) { Remove-Item $pilotProfile -Recurse -Force }
        if (Test-Path $localProfile) { Remove-Item $localProfile -Recurse -Force }
    }
}
