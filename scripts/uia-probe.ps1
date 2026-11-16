# What a screen reader (NVDA, Narrator) gets on Windows: the UI Automation
# tree with role and name, then after each key where the focus is. Must run
# in the logged-in session (a scheduled task with /IT), not over SSH: it
# sends real keys to the foreground window.
#   uia-probe.ps1 -Title "Menu probe" -Exe app.exe -Keys "{TAB} +{F10}*1 {DOWN}*2" -Out uia.log
param([string]$Title = "Menu probe", [string]$Exe = "", [string]$Out = "uia.log",
      [string]$Keys = "", [int]$Depth = 12, [string]$Arguments = "")

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
# Native Win32 menus (Slint's context menu on Windows) are read through MSAA,
# as NVDA does; the .NET UI Automation client here shows them as empty panes.
# Compiled next to this script: some setups block running DLLs from %TEMP%.
$msaaDll = Join-Path $PSScriptRoot "msaa-probe.dll"
# Loaded from memory, so a virus scanner looking at the file doesn't block it.
if (Test-Path $msaaDll) { [void][Reflection.Assembly]::Load([IO.File]::ReadAllBytes($msaaDll)) } else {
Add-Type -OutputAssembly $msaaDll -ReferencedAssemblies Accessibility @"
using System;
using System.Text;
using System.Runtime.InteropServices;
using Accessibility;
public static class Msaa {
    [DllImport("oleacc.dll")]
    static extern int AccessibleObjectFromWindow(IntPtr hwnd, int id, ref Guid iid, [MarshalAs(UnmanagedType.Interface)] out IAccessible ppv);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string cls, string name);
    [DllImport("oleacc.dll", CharSet = CharSet.Unicode)]
    static extern uint GetRoleText(uint role, StringBuilder sb, uint len);
    static string Role(object r) {
        if (r is int) { var sb = new StringBuilder(64); GetRoleText((uint)(int)r, sb, 64); return sb.ToString(); }
        return "" + r;
    }
    // States: 0x1 unavailable, 0x4 focused, 0x10 checked, 0x80 hot-tracked.
    public static string Menu() {
        IntPtr h = FindWindow("#32768", null);
        if (h == IntPtr.Zero) return null;
        var iid = new Guid("618736E0-3C3D-11CF-810C-00AA00389B71");
        IAccessible acc;
        if (AccessibleObjectFromWindow(h, -4, ref iid, out acc) != 0 || acc == null) return "  native menu: no IAccessible";
        var sb = new StringBuilder("  native menu: " + Role(acc.get_accRole(0)) + ", focus on item " + acc.accFocus);
        for (int i = 1; i <= acc.accChildCount; i++) {
            sb.Append("\n    " + Role(acc.get_accRole(i)) + " '" + acc.get_accName(i) + "' state=0x" + ((int)acc.get_accState(i)).ToString("x"));
        }
        return sb.ToString();
    }
}
"@
[void][Reflection.Assembly]::Load([IO.File]::ReadAllBytes($msaaDll))
}
$A = [System.Windows.Automation.AutomationElement]
$log = New-Object System.Collections.Generic.List[string]

function Line($e, $level) {
    $c = $e.Current
    $kind = $c.ControlType.ProgrammaticName -replace '^ControlType\.', ''
    $extra = ""
    try { $s = $e.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern); $extra += " selected=" + $s.Current.IsSelected } catch {}
    try { $t = $e.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern); $extra += " toggle=" + $t.Current.ToggleState } catch {}
    if (-not $c.IsEnabled) { $extra += " disabled" }
    if ($c.ClassName) { $extra += " class=" + $c.ClassName }
    ("  " * $level) + "$kind `"$($c.Name)`"$extra"
}

function Tree($e, $level) {
    if ($level -gt $Depth) { return }
    $log.Add((Line $e $level))
    $w = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    $k = $w.GetFirstChild($e)
    while ($k) { Tree $k ($level + 1); $k = $w.GetNextSibling($k) }
}

$proc = $null
if ($Exe) {
    if ($Arguments) { $proc = Start-Process $Exe -ArgumentList $Arguments -PassThru } else { $proc = Start-Process $Exe -PassThru }
    Start-Sleep 4
}
$condition = New-Object System.Windows.Automation.PropertyCondition($A::NameProperty, $Title)
$window = $null
for ($i = 0; $i -lt 20 -and -not $window; $i++) {
    $window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
    if (-not $window) { Start-Sleep -Milliseconds 500 }
}
if (-not $window) { "window '$Title' not found" | Out-File $Out -Encoding utf8; exit 1 }

# AccessKit builds its tree when asked; give it a moment.
$null = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
Start-Sleep 1
$log.Add("== tree")
Tree $window 0

$log.Add("== focus per key")
# The window itself takes no focus; the app sets it at start.
$list = @("")
# Keys: "{DOWN}*20 {END} {UP}" (spaces between, *n = n times).
if ($Keys) {
    $list = @("")
    foreach ($t in $Keys.Split(" ")) {
        if ($t -match '^(.+)\*(\d+)$') { $list += @($Matches[1]) * [int]$Matches[2] } else { $list += $t }
    }
}
foreach ($key in $list) {
    if ($key) { [System.Windows.Forms.SendKeys]::SendWait($key); Start-Sleep -Milliseconds 900 }
    $f = $A::FocusedElement
    $log.Add("after '$key': " + (Line $f 0))
    # An open native menu: what NVDA reads from it.
    $menu = [Msaa]::Menu()
    if ($menu) { $log.Add($menu) }
}
if ($proc) {
    $proc.Refresh()
    $log.Add("memory: " + [math]::Round($proc.WorkingSet64 / 1MB) + " MB")
    # Close only the app this script started.
    Stop-Process -Id $proc.Id -ErrorAction SilentlyContinue
}
$log | Out-File $Out -Encoding utf8
