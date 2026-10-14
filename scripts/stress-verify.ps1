param([Parameter(Mandatory)][int]$ProcessId)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
$window = [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition) | Where-Object { $_.Current.BoundingRectangle.Width -gt 100 } | Select-Object -First 1
if (!$window) { throw 'Stress window not found' }
$buttonCondition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty, '触发失败')
$button = $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $buttonCondition)
$button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
$deadline = [DateTime]::UtcNow.AddSeconds(5)
do {
    Start-Sleep -Milliseconds 100
    $elements = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    $names = @($elements | ForEach-Object { $_.Current.Name })
    if (($names -contains '前一步已执行') -and ($names | Where-Object { $_ -match 'steps\[1\].*65536' })) {
        if ($names -contains '不应执行') { throw 'Later step unexpectedly executed' }
        'PASS runtime error visible; preceding assignment preserved; subsequent assignment stopped'
        & (Join-Path $PSScriptRoot 'window-inspect.ps1') -ProcessId $ProcessId -EvidenceName 'long-error'
        return
    }
} while ([DateTime]::UtcNow -lt $deadline)
throw "Expected runtime error not found: $($names -join ' | ')"
