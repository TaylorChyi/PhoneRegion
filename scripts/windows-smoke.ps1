param(
    [string]$ExePath = './target/release/PhoneRegion.exe',
    [switch]$BlockNetwork,
    [string]$DemoDirectory
)

$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class NativeWindow
{
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string className, string windowName);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr GetDlgItem(IntPtr window, int id);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern int GetDlgCtrlID(IntPtr window);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr GetParent(IntPtr window);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassNameW(IntPtr window, StringBuilder className, int capacity);

    public delegate bool EnumWindowCallback(IntPtr window, IntPtr argument);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool EnumChildWindows(IntPtr parent, EnumWindowCallback callback, IntPtr argument);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr SendMessageW(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);

    [DllImport("user32.dll", EntryPoint = "SendMessageW", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr SendTextMessageW(IntPtr window, uint message, IntPtr wparam, string text);

    [DllImport("user32.dll", EntryPoint = "SendMessageW", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr ReadTextMessageW(IntPtr window, uint message, IntPtr capacity, StringBuilder text);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessageW(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowCallback callback, IntPtr argument);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetWindowTextW(IntPtr window, StringBuilder text, int size);

    public static string DescribeProcessWindows(uint pid)
    {
        var lines = new List<string>();
        EnumWindowCallback record = (hwnd, unused) => {
            var cls = new StringBuilder(100);
            var title = new StringBuilder(240);
            GetClassNameW(hwnd, cls, cls.Capacity);
            GetWindowTextW(hwnd, title, title.Capacity);
            lines.Add($"{cls} [{GetDlgCtrlID(hwnd)}] {title}");
            return true;
        };
        EnumWindows((hwnd, unused) => {
            GetWindowThreadProcessId(hwnd, out uint owner);
            if (owner == pid) {
                record(hwnd, unused);
                EnumChildWindows(hwnd, record, IntPtr.Zero);
            }
            return true;
        }, IntPtr.Zero);
        return string.Join(" | ", lines);
    }

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern bool IsWindowEnabled(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [StructLayout(LayoutKind.Sequential)]
    public struct WindowRect { public int left, top, right, bottom; }
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetWindowRect(IntPtr window, out WindowRect rect);
    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr window, uint attribute,
        out WindowRect bounds, int size);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PrintWindow(IntPtr window, IntPtr deviceContext, uint flags);

    [StructLayout(LayoutKind.Sequential)]
    private struct GuiThreadInfo {
        public int size, flags;
        public IntPtr active, focus, capture, menu, move, caret;
        public int left, top, right, bottom;
    }
    [DllImport("user32.dll")]
    private static extern bool GetGUIThreadInfo(uint thread, ref GuiThreadInfo info);

    public static IntPtr FocusedControl(IntPtr window) {
        var info = new GuiThreadInfo { size = Marshal.SizeOf<GuiThreadInfo>() };
        uint thread = GetWindowThreadProcessId(window, out uint owner);
        return GetGUIThreadInfo(thread, ref info) ? info.focus : IntPtr.Zero;
    }
}
'@

function Read-WindowText([IntPtr]$Window) {
    $text = [System.Text.StringBuilder]::new(4096)
    [void][NativeWindow]::ReadTextMessageW($Window, 0x000D, [IntPtr]$text.Capacity, $text)
    return $text.ToString()
}

function Assert-Result([IntPtr]$InputControl, [IntPtr]$QueryControl, [IntPtr]$ResultControl,
                       [string]$Number, [string[]]$Expected, [string[]]$Forbidden, [switch]$Live) {
    if ([NativeWindow]::SendTextMessageW($InputControl, 0x000C, [IntPtr]::Zero, $Number) -eq [IntPtr]::Zero) {
        throw "Could not enter the test number."
    }
    if (-not $Live) {
        [void][NativeWindow]::SendMessageW($QueryControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)
    }
    $result = Read-WindowText $ResultControl
    foreach ($word in $Expected) {
        if (-not $result.Contains($word)) {
            throw "The result for '$Number' omitted '$word': $result"
        }
    }
    foreach ($word in $Forbidden) {
        if ($result.Contains($word)) {
            throw "The result for '$Number' retained '$word': $result"
        }
    }
}

function Wait-Dialog([string]$Title, [int]$ProcessId) {
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        $dialog = [NativeWindow]::FindWindowW('#32770', $Title)
        if ($dialog -ne [IntPtr]::Zero) {
            [uint32]$dialogProcessId = 0
            [void][NativeWindow]::GetWindowThreadProcessId($dialog, [ref]$dialogProcessId)
            if ($dialogProcessId -eq $ProcessId) {
                return $dialog
            }
        }
        Start-Sleep -Milliseconds 100
    }
    throw "The '$Title' file dialog did not appear within 10 seconds."
}

function Complete-FileDialog([IntPtr]$Dialog, [string]$Path, [string]$DemoFrame) {
    # Explorer uses different child IDs for Open and Save. Alt+N focuses the
    # filename editor in both dialogs on the Windows runner's English desktop.
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        if ([NativeWindow]::IsWindowVisible($Dialog) -and [NativeWindow]::IsWindowEnabled($Dialog)) {
            [void][NativeWindow]::SetForegroundWindow($Dialog)
            if ([NativeWindow]::GetForegroundWindow() -eq $Dialog) {
                [System.Windows.Forms.SendKeys]::SendWait('%n')
                [System.Windows.Forms.SendKeys]::SendWait('^a')
                Set-Clipboard -Value $Path
                [System.Windows.Forms.SendKeys]::SendWait('^v')
                Start-Sleep -Milliseconds 350
                $focused = [NativeWindow]::FocusedControl($Dialog)
                if ($focused -ne [IntPtr]::Zero -and (Read-WindowText $focused) -eq $Path) {
                    if ($DemoFrame) { Save-DemoFrame $Dialog $DemoFrame }
                    [System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
                    return
                }
            }
        }
        Start-Sleep -Milliseconds 150
    }
    throw "The filename edit did not retain '$Path'."
}

function Save-DemoFrame([IntPtr]$Window, [string]$Name) {
    if (-not $DemoDirectory) { return }
    $full = [NativeWindow+WindowRect]::new()
    if (-not [NativeWindow]::GetWindowRect($Window, [ref]$full)) {
        throw "Could not locate the demo window for '$Name'."
    }
    $visible = [NativeWindow+WindowRect]::new()
    $status = [NativeWindow]::DwmGetWindowAttribute($Window, 9, [ref]$visible,
        [System.Runtime.InteropServices.Marshal]::SizeOf($visible))
    if ($status -ne 0) {
        throw "Could not determine the visible window bounds for '$Name' (DWM $status)."
    }
    if ($visible.left -lt $full.left -or $visible.top -lt $full.top -or
        $visible.right -gt $full.right -or $visible.bottom -gt $full.bottom) {
        throw "The visible frame extends outside the window rectangle for '$Name'."
    }
    $width = $visible.right - $visible.left
    $height = $visible.bottom - $visible.top
    if ($width -lt 200 -or $height -lt 200) {
        throw "The demo window has unexpected dimensions: ${width}x${height}."
    }
    $bitmap = [System.Drawing.Bitmap]::new($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        [void][NativeWindow]::SetForegroundWindow($Window)
        Start-Sleep -Milliseconds 180
        if ([NativeWindow]::GetForegroundWindow() -eq $Window) {
            $graphics.CopyFromScreen($visible.left, $visible.top, 0, 0,
                [System.Drawing.Size]::new($width, $height))
        }
        else {
            $fullWidth = $full.right - $full.left
            $fullHeight = $full.bottom - $full.top
            $fullBitmap = [System.Drawing.Bitmap]::new($fullWidth, $fullHeight)
            $fullGraphics = [System.Drawing.Graphics]::FromImage($fullBitmap)
            try {
                $deviceContext = $fullGraphics.GetHdc()
                try {
                    if (-not [NativeWindow]::PrintWindow($Window, $deviceContext, 2)) {
                        throw "Could not capture the demo window for '$Name'."
                    }
                }
                finally {
                    $fullGraphics.ReleaseHdc($deviceContext)
                }
                $source = [System.Drawing.Rectangle]::new(
                    $visible.left - $full.left, $visible.top - $full.top, $width, $height)
                $destination = [System.Drawing.Rectangle]::new(0, 0, $width, $height)
                $graphics.DrawImage($fullBitmap, $destination, $source,
                    [System.Drawing.GraphicsUnit]::Pixel)
            }
            finally {
                $fullGraphics.Dispose()
                $fullBitmap.Dispose()
            }
        }
        $bitmap.Save((Join-Path $DemoDirectory $Name),
            [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

$source = (Resolve-Path -LiteralPath $ExePath).Path
$demoEnabled = -not [string]::IsNullOrWhiteSpace($DemoDirectory)
if ($demoEnabled) {
    $DemoDirectory = [System.IO.Path]::GetFullPath($DemoDirectory)
    New-Item -ItemType Directory -Path $DemoDirectory -Force | Out-Null
}
$scratchRoot = if ($demoEnabled -and $env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$scratchName = if ($demoEnabled) {
    'PhoneDemo-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
} else {
    'PhoneRegionSmoke-' + [guid]::NewGuid().ToString('N')
}
$isolated = Join-Path $scratchRoot $scratchName
New-Item -ItemType Directory -Path $isolated | Out-Null
$copiedExe = Join-Path $isolated 'PhoneRegion.exe'
Copy-Item -LiteralPath $source -Destination $copiedExe
$application = $null
$firewallRule = $null

try {
    # Launch a copy away from the repository to check that the lookup data is bundled.
    if ($BlockNetwork) {
        $newRuleName = 'PhoneRegionSmoke-' + [guid]::NewGuid().ToString('N')
        New-NetFirewallRule -Name $newRuleName -DisplayName $newRuleName -Direction Outbound -Program $copiedExe -Action Block -Profile Any | Out-Null
        $firewallRule = $newRuleName
        if (-not (Get-NetFirewallRule -Name $firewallRule -ErrorAction SilentlyContinue)) {
            throw 'The outbound block rule was not installed.'
        }
    }
    $application = Start-Process -FilePath $copiedExe -WorkingDirectory $isolated -PassThru
    [void]$application.WaitForInputIdle(15000)
    $window = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        Start-Sleep -Milliseconds 150
        $application.Refresh()
        if ($application.HasExited) {
            throw "The application exited before its window appeared (exit code $($application.ExitCode))."
        }
        $candidate = [NativeWindow]::FindWindowW('PhoneRegionMainWindow', '手机号归属地 · 离线查询')
        if ($candidate -ne [IntPtr]::Zero) {
            [uint32]$windowProcessId = 0
            [void][NativeWindow]::GetWindowThreadProcessId($candidate, [ref]$windowProcessId)
            if ($windowProcessId -eq $application.Id -and
                [NativeWindow]::GetDlgItem($candidate, 1005) -ne [IntPtr]::Zero) {
                $window = $candidate
                break
            }
        }
    }
    if ($window -eq [IntPtr]::Zero) {
        throw 'The native application controls did not initialise within 15 seconds.'
    }

    $inputControl = [NativeWindow]::GetDlgItem($window, 1001)
    $queryControl = [NativeWindow]::GetDlgItem($window, 1002)
    $resultControl = [NativeWindow]::GetDlgItem($window, 1004)
    if ($inputControl -eq [IntPtr]::Zero -or $queryControl -eq [IntPtr]::Zero -or
        $resultControl -eq [IntPtr]::Zero) {
        throw 'The phone input, query button or visible result control is missing.'
    }

    if ($demoEnabled) {
        Save-DemoFrame $window '01-empty.png'
    }
    Assert-Result $inputControl $queryControl $resultControl '13213000000' @('中国联通', '河南省', '郑州市') @() -Live
    Save-DemoFrame $window '02-single-result.png'
    if ($demoEnabled) {
        foreach ($stage in @(
            @('03-prefix.png', '+86'),
            @('04-area.png', '+86 132'),
            @('05-group.png', '+86 132 1300')
        )) {
            [void][NativeWindow]::SendTextMessageW($inputControl, 0x000C,
                [IntPtr]::Zero, $stage[1])
            Save-DemoFrame $window $stage[0]
        }
    }
    Assert-Result $inputControl $queryControl $resultControl '+86 132 1300 0000' @('中国联通', '河南省', '郑州市') @()
    Save-DemoFrame $window '06-formatted-result.png'
    Assert-Result $inputControl $queryControl $resultControl '123' @('请输入 11 位') @('中国联通', '河南省', '郑州市')
    [void][NativeWindow]::SendTextMessageW($inputControl, 0x000C, [IntPtr]::Zero, '')
    if ((Read-WindowText $resultControl) -ne '') {
        throw 'Clearing the input did not clear the old result.'
    }
    if (-not [NativeWindow]::PostMessageW($inputControl, 0x0100, [IntPtr]13, [IntPtr]::Zero)) {
        throw 'Could not send Enter to the phone input.'
    }
    $enterWorked = $false
    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        if ((Read-WindowText $resultControl).Contains('请输入手机号')) {
            $enterWorked = $true
            break
        }
        Start-Sleep -Milliseconds 50
    }
    if (-not $enterWorked) {
        throw 'Pressing Enter in the phone input did not trigger a query.'
    }
    if ($demoEnabled) {
        Assert-Result $inputControl $queryControl $resultControl '+86 132 1300 0000' @('中国联通', '河南省', '郑州市') @()
        Save-DemoFrame $window '07-before-import.png'
    }

    $inputCsv = Join-Path $isolated 'phones.csv'
    $outputCsv = Join-Path $isolated 'result.csv'
    $csvContent = "手机号`r`n13213000000`r`n+86 132 1300 0000`r`n123`r`n"
    if ($demoEnabled) {
        $csvContent += "13900200000`r`n18901600000`r`n"
    }
    [System.IO.File]::WriteAllText($inputCsv, $csvContent,
        [System.Text.UTF8Encoding]::new($true))
    $importControl = [NativeWindow]::GetDlgItem($window, 1003)
    $progressControl = [NativeWindow]::GetDlgItem($window, 1005)
    if ($importControl -eq [IntPtr]::Zero -or $progressControl -eq [IntPtr]::Zero) {
        throw 'The batch import button or progress control is missing.'
    }
    if (-not [NativeWindow]::PostMessageW($importControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not open the batch file picker.'
    }
    $openDialog = Wait-Dialog '选择手机号文件' $application.Id
    $openFrame = if ($demoEnabled) { '08-open-dialog.png' } else { '' }
    Complete-FileDialog $openDialog $inputCsv $openFrame
    $saveDialog = Wait-Dialog '保存查询结果' $application.Id
    $saveFrame = if ($demoEnabled) { '09-save-dialog.png' } else { '' }
    Complete-FileDialog $saveDialog $outputCsv $saveFrame

    for ($attempt = 0; $attempt -lt 200 -and -not (Test-Path -LiteralPath $outputCsv); $attempt++) {
        $application.Refresh()
        if ($application.HasExited) {
            throw "The application exited while processing the CSV (exit code $($application.ExitCode))."
        }
        Start-Sleep -Milliseconds 100
    }
    if (-not (Test-Path -LiteralPath $outputCsv)) {
        throw "The output CSV did not appear within 20 seconds. Progress: $(Read-WindowText $progressControl)"
    }
    $records = @(Import-Csv -LiteralPath $outputCsv -Encoding utf8)
    $expectedRows = if ($demoEnabled) { 5 } else { 3 }
    if ($records.Count -ne $expectedRows) {
        throw "Expected $expectedRows output rows, found $($records.Count)."
    }
    $columns = @($records[0].PSObject.Properties.Name)
    if (($columns -join '|') -ne '手机号|运营商|归属地省份|归属地地级市') {
        throw "Unexpected output columns: $($columns -join ', ')"
    }
    for ($index = 0; $index -lt 2; $index++) {
        if ($records[$index].手机号 -ne '13213000000' -or $records[$index].运营商 -ne '中国联通' -or
            $records[$index].归属地省份 -ne '河南省' -or $records[$index].归属地地级市 -ne '郑州市') {
            throw "Incorrect CSV output at row $($index + 2)."
        }
    }
    if ($records[2].运营商 -ne '' -or $records[2].归属地省份 -ne '' -or $records[2].归属地地级市 -ne '') {
        throw 'The invalid number received invented location or carrier data.'
    }
    if ($demoEnabled) {
        $mobile = $records[3]
        $telecom = $records[4]
        if ($mobile.手机号 -ne '13900200000' -or $mobile.运营商 -ne '中国移动' -or
            $mobile.归属地省份 -ne '广东省' -or $mobile.归属地地级市 -ne '广州市') {
            throw 'The demo CSV mobile carrier row is incorrect.'
        }
        if ($telecom.手机号 -ne '18901600000' -or $telecom.运营商 -ne '中国电信' -or
            $telecom.归属地省份 -ne '上海市' -or $telecom.归属地地级市 -ne '上海市') {
            throw 'The demo CSV telecom carrier row is incorrect.'
        }
    }
    if (-not (Test-Path -LiteralPath $inputCsv)) {
        throw 'The original input CSV disappeared.'
    }
    if ($demoEnabled) {
        for ($attempt = 0; $attempt -lt 40; $attempt++) {
            if ((Read-WindowText $progressControl).Contains('已保存')) { break }
            Start-Sleep -Milliseconds 100
        }
        if (-not (Read-WindowText $progressControl).Contains('已保存')) {
            throw 'The batch completion status did not appear for the demo.'
        }
        Save-DemoFrame $window '10-batch-complete.png'
        Copy-Item -LiteralPath $outputCsv -Destination (Join-Path $DemoDirectory 'result.csv')
    }
    Write-Host 'Windows GUI smoke test passed, including a standalone EXE launch.'
}
catch {
    if ($null -ne $application -and -not $application.HasExited) {
        Write-Host ([NativeWindow]::DescribeProcessWindows([uint32]$application.Id))
        $byProcess = [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ProcessIdProperty, [int]$application.Id)
        $windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll(
            [System.Windows.Automation.TreeScope]::Children, $byProcess)
        foreach ($wnd in $windows) {
            $labels = $wnd.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition)
            Write-Host (($labels | ForEach-Object { $_.Current.Name } | Where-Object { $_ }) -join ' / ')
        }
    }
    throw
}
finally {
    if ($null -ne $application) {
        $application.Refresh()
        if (-not $application.HasExited) {
            Stop-Process -Id $application.Id -Force
            $application.WaitForExit()
        }
        $application.Dispose()
    }
    if ($null -ne $firewallRule) {
        Remove-NetFirewallRule -Name $firewallRule -ErrorAction Stop
    }
    Remove-Item -LiteralPath $isolated -Recurse -Force
}
