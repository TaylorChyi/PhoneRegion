<p align="center"><img src="docs/icon.png" width="72" height="72" alt="PhoneRegion icon"></p>

<h1 align="center">PhoneRegion</h1>

<p align="center"><strong>Offline mobile number lookup for mainland China</strong></p>

<p align="center"><a href="README.md">简体中文</a> · <strong>English</strong></p>

<p align="center">
  <a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/TaylorChyi/PhoneRegion?label=Release&amp;color=176b50"></a>
  <img alt="Windows 10 and 11 x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20%C2%B7%20x64-0969da">
  <img alt="Fully offline" src="https://img.shields.io/badge/Lookup-Fully%20offline-176b50">
  <img alt="Excel and CSV batch lookup" src="https://img.shields.io/badge/Batch-Excel%20%2F%20CSV-67539b">
</p>

<p align="center">Enter a mainland Chinese mobile number to see its <strong>original carrier, province and prefecture-level area</strong><br>right in the window. Import a whole list to look up the numbers, then export the results when ready, even without an internet connection.</p>

<h3 align="center"><a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest/download/PhoneRegion.exe">⬇ Download for Windows · PhoneRegion.exe</a></h3>

<p align="center">Double-click to start. No installation, account or configuration is required. The interface and exported headings are in Simplified Chinese. <a href="https://github.com/TaylorChyi/PhoneRegion/releases">All releases</a></p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/window-dark.png">
    <img src="docs/window-light.png" width="880" alt="PhoneRegion showing the sample number registered to China Unicom in Zhengzhou, Henan Province">
  </picture>
</p>

<p align="center"><a href="docs/window-light.png">Full-size light view</a> · <a href="docs/window-dark.png">Full-size dark view</a></p>

<h2 align="center">Look up one number instantly</h2>

<p align="center">The carrier, province and prefecture-level area fields are visible as soon as the window opens, each showing a dash until a result is available.<br>Paste <code>+86 132 1300 0000</code> to see <strong>China Unicom · Henan Province · Zhengzhou City</strong>.</p>

<p align="center">Use <code>+86</code>, <code>0086</code> or <code>86</code> prefixes, full-width digits, spaces, brackets and hyphens within the number.</p>

<h2 align="center">Look up an entire file</h2>

<p align="center"><a href="https://github.com/TaylorChyi/PhoneRegion/releases/download/v1.3.1/example.csv"><strong>Download a sample CSV</strong></a>, or click <strong>保存示例 CSV (Save sample CSV)</strong> above the drop area to get the same file offline.<br>It contains three number formats. For a single-number lookup, try <code>+86 132 1300 0000</code>.</p>

<p align="center">Click the <strong>点击或拖入文件 (Click or drop a file)</strong> box or drag one Excel (<code>.xls</code>, <code>.xlsx</code>) or CSV file onto it. Parsing and lookup start immediately.<br>Put the numbers in the first column and leave other columns as they are. Excel does not need to be installed.<br>The app reads column A of the first worksheet or the first CSV column. A common heading such as 手机号 (mobile number) is optional.</p>

<p align="center">Several complete numbers separated by spaces in one cell produce separate result rows.<br>The <strong>导出结果 (Export results)</strong> button stays below the drop area and becomes available when there are rows to export. Click it to choose a location and Excel or CSV format.<br>You can export again after saving or cancelling. The original file stays intact. Valid numbers are normalised to 11 digits, with the original order and duplicates preserved.</p>

<table align="center">
  <thead><tr><th align="center">手机号<br><sub>Mobile number</sub></th><th align="center">运营商<br><sub>Carrier</sub></th><th align="center">归属地省份<br><sub>Province</sub></th><th align="center">归属地地级市<br><sub>Prefecture-level area</sub></th></tr></thead>
  <tbody>
    <tr><td align="center">13213000000</td><td align="center">中国联通</td><td align="center">河南省</td><td align="center">郑州市</td></tr>
    <tr><td align="center">13900200000</td><td align="center">中国移动</td><td align="center">广东省</td><td align="center">广州市</td></tr>
    <tr><td align="center">18901600000</td><td align="center">中国电信</td><td align="center">上海市</td><td align="center">上海市</td></tr>
  </tbody>
</table>

<h2 align="center">See it in action</h2>

<p align="center"><img src="docs/demo.gif" width="880" alt="Windows demo showing a formatted number lookup, CSV import and the Export results button"></p>

<h2 align="center">Fully offline, with numbers kept on your PC</h2>

<p align="center">Lookup data is included. Queries and exports run locally. The app does not upload your numbers or files and keeps no search history.<br>Copy the single <code>PhoneRegion.exe</code> file to a USB drive or another Windows 10 or 11 x64 PC and double-click it there.<br>The interface follows Windows light, dark and high-contrast settings.</p>

<h2 align="center">Understanding the result</h2>

<p align="center">The included dataset is dated <strong>22 October 2026</strong> and contains <strong>522,410</strong> seven-digit prefixes.<br><strong>517,515</strong> have a known prefecture-level area. For the remaining <strong>4,895</strong>, the province is shown and the city field is blank.<br>Area names cover cities, autonomous prefectures, prefectures, leagues and municipalities in their official form.</p>

<p align="center">Carrier and area describe the original allocation of the number prefix. Coverage includes China Mobile, China Unicom, China Telecom and China Broadnet.<br>Where the data includes a virtual operator, its brand appears alongside the underlying carrier. The current carrier may differ after a number has been ported.<br>A prefix cannot tell you where a person or phone is now. Invalid inputs and unlisted prefixes remain in the export with the three lookup fields blank. <a href="data/SOURCE.md">Data source</a></p>

<p align="center"><sub>mainland China mobile numbers · original carrier · prefecture-level city · offline phone number lookup · Excel and CSV batch lookup</sub></p>
