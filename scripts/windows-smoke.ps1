param(
    [string]$ExePath = './target/release/PhoneRegion.exe',
    [switch]$BlockNetwork
)

$ErrorActionPreference = 'Stop'

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

    public static IntPtr FindFileNameEdit(IntPtr dialog)
    {
        IntPtr result = IntPtr.Zero;
        EnumChildWindows(dialog, (child, ignored) => {
            var className = new StringBuilder(64);
            GetClassNameW(child, className, className.Capacity);
            if (className.ToString() != "Edit") return true;
            for (IntPtr ancestor = child; ancestor != IntPtr.Zero && ancestor != dialog;
                 ancestor = GetParent(ancestor)) {
                int id = GetDlgCtrlID(ancestor);
                if (id == 1148 || id == 1152) {
                    result = child;
                    return false;
                }
            }
            return true;
        }, IntPtr.Zero);
        return result;
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

function Complete-FileDialog([IntPtr]$Dialog, [string]$Path) {
    # The common dialog creates and may replace its filename edit while initialising.
    # Wait for its value to round-trip before asking the dialog to accept it.
    for ($attempt = 0; $attempt -lt 60; $attempt++) {
        $filename = [NativeWindow]::FindFileNameEdit($Dialog)
        $accept = [NativeWindow]::GetDlgItem($Dialog, 1)
        if ($filename -ne [IntPtr]::Zero -and $accept -ne [IntPtr]::Zero) {
            [void][NativeWindow]::SendTextMessageW($filename, 0x000C, [IntPtr]::Zero, $Path)
            if ((Read-WindowText $filename) -eq $Path) {
                if (-not [NativeWindow]::PostMessageW($Dialog, 0x0111, [IntPtr]1, $accept)) {
                    throw 'The common dialog did not accept its OK command.'
                }
                return
            }
        }
        Start-Sleep -Milliseconds 150
    }
    throw "The filename edit did not retain '$Path'."
}

$source = (Resolve-Path -LiteralPath $ExePath).Path
$isolated = Join-Path ([System.IO.Path]::GetTempPath()) ('PhoneRegionSmoke-' + [guid]::NewGuid().ToString('N'))
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

    Assert-Result $inputControl $queryControl $resultControl '13213000000' @('中国联通', '河南省', '郑州市') @() -Live
    Assert-Result $inputControl $queryControl $resultControl '+86 132 1300 0000' @('中国联通', '河南省', '郑州市') @()
    Assert-Result $inputControl $queryControl $resultControl '123' @('请输入 11 位') @('中国联通', '河南省', '郑州市')

    $inputCsv = Join-Path $isolated 'phones.csv'
    $outputCsv = Join-Path $isolated 'result.csv'
    [System.IO.File]::WriteAllText(
        $inputCsv,
        "手机号`r`n13213000000`r`n+86 132 1300 0000`r`n123`r`n",
        [System.Text.UTF8Encoding]::new($true)
    )
    $importControl = [NativeWindow]::GetDlgItem($window, 1003)
    $progressControl = [NativeWindow]::GetDlgItem($window, 1005)
    if ($importControl -eq [IntPtr]::Zero -or $progressControl -eq [IntPtr]::Zero) {
        throw 'The batch import button or progress control is missing.'
    }
    if (-not [NativeWindow]::PostMessageW($importControl, 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw 'Could not open the batch file picker.'
    }
    $openDialog = Wait-Dialog '选择手机号文件' $application.Id
    Complete-FileDialog $openDialog $inputCsv
    $saveDialog = Wait-Dialog '保存查询结果' $application.Id
    Complete-FileDialog $saveDialog $outputCsv

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
    if ($records.Count -ne 3) {
        throw "Expected three output rows, found $($records.Count)."
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
    if (-not (Test-Path -LiteralPath $inputCsv)) {
        throw 'The original input CSV disappeared.'
    }
    Write-Host 'Windows GUI smoke test passed, including a standalone EXE launch.'
}
catch {
    if ($null -ne $application -and -not $application.HasExited) {
        Write-Host ([NativeWindow]::DescribeProcessWindows([uint32]$application.Id))
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
