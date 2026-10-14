param([int]$ProcessId, [string]$EvidenceName = "window-$ProcessId")
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing
$condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
$windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)
$window = $windows | Where-Object { $_.Current.BoundingRectangle.Width -gt 100 } | Select-Object -First 1
if (!$window) { throw "No window for process $ProcessId" }
$rect = $window.Current.BoundingRectangle
"Window: $($window.Current.Name); bounds: $rect"
$elements = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
foreach ($element in $elements) {
    "$($element.Current.ControlType.ProgrammaticName) | $($element.Current.Name) | $($element.Current.BoundingRectangle)"
}
$bitmap = [System.Drawing.Bitmap]::new([int]$rect.Width, [int]$rect.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen([int]$rect.X, [int]$rect.Y, 0, 0, $bitmap.Size)
$output = Join-Path (Split-Path -Parent $PSScriptRoot) 'evidence'
if (!(Test-Path -LiteralPath $output)) { [System.IO.Directory]::CreateDirectory($output) | Out-Null }
$bitmap.Save((Join-Path $output "$EvidenceName.png"))
$graphics.Dispose()
$bitmap.Dispose()
