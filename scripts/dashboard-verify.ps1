param([Parameter(Mandatory)][int]$ProcessId, [string]$EvidenceName = 'dashboard-light', [switch]$SmokeOnly)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class DashboardInput {
 [StructLayout(LayoutKind.Sequential)] public struct Point {public int X;public int Y;}
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point p);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int c);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,IntPtr p);
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a,uint b,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
 [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h,int x,int y,int w,int height,bool repaint);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h,IntPtr after,int x,int y,int w,int height,uint flags);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint x,uint y,int data,UIntPtr extra);
 [DllImport("user32.dll")] public static extern void keybd_event(byte vk,byte scan,uint flags,UIntPtr extra);
 public static void Focus(IntPtr h) {
  uint current=GetCurrentThreadId(), foreground=GetWindowThreadProcessId(GetForegroundWindow(),IntPtr.Zero);
  if(current!=foreground) AttachThreadInput(current,foreground,true);
  try {ShowWindow(h,9);BringWindowToTop(h);SetForegroundWindow(h);}
  finally {if(current!=foreground) AttachThreadInput(current,foreground,false);}
 }
}
'@
[DashboardInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty,$ProcessId)
$window = [System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children,$condition)
if (!$window) { throw 'Dashboard window not found' }
$handle = [IntPtr]$window.Current.NativeWindowHandle
[DashboardInput]::Focus($handle)
[DashboardInput]::MoveWindow($handle,40,40,1780,1260,$true) | Out-Null
Start-Sleep -Milliseconds 400
if ([DashboardInput]::GetForegroundWindow() -ne $handle) {
 [DashboardInput]::SetWindowPos($handle,[IntPtr](-1),0,0,0,0,0x43) | Out-Null
 [DashboardInput]::SetCursorPos(300,60) | Out-Null
 [DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
 [DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 200
 [DashboardInput]::SetWindowPos($handle,[IntPtr](-2),0,0,0,0,0x43) | Out-Null
}
if ([DashboardInput]::GetForegroundWindow() -ne $handle) { throw 'Dashboard did not receive foreground focus' }
[DashboardInput]::SetCursorPos(100,250) | Out-Null
[DashboardInput]::mouse_event(0x800,0,0,12000,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 400
function Find-Name([string]$name) {
 $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition) | Where-Object {$_.Current.Name -eq $name} | Select-Object -First 1
}
function Assert-Name([string]$name) {
 for ($i=0;$i -lt 30;$i++) {if (Find-Name $name) {"PASS visible text: $name";return};Start-Sleep -Milliseconds 100}
 throw "Missing visible text: $name"
}
function Click-Element($element) {
 if (!$element) {throw 'Cannot click missing control'}
 $r = $element.Current.BoundingRectangle
 [DashboardInput]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 [DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
 [DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 200
}
function Capture([string]$suffix) {
 [DashboardInput]::Focus($handle)
 Start-Sleep -Milliseconds 200
 $r = $window.Current.BoundingRectangle
 $image = [System.Drawing.Bitmap]::new([int]$r.Width,[int]$r.Height)
 $graphics = [System.Drawing.Graphics]::FromImage($image)
 $graphics.CopyFromScreen([int]$r.X,[int]$r.Y,0,0,$image.Size)
 $path = Join-Path (Split-Path -Parent $PSScriptRoot) "evidence/$EvidenceName-$suffix.png"
 $image.Save($path);$graphics.Dispose();$image.Dispose();"SCREENSHOT $path"
}
Assert-Name '运行状态看板'
Assert-Name 'CPU 使用率'
Assert-Name '42.0%'
Capture 'initial'
$chart = Find-Name '图表 trend-chart，系列 CPU、内存，12 个数据点'
if (!$chart) {throw 'Chart accessibility summary not found'}
$plot=$chart.Current.BoundingRectangle
[DashboardInput]::Focus($handle)
[DashboardInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
[DashboardInput]::SetCursorPos([int]($plot.X+$plot.Width*0.6),[int]($plot.Y+$plot.Height*0.34)) | Out-Null
[DashboardInput]::mouse_event(1,0,0,0,[UIntPtr]::Zero)
for ($hoverAttempt=0;$hoverAttempt -lt 20;$hoverAttempt++) {
 [DashboardInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
 [DashboardInput]::SetCursorPos([int]($plot.X+$plot.Width*0.6),[int]($plot.Y+$plot.Height*0.34)) | Out-Null
 Start-Sleep -Milliseconds 60
 if (Find-Name "CPU`nX: 3`nY: 0.66") {break}
}
"HOVER plot bounds: $plot; target $([int]($plot.X+$plot.Width*0.6)),$([int]($plot.Y+$plot.Height*0.34))"
$cursor=[DashboardInput+Point]::new();[DashboardInput]::GetCursorPos([ref]$cursor) | Out-Null;"HOVER actual cursor $($cursor.X),$($cursor.Y); foreground $([DashboardInput]::GetForegroundWindow()) target $handle"
try {Assert-Name "CPU`nX: 3`nY: 0.66"} catch {Capture 'hover-failed';throw}
Capture 'hover'
[DashboardInput]::mouse_event(0x800,0,0,120,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 300
Capture 'zoom'
[DashboardInput]::SetCursorPos([int]($plot.X+$plot.Width*0.5),[int]($plot.Y+$plot.Height*0.5)) | Out-Null
[DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
for ($move=1;$move -le 6;$move++) {
 [DashboardInput]::SetCursorPos([int]($plot.X+$plot.Width*0.5+10*$move),[int]($plot.Y+$plot.Height*0.5)) | Out-Null
 Start-Sleep -Milliseconds 30
}
[DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 200
Capture 'pan'
[DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
[DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 50
[DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
[DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 250
Click-Element (Find-Name 'CPU')
Capture 'legend'
Click-Element (Find-Name 'CPU')
if (!$SmokeOnly) {
 Click-Element (Find-Name '更新指标')
 Assert-Name '指标已更新 · 确定性本地动作，不访问网络'
 Assert-Name '57.0%'
 Capture 'updated'
 # Scroll the page outside chart/table regions, then verify the fixed header and sort.
 $r=$window.Current.BoundingRectangle
 [DashboardInput]::SetCursorPos([int]($r.Right-35),[int]($r.Top+200)) | Out-Null
 [DashboardInput]::mouse_event(0x800,0,0,-1200,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 400
 Click-Element (Find-Name '负载')
 Assert-Name '负载 ▲'
 Capture 'sorted'
 $header=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition) | Where-Object {$_.Current.Name -eq '节点名称' -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button} | Select-Object -First 1
 $before=$header.Current.BoundingRectangle
 [DashboardInput]::SetCursorPos([int]$before.Right,[int]($before.Y+$before.Height/2)) | Out-Null
 [DashboardInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
 for ($move=1;$move -le 8;$move++) {
  [DashboardInput]::SetCursorPos([int]($before.Right+10*$move),[int]($before.Y+$before.Height/2)) | Out-Null
  Start-Sleep -Milliseconds 30
 }
 [DashboardInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 200
 $after=$header.Current.BoundingRectangle
 if ([Math]::Abs($after.Width-$before.Width) -le 30) {throw "Column drag did not resize: before=$before after=$after"}
 "PASS column drag width $($before.Width) -> $($after.Width)"
 Capture 'column-resized'
 # Select the first actual row, then move selection using the keyboard.
 $c=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty,[System.Windows.Automation.ControlType]::Button)
 $row=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,$c) | Where-Object {$_.Current.Name.StartsWith('节点名称:') -and $_.Current.BoundingRectangle.Height -gt 20} | Select-Object -First 1
 Click-Element $row
 [DashboardInput]::keybd_event(0x28,0,0,[UIntPtr]::Zero)
 [DashboardInput]::keybd_event(0x28,0,2,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 200
 Capture 'selected'
 $r=$row.Current.BoundingRectangle
 [DashboardInput]::SetCursorPos([int]($r.X+100),[int]($r.Y+15)) | Out-Null
 [DashboardInput]::mouse_event(0x800,0,0,-480,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 300
 Assert-Name '负载 ▲'
 Capture 'table-scrolled'
 [DashboardInput]::MoveWindow($handle,40,40,1150,850,$true) | Out-Null
 Start-Sleep -Milliseconds 300
 Assert-Name '节点名称'
 Capture 'resized'
 [DashboardInput]::keybd_event(0x11,0,0,[UIntPtr]::Zero)
 [DashboardInput]::keybd_event(0xBD,0,0,[UIntPtr]::Zero)
 [DashboardInput]::keybd_event(0xBD,0,2,[UIntPtr]::Zero)
 [DashboardInput]::keybd_event(0x11,0,2,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 300
 Capture 'ui-scaled'
 "PASS update, page scroll, sort click, row click, keyboard and resize inputs delivered"
}
