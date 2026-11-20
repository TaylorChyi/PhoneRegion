# 手机号归属地 · 离线查询

在 Windows 10 或 11（x64）查询中国大陆手机号码的原号段运营商、省份和地级市。号段库随程序内置，断网可用，无需安装 Excel。输入的号码在本机处理，程序不上传号码或保存查询历史。

**[下载 Windows 程序](https://github.com/TaylorChyi/PhoneRegion/raw/refs/heads/main/download/PhoneRegion.exe)**。下载后双击 `PhoneRegion.exe`。也可以将这一个文件复制到离线电脑。

## 单个号码

输完或粘贴一个完整号码，结果直接显示在窗口中。也可以按 Enter 或点「查询」。例如，`+86 132 1300 0000` 会显示「中国联通 · 河南省 · 郑州市」。支持 11 位号码、`+86`、`0086`、13 位纯数字的 `86` 前缀、全角数字，以及常见分组空格、括号和短横线。

## 批量查询

1. 点「导入 Excel / CSV」，选择 `.xlsx`、`.xls` 或 `.csv` 文件。程序只取 Excel 第一张工作表的 A 列，或 CSV 的第一列。常见表头（如「手机号」）可以保留。
2. 选择结果文件的位置。Excel 默认另存为 `.xlsx`，CSV 默认另存为 `.csv`。也可在保存窗口中选择另一种输出格式。
3. 在结果文件中查看四列。程序保留号码的顺序和重复项。若一个单元格里有多个以空白分隔的完整号码，会依次展开为多行。

| 手机号 | 运营商 | 归属地省份 | 归属地地级市 |
| --- | --- | --- | --- |
| 13213000000 | 中国联通 | 河南省 | 郑州市 |

有效号码统一输出为 11 位。无效输入和未收录号段的后三列为空，窗口分别统计数量。CSV 空行和 Excel 首列已用范围内的空行会保留，结果另存为新文件。建议用 `.xlsx` 保持手机号文本格式。CSV 输出使用带 BOM 的 UTF-8；CSV 输入支持 UTF-8、GBK / GB18030 和带 BOM 的 UTF-16。默认按逗号分列，也识别 Excel 在首行声明的分号或制表符分隔。

## 数据口径

内置数据截至 **2026-10-22**，共 **522,410 个七位号段**。运营商反映号码最初分配的号段，携号转网后可能已变更。归属地是号段登记地区，无法判断持机人当前位置。

地市列使用地级市、自治州、地区、盟的规范名称；直辖市填该市。其中 517,515 个号段可确定到地市层级。其余 4,895 个号段只填省份，地市列留空。[数据版本、来源和许可](data/SOURCE.md)

## 从源码构建

Windows x64 的 Rust stable（MSVC 工具链）及 Visual Studio C++ 构建工具可以复建程序。首次构建需要获取锁定的 Rust 依赖。数据文件已随源码提供。

```powershell
cargo test --locked --all-targets
cargo build --release --locked
.\target\release\PhoneRegion.exe
```

在 Windows 上运行 `./scripts/windows-smoke.ps1 -ExePath ./download/PhoneRegion.exe` 可验证单号界面和批量导出。Windows 打包时使用的第三方许可可在 [许可压缩包](windows/third-party-notices.zip) 中查看，同一压缩包嵌入程序资源 101。
