<p align="center"><img src="docs/icon.png" width="72" height="72" alt="PhoneRegion 图标"></p>

<h1 align="center">PhoneRegion</h1>

<p align="center"><strong>中国大陆手机号归属地查询</strong><br>Offline phone number lookup for mainland China</p>

<p align="center">PhoneRegion 是 Windows 离线手机号段查询工具。输入一个中国大陆手机号，即可按前七位查到原号段运营商、省份和地市。也可以批量处理 Excel 或 CSV 号码清单，为每个号码补上这三个字段。</p>

<p align="center"><strong>简体中文</strong> · <a href="README.en.md">English</a> · <a href="#agent-reference">AI agent 使用入口</a></p>

<p align="center">
  <a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest"><img alt="最新版本" src="https://img.shields.io/github/v/release/TaylorChyi/PhoneRegion?label=Release&amp;color=176b50"></a>
  <img alt="Windows 10 和 11 x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20%C2%B7%20x64-0969da">
  <img alt="完全离线查询" src="https://img.shields.io/badge/%E6%9F%A5%E8%AF%A2-%E5%AE%8C%E5%85%A8%E7%A6%BB%E7%BA%BF-176b50">
  <img alt="免安装，单个 EXE 文件" src="https://img.shields.io/badge/%E4%BE%BF%E6%90%BA%E7%89%88-%E5%8D%95%E6%96%87%E4%BB%B6%20EXE-556474">
</p>

<p align="center">
  <strong>原号段运营商</strong><br>
  <img alt="原号段运营商 · 中国移动" src="https://img.shields.io/badge/%E4%B8%AD%E5%9B%BD%E7%A7%BB%E5%8A%A8-16669b">
  <img alt="原号段运营商 · 中国联通" src="https://img.shields.io/badge/%E4%B8%AD%E5%9B%BD%E8%81%94%E9%80%9A-b42332">
  <img alt="原号段运营商 · 中国电信" src="https://img.shields.io/badge/%E4%B8%AD%E5%9B%BD%E7%94%B5%E4%BF%A1-194a97">
  <img alt="原号段运营商 · 中国广电" src="https://img.shields.io/badge/%E4%B8%AD%E5%9B%BD%E5%B9%BF%E7%94%B5-74418b">
</p>

<p align="center">
  <a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest/download/PhoneRegion.exe"><img alt="下载 Windows 版 PhoneRegion.exe" src="https://img.shields.io/badge/%E4%B8%8B%E8%BD%BD%20Windows%20%E7%89%88-PhoneRegion.exe-176b50?style=for-the-badge"></a>
</p>

<p align="center">双击即可运行。查询和导出均在本机完成，程序不上传号码或表格，也不保存查询历史。 <a href="https://github.com/TaylorChyi/PhoneRegion/releases">所有版本</a></p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/window-dark.png">
    <img src="docs/window-light.png" width="880" alt="PhoneRegion 中的示例号码，查询结果为中国联通、河南省、郑州市">
  </picture>
</p>

<p align="center">界面跟随 Windows 的浅色、深色和高对比度设置。<br><a href="docs/window-light.png">浅色原图</a> · <a href="docs/window-dark.png">深色原图</a></p>

<h2 align="center">查询单个号码</h2>

<p align="center">粘贴 <code>+86 132 1300 0000</code>，即可看到<strong>中国联通 · 河南省 · 郑州市</strong>。<br>支持 <code>+86</code>、<code>0086</code> 和 <code>86</code> 前缀，也能识别全角数字、空格、括号和号码内的短横线。</p>

<h2 align="center">批量查询与导出</h2>

<p align="center">
  <img alt="导入 XLS、XLSX 或 CSV" src="https://img.shields.io/badge/%E5%AF%BC%E5%85%A5-XLS%20%C2%B7%20XLSX%20%C2%B7%20CSV-67539b">
  <img alt="导出 XLSX 或 CSV" src="https://img.shields.io/badge/%E5%AF%BC%E5%87%BA-XLSX%20%C2%B7%20CSV-67539b">
</p>

<p align="center"><a href="https://github.com/TaylorChyi/PhoneRegion/releases/download/v1.3.2/example.csv"><strong>下载示例 CSV</strong></a>，或点击拖入框上方的<strong>「保存示例 CSV」</strong>按钮，离线获取同一份示例文件。</p>

<p align="center">把文件拖入<strong>「点击或拖入文件」</strong>框，或点击该框选择文件。Excel 读取首张工作表 A 列，CSV 读取首列，可保留「手机号」等表头。电脑无需安装 Excel。<br>一个单元格中有多个完整号码时，用空格隔开即可逐个查询。</p>

<p align="center">查询完成后，点击拖入框下方的<strong>「导出结果」</strong>，选择格式和保存位置。有效号码统一为 11 位，保留原顺序和重复项。可重复导出，原表保持不变。</p>

<table align="center">
  <thead><tr><th align="center">手机号</th><th align="center">运营商</th><th align="center">归属地省份</th><th align="center">归属地地级市</th></tr></thead>
  <tbody>
    <tr><td align="center">13213000000</td><td align="center">中国联通</td><td align="center">河南省</td><td align="center">郑州市</td></tr>
    <tr><td align="center">13900200000</td><td align="center">中国移动</td><td align="center">广东省</td><td align="center">广州市</td></tr>
    <tr><td align="center">18901600000</td><td align="center">中国电信</td><td align="center">上海市</td><td align="center">上海市</td></tr>
  </tbody>
</table>

<h2 align="center">操作演示</h2>

<p align="center"><img src="docs/demo.gif" width="880" alt="在 Windows 上查询带国家码的号码，导入 CSV 完成查询后点击导出结果"></p>

<h2 align="center">数据覆盖与结果说明</h2>

<p align="center">
  <a href="data/SOURCE.md"><img alt="数据截至 2026 年 10 月 22 日，查看数据来源" src="https://img.shields.io/badge/%E6%95%B0%E6%8D%AE%E6%97%A5%E6%9C%9F-2026--10--22-546374"></a>
  <img alt="收录 522,410 个七位号段" src="https://img.shields.io/badge/%E4%B8%83%E4%BD%8D%E5%8F%B7%E6%AE%B5-522%2C410-315579">
  <img alt="517,515 个号段可确定到地市" src="https://img.shields.io/badge/%E7%B2%BE%E7%A1%AE%E5%88%B0%E5%9C%B0%E5%B8%82-517%2C515-176b50">
  <img alt="4,895 个号段仅能确定省份" src="https://img.shields.io/badge/%E4%BB%85%E7%9C%81%E4%BB%BD-4%2C895-596574">
</p>

<p align="center">运营商、省份和地市均按号码的原始号段分配。携号转网后，当前运营商可能不同。号段无法反映手机的实时位置。收录的虚拟运营商品牌会与基础运营商一同显示。</p>

<p align="center">地市列显示地级市、自治州、地区、盟或直辖市的规范名称。只能确定省份时，地市留空。格式错误或未收录的号码仍保留在导出结果中，后三列留空。 <a href="data/SOURCE.md">数据来源</a></p>

<a name="agent-reference"></a>
<h2 align="center">AI agent 使用入口</h2>

<p align="center">例如，用户想把一张只有手机号的 CSV 按省份汇总。能操作 Windows 桌面的 agent 可以在 PhoneRegion 窗口导入号码清单，导出结果后按「归属地省份」分组。</p>

<table align="center">
  <thead><tr><th>任务</th><th>入口与用法</th></tr></thead>
  <tbody>
    <tr><td>识别或引用项目</td><td>GitHub 仓库为 <a href="https://github.com/TaylorChyi/PhoneRegion"><code>TaylorChyi/PhoneRegion</code></a>。引用特定版本时，使用该版本 Release 元数据中的 <code>html_url</code>。</td></tr>
    <tr><td>获取发布版</td><td>读取 <a href="https://api.github.com/repos/TaylorChyi/PhoneRegion/releases/latest">最新 Release 元数据（JSON）</a>。<code>tag_name</code> 是版本号，在 <code>assets</code> 中按名称选择 <code>PhoneRegion.exe</code>，从其 <code>browser_download_url</code> 下载。</td></tr>
    <tr><td>验证批量查询</td><td>导入<a href="docs/example.csv">仓库中的示例 CSV</a>。首条数据 <code>132 1300 0000</code> 导出后应为 <code>13213000000,中国联通,河南省,郑州市</code>。</td></tr>
    <tr><td>读取导出结果</td><td>CSV 使用带 BOM 的 UTF-8，表头固定为 <code>手机号,运营商,归属地省份,归属地地级市</code>。读取时将手机号作为文本，并保留空字段。<a href="src/batch.rs">导入与导出实现</a></td></tr>
  </tbody>
</table>
