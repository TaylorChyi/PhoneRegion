param(
    [string]$ExePath = './target/release/PhoneRegion.exe',
    [switch]$BlockNetwork,
    [string]$DemoDirectory
)

$ErrorActionPreference = 'Stop'

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

    public static IntPtr FindDialogChild(IntPtr dialog, string className, int id) {
        IntPtr found = IntPtr.Zero;
        EnumChildWindows(dialog, (child, unused) => {
            if (GetDlgCtrlID(child) == id) {
                var name = new StringBuilder(64);
                GetClassNameW(child, name, name.Capacity);
                if (name.ToString() == className) { found = child; return false; }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

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
    public static extern bool IsWindow(IntPtr hwnd);
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
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetClientRect(IntPtr window, out WindowRect rect);
    [DllImport("user32.dll")]
    public static extern bool GetUpdateRect(IntPtr window, out WindowRect rect, bool erase);
    [DllImport("user32.dll")]
    public static extern bool UpdateWindow(IntPtr window);
    [StructLayout(LayoutKind.Sequential)]
    public struct WindowPoint { public int x, y; }
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool ClientToScreen(IntPtr window, ref WindowPoint point);
    [DllImport("user32.dll")]
    public static extern IntPtr GetWindowDpiAwarenessContext(IntPtr window);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool AreDpiAwarenessContextsEqual(IntPtr first, IntPtr second);
    [DllImport("user32.dll")]
    public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y,
        int width, int height, uint flags);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsZoomed(IntPtr window);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsIconic(IntPtr window);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool OpenClipboard(IntPtr owner);
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool EmptyClipboard();
    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr SetClipboardData(uint format, IntPtr data);
    [DllImport("user32.dll")]
    public static extern bool CloseClipboard();
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern IntPtr GlobalAlloc(uint flags, UIntPtr bytes);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern IntPtr GlobalLock(IntPtr memory);
    [DllImport("kernel32.dll")]
    public static extern bool GlobalUnlock(IntPtr memory);
    [DllImport("kernel32.dll")]
    public static extern IntPtr GlobalFree(IntPtr memory);
    public static void PutClipboardText(string value) {
        var encoded = Encoding.Unicode.GetBytes(value + "\0");
        IntPtr memory = GlobalAlloc(2, (UIntPtr)encoded.Length);
        if (memory == IntPtr.Zero) throw new Exception("GlobalAlloc failed for clipboard text");
        IntPtr pointer = GlobalLock(memory);
        if (pointer == IntPtr.Zero) { GlobalFree(memory); throw new Exception("GlobalLock failed"); }
        Marshal.Copy(encoded, 0, pointer, encoded.Length);
        GlobalUnlock(memory);
        bool opened = false;
        for (int attempt = 0; attempt < 20 && !opened; attempt++) {
            opened = OpenClipboard(IntPtr.Zero);
            if (!opened) System.Threading.Thread.Sleep(25);
        }
        if (!opened) { GlobalFree(memory); throw new Exception("OpenClipboard failed"); }
        try {
            if (!EmptyClipboard() || SetClipboardData(13, memory) == IntPtr.Zero)
                throw new Exception("SetClipboardData failed");
            memory = IntPtr.Zero;
        } finally {
            CloseClipboard();
            if (memory != IntPtr.Zero) GlobalFree(memory);
        }
    }
    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr window, uint attribute,
        out WindowRect bounds, int size);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PrintWindow(IntPtr window, IntPtr deviceContext, uint flags);

}
'@

function Read-WindowText([IntPtr]$Window) {
    $text = [System.Text.StringBuilder]::new(4096)
    [void][NativeWindow]::ReadTextMessageW($Window, 0x000D, [IntPtr]$text.Capacity, $text)
    return $text.ToString()
}

function Assert-ResultState([IntPtr]$Window, [string]$Carrier, [string]$Province,
                            [string]$City, [string]$Caveat) {
    $labelControl = [NativeWindow]::GetDlgItem($Window, 1006)
    if ($labelControl -ne [IntPtr]::Zero -and
        [NativeWindow]::IsWindowVisible($labelControl) -and
        (Read-WindowText $labelControl) -eq '查询结果') {
        throw 'The removed result heading is still visible.'
    }
    $actual = Read-WindowText ([NativeWindow]::GetDlgItem($Window, 1004))
    $expected = "运营商 $Carrier`r`n省份 $Province`r`n地级市 $City"
    if ($actual -ne $expected) {
        throw "Result fields differ. Expected '$expected'; got '$actual'."
    }
    $note = Read-WindowText ([NativeWindow]::GetDlgItem($Window, 1007))
    if ($Caveat -eq '') {
        if ($note -ne '') { throw "Unexpected result note: '$note'." }
    }
    elseif (-not $note.Contains($Caveat)) {
        throw "Result note omitted '$Caveat': '$note'."
    }
}

function Snapshot-StableControls([IntPtr]$Window) {
    $snapshot = @{}
    foreach ($id in @(1003, 1004, 1008, 1009)) {
        $rect = Read-WindowRect ([NativeWindow]::GetDlgItem($Window, $id))
        $snapshot[$id] = "{0},{1},{2},{3}" -f $rect.left, $rect.top, $rect.right, $rect.bottom
    }
    return $snapshot
}

function Assert-StableControls([IntPtr]$Window, [hashtable]$Baseline) {
    $current = Snapshot-StableControls $Window
    foreach ($id in $Baseline.Keys) {
        if ($current[$id] -ne $Baseline[$id]) {
            throw "Control $id moved during phone editing: $($Baseline[$id]) -> $($current[$id])."
        }
    }
}

function Test-ClearStatus([IntPtr]$ProgressControl) {
    return (-not [NativeWindow]::IsWindowVisible($ProgressControl) -or
        (Read-WindowText $ProgressControl) -eq '')
}

function Assert-ExportAvailability([IntPtr]$ExportControl, [bool]$Enabled) {
    if ($ExportControl -eq [IntPtr]::Zero -or
        -not [NativeWindow]::IsWindowVisible($ExportControl)) {
        throw 'The export button must remain visible.'
    }
    if ([NativeWindow]::IsWindowEnabled($ExportControl) -ne $Enabled) {
        throw "Export button enabled state differs from expected $Enabled."
    }
}

function Assert-BatchReady([IntPtr]$ExportControl, [IntPtr]$ProgressControl) {
    if (-not [NativeWindow]::IsWindowVisible($ExportControl) -or
        -not [NativeWindow]::IsWindowEnabled($ExportControl)) {
        throw 'The export button is not visible and enabled after successful processing.'
    }
    if (-not (Test-ClearStatus $ProgressControl)) {
        throw "A success status remains visible: $(Read-WindowText $ProgressControl)"
    }
}

function Assert-NoFullWindowInvalidation([IntPtr]$Window) {
    $pending = [NativeWindow+WindowRect]::new()
    if (-not [NativeWindow]::GetUpdateRect($Window, [ref]$pending, $false)) { return }
    $client = [NativeWindow+WindowRect]::new()
    [void][NativeWindow]::GetClientRect($Window, [ref]$client)
    $pendingArea = ($pending.right - $pending.left) * ($pending.bottom - $pending.top)
    $clientArea = $client.right * $client.bottom
    if ($clientArea -gt 0 -and $pendingArea -gt $clientArea * 0.75) {
        throw "Typing invalidated most of the application client: $pendingArea / $clientArea pixels."
    }
}

function Check-LiveEditing([IntPtr]$Window, [IntPtr]$InputControl, [IntPtr]$QueryControl) {
    [void][NativeWindow]::SendTextMessageW($InputControl, 0x000C, [IntPtr]::Zero, '')
    Assert-ResultState $Window '—' '—' '—' ''
    $stable = Snapshot-StableControls $Window
    foreach ($character in '13213000000'.ToCharArray()) {
        [void][NativeWindow]::UpdateWindow($Window)
        [void][NativeWindow]::SendMessageW($InputControl, 0x0102, [IntPtr][int][char]$character, [IntPtr]::Zero)
        Assert-NoFullWindowInvalidation $Window
        Assert-StableControls $Window $stable
        if ((Read-WindowText $InputControl).Length -lt 11) {
            Assert-ResultState $Window '—' '—' '—' ''
        }
    }
    if ((Read-WindowText $InputControl) -ne '13213000000') {
        throw 'WM_CHAR did not enter all eleven digits.'
    }
    Assert-ResultState $Window '中国联通' '河南省' '郑州市' '原号段'
    [void][NativeWindow]::UpdateWindow($Window)
    [void][NativeWindow]::SendMessageW($InputControl, 0x0102, [IntPtr]8, [IntPtr]::Zero)
    Assert-NoFullWindowInvalidation $Window
    if ((Read-WindowText $InputControl) -ne '1321300000') { throw 'Backspace did not remove the last digit.' }
    Assert-ResultState $Window '—' '—' '—' ''
    Assert-StableControls $Window $stable
    [void][NativeWindow]::SendMessageW($InputControl, 0x0102, [IntPtr][int][char]'0', [IntPtr]::Zero)
    Assert-ResultState $Window '中国联通' '河南省' '郑州市' '原号段'
    [void][NativeWindow]::SendTextMessageW($InputControl, 0x000C, [IntPtr]::Zero, '')
    [NativeWindow]::PutClipboardText('+86 132 1300 0000')
    [void][NativeWindow]::SendMessageW($InputControl, 0x0302, [IntPtr]::Zero, [IntPtr]::Zero)
    if ((Read-WindowText $InputControl) -ne '+86 132 1300 0000') {
        throw 'WM_PASTE did not insert the formatted phone number.'
    }
    Assert-ResultState $Window '中国联通' '河南省' '郑州市' '原号段'
    Assert-StableControls $Window $stable
    [void][NativeWindow]::SendMessageW($QueryControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)
    Assert-ResultState $Window '中国联通' '河南省' '郑州市' '原号段'
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
        if ($dialog -ne [IntPtr]::Zero -and [NativeWindow]::IsWindowVisible($dialog) -and
            [NativeWindow]::IsWindowEnabled($dialog)) {
            [uint32]$dialogProcessId = 0
            [void][NativeWindow]::GetWindowThreadProcessId($dialog, [ref]$dialogProcessId)
            $cancel = [NativeWindow]::FindDialogChild($dialog, 'Button', 2)
            if ($dialogProcessId -eq $ProcessId -and $cancel -ne [IntPtr]::Zero -and
                [NativeWindow]::IsWindowVisible($cancel)) {
                return $dialog
            }
        }
        Start-Sleep -Milliseconds 100
    }
    throw "The '$Title' file dialog did not appear within 10 seconds."
}

function Complete-FileDialog([IntPtr]$Dialog, [string]$Path, [string]$DemoFrame) {
    # Explorer builds the file-name ComboBox after the dialog HWND appears.
    # The filename edit is 1001 in the Save dialogue and 1148 in the classic Open dialogue.
    for ($attempt = 0; $attempt -lt 80; $attempt++) {
        if ([NativeWindow]::IsWindowVisible($Dialog) -and [NativeWindow]::IsWindowEnabled($Dialog)) {
            $filename = [NativeWindow]::FindDialogChild($Dialog, 'Edit', 1001)
            if ($filename -eq [IntPtr]::Zero) {
                $filename = [NativeWindow]::FindDialogChild($Dialog, 'Edit', 1148)
            }
            $confirm = [NativeWindow]::FindDialogChild($Dialog, 'Button', 1)
            if ($filename -ne [IntPtr]::Zero -and $confirm -ne [IntPtr]::Zero -and
                [NativeWindow]::IsWindowEnabled($filename)) {
                # Pasting into the focused edit also notifies Explorer's filename model.
                [void][NativeWindow]::SendMessageW($Dialog, 0x0028, $filename, [IntPtr]1)
                [void][NativeWindow]::SendMessageW($filename, 0x00B1, [IntPtr]::Zero, [IntPtr](-1))
                [NativeWindow]::PutClipboardText($Path)
                [void][NativeWindow]::SendMessageW($filename, 0x0302, [IntPtr]::Zero, [IntPtr]::Zero)
                Start-Sleep -Milliseconds 100
                if ([NativeWindow]::IsWindowVisible($Dialog) -and [NativeWindow]::IsWindowEnabled($confirm) -and
                    (Read-WindowText $filename) -eq $Path) {
                    if ($DemoFrame) { Save-DemoFrame $Dialog $DemoFrame }
                    if (-not [NativeWindow]::PostMessageW($confirm, 0x00F5,
                        [IntPtr]::Zero, [IntPtr]::Zero)) {
                        throw 'Could not dispatch the file dialog confirmation.'
                    }
                    return
                }
            }
        }
        Start-Sleep -Milliseconds 100
    }
    throw "The file dialog filename control did not accept '$Path' (dialog=$Dialog visible=$([NativeWindow]::IsWindowVisible($Dialog)), edit=$filename, confirm=$confirm)."
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

function Read-WindowRect([IntPtr]$Window) {
    $rect = [NativeWindow+WindowRect]::new()
    if (-not [NativeWindow]::GetWindowRect($Window, [ref]$rect)) {
        throw 'Could not read the native window rectangle.'
    }
    return $rect
}

function Assert-ControlBounds([IntPtr]$Window, [int[]]$Ids) {
    $client = [NativeWindow+WindowRect]::new()
    if (-not [NativeWindow]::GetClientRect($Window, [ref]$client)) {
        throw 'Could not read the client viewport.'
    }
    $origin = [NativeWindow+WindowPoint]::new()
    if (-not [NativeWindow]::ClientToScreen($Window, [ref]$origin)) {
        throw 'Could not locate the client viewport on screen.'
    }
    $viewport = [NativeWindow+WindowRect]::new()
    $viewport.left = $origin.x
    $viewport.top = $origin.y
    $viewport.right = $origin.x + $client.right
    $viewport.bottom = $origin.y + $client.bottom
    foreach ($id in $Ids) {
        $control = [NativeWindow]::GetDlgItem($Window, $id)
        if ($control -eq [IntPtr]::Zero) { throw "Missing control $id after resizing." }
        $child = Read-WindowRect $control
        if ($child.right -le $child.left -or $child.bottom -le $child.top -or
            $child.left -lt $viewport.left -or $child.right -gt $viewport.right -or
            $child.top -lt $viewport.top -or $child.bottom -gt $viewport.bottom) {
            throw "Control $id extends beyond the visible client viewport."
        }
    }
}

function Assert-SamplePlacement([IntPtr]$Window, [bool]$Narrow) {
    $note = Read-WindowRect ([NativeWindow]::GetDlgItem($Window, 1007))
    $sample = Read-WindowRect ([NativeWindow]::GetDlgItem($Window, 1009))
    $import = Read-WindowRect ([NativeWindow]::GetDlgItem($Window, 1003))
    if ($sample.bottom -gt $import.top) {
        throw 'The example CSV button overlaps the file drop area.'
    }
    if ($Narrow) {
        if ($sample.top -lt $note.bottom) {
            throw 'The narrow layout did not move the example CSV button below the carrier note.'
        }
    }
    else {
        if ($sample.top -ge $note.bottom -or $sample.bottom -le $note.top -or
            $note.right -gt $sample.left) {
            throw 'The wide layout did not place the example CSV button beside the carrier note.'
        }
    }
}

function Scroll-WindowToEdge([IntPtr]$Window, [bool]$Bottom) {
    $command = if ($Bottom) { 3 } else { 2 } # SB_PAGEDOWN / SB_PAGEUP
    for ($page = 0; $page -lt 16; $page++) {
        [void][NativeWindow]::SendMessageW($Window, 0x0115, [IntPtr]$command, [IntPtr]::Zero)
    }
    Start-Sleep -Milliseconds 100
}

function Check-WindowLayouts([IntPtr]$Window, [IntPtr]$ResultControl) {
    $original = Read-WindowRect $Window
    $clientWidths = @()
    $resultHeights = @()
    try {
        foreach ($width in @(1080, 720)) {
            if (-not [NativeWindow]::SetWindowPos($Window, [IntPtr]::Zero,
                $original.left, $original.top, $width, 780, 0x0004)) {
                throw "Could not resize the window to $width pixels."
            }
            Start-Sleep -Milliseconds 160
            $client = [NativeWindow+WindowRect]::new()
            if (-not [NativeWindow]::GetClientRect($Window, [ref]$client)) {
                throw 'Could not read the resized client area.'
            }
            $clientWidths += $client.right - $client.left
            Scroll-WindowToEdge $Window $false
            Assert-ControlBounds $Window @(1001, 1002, 1004)
            $resultRect = Read-WindowRect $ResultControl
            $resultHeights += $resultRect.bottom - $resultRect.top
            if (-not (Read-WindowText $ResultControl).Contains('中国联通')) {
                throw 'The lookup result disappeared while resizing.'
            }
            if ($width -eq 720 -and $DemoDirectory) {
                Save-DemoFrame $Window 'layout-narrow.png'
            }
            Scroll-WindowToEdge $Window $true
            Assert-ControlBounds $Window @(1003, 1008)
            $status = [NativeWindow]::GetDlgItem($Window, 1005)
            if ([NativeWindow]::IsWindowVisible($status)) {
                Assert-ControlBounds $Window @(1005)
            }
            Assert-SamplePlacement $Window ($width -eq 720)
            Scroll-WindowToEdge $Window $false
        }
        if ($clientWidths[0] - $clientWidths[1] -lt 200) {
            throw 'The wide and narrow layouts did not receive distinct client widths.'
        }
        if ($resultHeights[1] - $resultHeights[0] -lt 70) {
            throw 'The narrow layout did not switch the result from columns to rows.'
        }
        [void][NativeWindow]::ShowWindow($Window, 3)
        Start-Sleep -Milliseconds 160
        if (-not [NativeWindow]::IsZoomed($Window)) {
            throw 'The application did not maximise.'
        }
        [void][NativeWindow]::ShowWindow($Window, 9)
        [void][NativeWindow]::ShowWindow($Window, 6)
        Start-Sleep -Milliseconds 120
        if (-not [NativeWindow]::IsIconic($Window)) {
            throw 'The application did not minimise.'
        }
        [void][NativeWindow]::ShowWindow($Window, 9)
    }
    finally {
        [void][NativeWindow]::ShowWindow($Window, 9)
        [void][NativeWindow]::SetWindowPos($Window, [IntPtr]::Zero,
            $original.left, $original.top,
            $original.right - $original.left, $original.bottom - $original.top, 0x0004)
    }
    return $clientWidths
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
        $candidate = [NativeWindow]::FindWindowW('PhoneRegionMainWindow', 'PhoneRegion')
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
    $exportControl = [NativeWindow]::GetDlgItem($window, 1008)
    if ($inputControl -eq [IntPtr]::Zero -or $queryControl -eq [IntPtr]::Zero -or
        $resultControl -eq [IntPtr]::Zero) {
        throw 'The phone input, query button or visible result control is missing.'
    }

    $dpiContext = [NativeWindow]::GetWindowDpiAwarenessContext($window)
    if (-not [NativeWindow]::AreDpiAwarenessContextsEqual($dpiContext, [IntPtr](-4))) {
        throw 'The application window is not running with Per-Monitor-v2 DPI awareness.'
    }
    $realWindowDpi = [NativeWindow]::GetDpiForWindow($window)
    if ($realWindowDpi -lt 96) { throw "Unexpected real Windows display DPI: $realWindowDpi" }
    Write-Host "Actual runner display DPI: $realWindowDpi; window context: Per-Monitor-v2."

    if ($demoEnabled) {
        Save-DemoFrame $window '01-empty.png'
    }
    Assert-ExportAvailability $exportControl $false
    Assert-ResultState $window '—' '—' '—' ''
    Check-LiveEditing $window $inputControl $queryControl
    Assert-Result $inputControl $queryControl $resultControl '13213000000' @('中国联通', '河南省', '郑州市') @() -Live
    Assert-ExportAvailability $exportControl $false
    Save-DemoFrame $window '02-single-result.png'
    $clientWidths = Check-WindowLayouts $window $resultControl
    if ($demoEnabled) {
        $validation = @{
            real_display_dpi = $realWindowDpi
            dpi_awareness = 'Per-Monitor-v2'
            dpi_144_192_in_external_process = 'not simulated'
            wide_client_width = $clientWidths[0]
            narrow_client_width = $clientWidths[1]
            window_actions = @('resize', 'maximise', 'minimise', 'restore')
        }
        $validation | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $DemoDirectory 'validation.json') -Encoding utf8
    }
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
    [void][NativeWindow]::SendTextMessageW($inputControl, 0x000C, [IntPtr]::Zero, '123')
    Assert-ResultState $window '—' '—' '—' ''
    [void][NativeWindow]::SendMessageW($queryControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)
    Assert-ResultState $window '—' '—' '—' '请输入 11 位'
    [void][NativeWindow]::SendTextMessageW($inputControl, 0x000C, [IntPtr]::Zero, '13447100000')
    Assert-ResultState $window '—' '—' '—' '未收录'
    [void][NativeWindow]::SendTextMessageW($inputControl, 0x000C, [IntPtr]::Zero, '')
    Assert-ResultState $window '—' '—' '—' ''
    if (-not [NativeWindow]::PostMessageW($inputControl, 0x0100, [IntPtr]13, [IntPtr]::Zero)) {
        throw 'Could not send Enter to the phone input.'
    }
    $enterWorked = $false
    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        if ((Read-WindowText ([NativeWindow]::GetDlgItem($window, 1007))).Contains('请输入手机号')) {
            $enterWorked = $true
            break
        }
        Start-Sleep -Milliseconds 50
    }
    if (-not $enterWorked) {
        throw 'Pressing Enter in the phone input did not trigger a query.'
    }
    Assert-ResultState $window '—' '—' '—' '请输入手机号'
    if ($demoEnabled) {
        Assert-Result $inputControl $queryControl $resultControl '+86 132 1300 0000' @('中国联通', '河南省', '郑州市') @()
        Save-DemoFrame $window '07-before-import.png'
    }

    $sampleControl = [NativeWindow]::GetDlgItem($window, 1009)
    if ($sampleControl -eq [IntPtr]::Zero -or -not [NativeWindow]::IsWindowVisible($sampleControl)) {
        throw 'The Save example CSV control is missing or hidden.'
    }
    $sampleCsv = Join-Path $isolated 'sample.csv'
    if (-not [NativeWindow]::PostMessageW($sampleControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not open the example CSV save dialog.'
    }
    $sampleDialog = Wait-Dialog '保存示例 CSV' $application.Id
    Complete-FileDialog $sampleDialog $sampleCsv ''
    for ($attempt = 0; $attempt -lt 40 -and -not (Test-Path -LiteralPath $sampleCsv); $attempt++) {
        Start-Sleep -Milliseconds 50
    }
    if (-not (Test-Path -LiteralPath $sampleCsv)) { throw 'The example CSV was not saved.' }
    $expectedExample = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes(
        (Resolve-Path -LiteralPath './docs/example.csv').Path))
    $savedExample = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($sampleCsv))
    if ($savedExample -ne $expectedExample) {
        throw 'The saved example CSV differs from the bundled three-row example.'
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
    $exportBefore = Read-WindowRect $exportControl
    if (-not [NativeWindow]::PostMessageW($importControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not open the batch file picker.'
    }
    $openDialog = Wait-Dialog '选择手机号文件' $application.Id
    $openFrame = if ($demoEnabled) { '08-open-dialog.png' } else { '' }
    Complete-FileDialog $openDialog $inputCsv $openFrame
    for ($attempt = 0; $attempt -lt 200; $attempt++) {
        $application.Refresh()
        if ($application.HasExited) { throw 'The application exited while importing the CSV.' }
        if ([NativeWindow]::IsWindowVisible($exportControl) -and
            [NativeWindow]::IsWindowEnabled($exportControl) -and
            (Test-ClearStatus $progressControl)) { break }
        Start-Sleep -Milliseconds 100
    }
    Assert-BatchReady $exportControl $progressControl
    $exportReady = Read-WindowRect $exportControl
    if ($exportReady.left -ne $exportBefore.left -or $exportReady.top -ne $exportBefore.top -or
        $exportReady.right -ne $exportBefore.right -or $exportReady.bottom -ne $exportBefore.bottom) {
        throw 'The export button moved or changed size after import.'
    }
    if ([NativeWindow]::FindWindowW('#32770', '保存查询结果') -ne [IntPtr]::Zero -or
        (Test-Path -LiteralPath $outputCsv) -or
        @(Get-ChildItem -LiteralPath $isolated -Filter 'phones_归属地结果*' -File).Count -gt 0) {
        throw 'Import opened the save dialog or wrote an output before the user requested export.'
    }
    Assert-ControlBounds $window @(1008)
    Save-DemoFrame $window '09-ready-to-export.png'
    if (-not [NativeWindow]::PostMessageW($exportControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not open the result save dialog.'
    }
    $cancelDialog = Wait-Dialog '保存查询结果' $application.Id
    $cancelButton = [NativeWindow]::FindDialogChild($cancelDialog, 'Button', 2)
    [void][NativeWindow]::PostMessageW($cancelButton, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)
    for ($attempt = 0; $attempt -lt 100 -and
        ([NativeWindow]::IsWindow($cancelDialog) -or -not [NativeWindow]::IsWindowEnabled($window)); $attempt++) {
        Start-Sleep -Milliseconds 50
    }
    if ([NativeWindow]::IsWindow($cancelDialog) -or -not [NativeWindow]::IsWindowEnabled($window)) {
        throw 'The save dialog did not close when cancelled.'
    }
    Assert-ExportAvailability $exportControl $true
    if (-not [NativeWindow]::PostMessageW($exportControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not reopen the result save dialog.'
    }
    $saveDialog = Wait-Dialog '保存查询结果' $application.Id
    $saveFrame = if ($demoEnabled) { '10-save-dialog.png' } else { '' }
    Complete-FileDialog $saveDialog $outputCsv $saveFrame
    for ($attempt = 0; $attempt -lt 200; $attempt++) {
        $application.Refresh()
        if ($application.HasExited) {
            throw "The application exited while processing the CSV (exit code $($application.ExitCode))."
        }
        if ((Test-Path -LiteralPath $outputCsv) -and
            [NativeWindow]::IsWindowVisible($exportControl) -and
            [NativeWindow]::IsWindowEnabled($exportControl) -and
            (Test-ClearStatus $progressControl)) { break }
        Start-Sleep -Milliseconds 100
    }
    if (-not (Test-Path -LiteralPath $outputCsv)) {
        throw "The output CSV did not appear within 20 seconds. Progress: $(Read-WindowText $progressControl)"
    }
    Assert-BatchReady $exportControl $progressControl
    $exportSaved = Read-WindowRect $exportControl
    if ($exportSaved.left -ne $exportReady.left -or $exportSaved.top -ne $exportReady.top -or
        $exportSaved.right -ne $exportReady.right -or $exportSaved.bottom -ne $exportReady.bottom) {
        throw 'The export button moved or changed size after saving.'
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
        Save-DemoFrame $window '11-batch-complete.png'
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
