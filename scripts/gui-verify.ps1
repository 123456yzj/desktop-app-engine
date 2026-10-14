param([Parameter(Mandatory)][int]$ProcessId, [string]$EvidenceName = "window-$ProcessId")
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeInput {
    [StructLayout(LayoutKind.Explicit, Size=40)]
    public struct KeyInput {
        [FieldOffset(0)] public uint type;
        [FieldOffset(8)] public ushort vk;
        [FieldOffset(10)] public ushort scan;
        [FieldOffset(12)] public uint flags;
    }
    [DllImport("user32.dll", SetLastError=true)] public static extern uint SendInput(uint count, KeyInput[] inputs, int size);
    public static void Key(ushort vk, ushort scan, uint flags) {
        var input = new KeyInput { type=1, vk=vk, scan=scan, flags=flags };
        if (SendInput(1,new[]{input},40)!=1) throw new Exception("SendInput failed: "+Marshal.GetLastWin32Error());
    }
    public static void TypeText(string text) {
        Key(0x11,0,0); Key(0x41,0,0); Key(0x41,0,2); Key(0x11,0,2);
        foreach(char c in text) { Key(0,c,4); Key(0,c,6); }
    }
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int command);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, IntPtr pid);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint first, uint second, bool attach);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hwnd);
    public static void Focus(IntPtr hwnd) {
        uint current=GetCurrentThreadId(), foreground=GetWindowThreadProcessId(GetForegroundWindow(),IntPtr.Zero);
        if(current!=foreground) AttachThreadInput(current,foreground,true);
        try { ShowWindow(hwnd,9); BringWindowToTop(hwnd); SetForegroundWindow(hwnd); }
        finally { if(current!=foreground) AttachThreadInput(current,foreground,false); }
    }
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hwnd, int x, int y, int w, int h, bool repaint);
}
'@
[NativeInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
$windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)
$window = $windows | Where-Object { $_.Current.BoundingRectangle.Width -gt 100 } | Select-Object -First 1
if (!$window) { throw 'Window not found' }
[NativeInput]::ShowWindow([IntPtr]$window.Current.NativeWindowHandle, 9) | Out-Null
Start-Sleep -Milliseconds 200
[NativeInput]::SetForegroundWindow([IntPtr]$window.Current.NativeWindowHandle) | Out-Null
function Find-Control($type) {
    $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $type)
    $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
}
function Assert-Text([string]$expected) {
    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    do {
        $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty, $expected)
        if ($window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)) { "PASS visible text: $expected"; return }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Expected visible text not found: $expected"
}
function Click($element) {
    [NativeInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
    $rect = $element.Current.BoundingRectangle
    [NativeInput]::SetCursorPos([int]($rect.X + $rect.Width / 2), [int]($rect.Y + $rect.Height / 2)) | Out-Null
    [NativeInput]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
    [NativeInput]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 150
}
foreach ($name in @('小明','小红')) {
    [NativeInput]::Focus([IntPtr]$window.Current.NativeWindowHandle)
    Start-Sleep -Milliseconds 100
    $edit = Find-Control ([System.Windows.Automation.ControlType]::Edit)
    Click $edit
    if ([NativeInput]::GetForegroundWindow() -ne [IntPtr]$window.Current.NativeWindowHandle) { throw "Target $($window.Current.NativeWindowHandle) did not receive foreground focus; actual $([NativeInput]::GetForegroundWindow()); edit bounds $($edit.Current.BoundingRectangle)" }
    [NativeInput]::TypeText($name)
    Assert-Text $name
    $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty, '确认')
    Click ($window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition))
    Assert-Text "你好，$name"
}
[NativeInput]::MoveWindow([IntPtr]$window.Current.NativeWindowHandle, 250,250,480,360,$true) | Out-Null
Start-Sleep -Milliseconds 300
Assert-Text '你好，小红'
& (Join-Path $PSScriptRoot 'window-inspect.ps1') -ProcessId $ProcessId -EvidenceName $EvidenceName
"PASS input, two clicks, binding, resize, Chinese accessibility text"
