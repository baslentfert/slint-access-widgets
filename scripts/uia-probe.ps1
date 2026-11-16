# What a screen reader (NVDA, Narrator) gets on Windows: the UI Automation
# tree with role and name, then after each key where the focus is. Must run
# in the logged-in session (a scheduled task with /IT), not over SSH: it
# sends real keys to the foreground window.
#   uia-probe.ps1 -Title "Menu probe" -Exe app.exe -Keys "{TAB} +{F10}*1 {DOWN}*2" -Out uia.log
param([string]$Title = "Menu probe", [string]$Exe = "", [string]$Out = "uia.log",
      [string]$Keys = "", [int]$Depth = 12, [string]$Arguments = "")

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
# Native Win32 menus (Slint's context menu on Windows) are read through MSAA,
# as NVDA does (the .NET UI Automation client here shows them as empty
# panes): `cargo build --release -p slint-a11y-check` builds msaa-menu.exe.
$msaaMenu = Join-Path $PSScriptRoot "..\target\release\msaa-menu.exe"
$A = [System.Windows.Automation.AutomationElement]
$log = New-Object System.Collections.Generic.List[string]

function Line($e, $level) {
    $c = $e.Current
    $kind = $c.ControlType.ProgrammaticName -replace '^ControlType\.', ''
    $extra = ""
    try { $s = $e.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern); $extra += " selected=" + $s.Current.IsSelected } catch {}
    try { $t = $e.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern); $extra += " toggle=" + $t.Current.ToggleState } catch {}
    try { $v = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern); $extra += " value='" + $v.Current.Value + "'" } catch {}
    try { $x = $e.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern); $extra += " expand=" + $x.Current.ExpandCollapseState } catch {}
    if ($c.HelpText) { $extra += " help='" + $c.HelpText.Substring(0, [Math]::Min(30, $c.HelpText.Length)) + "'" }
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
    if (Test-Path $msaaMenu) {
        $menu = & $msaaMenu
        if ($menu) { $menu | ForEach-Object { $log.Add($_) } }
    }
}
if ($proc) {
    $proc.Refresh()
    $log.Add("memory: " + [math]::Round($proc.WorkingSet64 / 1MB) + " MB")
    # Close only the app this script started.
    Stop-Process -Id $proc.Id -ErrorAction SilentlyContinue
}
$log | Out-File $Out -Encoding utf8
