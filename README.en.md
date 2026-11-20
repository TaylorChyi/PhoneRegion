<p align="center"><img src="docs/icon.png" width="72" height="72" alt="PhoneRegion icon"></p>

<h1 align="center">PhoneRegion</h1>

<p align="center"><strong>Offline mobile number lookup for mainland China</strong></p>

<p align="center"><a href="README.md">简体中文</a> · <strong>English</strong></p>

<p align="center">
  <a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/TaylorChyi/PhoneRegion?label=Release&amp;color=176b50"></a>
  <img alt="Windows 10 and 11 x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20%C2%B7%20x64-0969da">
  <img alt="Fully offline" src="https://img.shields.io/badge/Lookup-Fully%20offline-176b50">
  <img alt="Single EXE, no installation required" src="https://img.shields.io/badge/Portable-Single%20EXE-556474">
</p>

<p align="center">
  <strong>Original network operators</strong><br>
  <img alt="Original network operators · China Mobile" src="https://img.shields.io/badge/China%20Mobile-16669b">
  <img alt="Original network operators · China Unicom" src="https://img.shields.io/badge/China%20Unicom-b42332">
  <img alt="Original network operators · China Telecom" src="https://img.shields.io/badge/China%20Telecom-194a97">
  <img alt="Original network operators · China Broadnet" src="https://img.shields.io/badge/China%20Broadnet-74418b">
</p>

<p align="center">
  <a href="https://github.com/TaylorChyi/PhoneRegion/releases/latest/download/PhoneRegion.exe"><img alt="Download PhoneRegion.exe for Windows" src="https://img.shields.io/badge/Download%20for%20Windows-PhoneRegion.exe-176b50?style=for-the-badge"></a>
</p>

<p align="center">Double-click to run. Numbers and files stay on your PC during lookup and export. The app keeps no search history. The interface and export headings are in Simplified Chinese. <a href="https://github.com/TaylorChyi/PhoneRegion/releases">All releases</a></p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/window-dark.png">
    <img src="docs/window-light.png" width="880" alt="PhoneRegion showing the sample number registered to China Unicom in Zhengzhou, Henan Province">
  </picture>
</p>

<p align="center">The app follows your Windows light, dark or high-contrast settings.<br><a href="docs/window-light.png">Full-size light view</a> · <a href="docs/window-dark.png">Full-size dark view</a></p>

<h2 align="center">Look up one number</h2>

<p align="center">Paste <code>+86 132 1300 0000</code> to see <strong>China Unicom · Henan Province · Zhengzhou City</strong>.<br>You can use <code>+86</code>, <code>0086</code> or <code>86</code> prefixes, full-width digits, spaces, brackets and hyphens within the number.</p>

<h2 align="center">Look up and export a list</h2>

<p align="center">
  <img alt="Import XLS, XLSX or CSV" src="https://img.shields.io/badge/Import-XLS%20%C2%B7%20XLSX%20%C2%B7%20CSV-67539b">
  <img alt="Export XLSX or CSV" src="https://img.shields.io/badge/Export-XLSX%20%C2%B7%20CSV-67539b">
</p>

<p align="center"><a href="https://github.com/TaylorChyi/PhoneRegion/releases/download/v1.3.2/example.csv"><strong>Download a sample CSV</strong></a>, or click <strong>保存示例 CSV (Save sample CSV)</strong> above the drop area to get the same file offline.</p>

<p align="center">Drop the file onto <strong>点击或拖入文件 (Click or drop a file)</strong>, or click the box to browse. The app reads column A of the first worksheet or the first CSV column. A heading such as 手机号 (mobile number) is optional. Excel does not need to be installed.<br>Separate complete numbers in the same cell with spaces to look up each one.</p>

<p align="center">After lookup, click <strong>导出结果 (Export results)</strong> below the drop area and choose a format and location. Valid numbers are normalised to 11 digits, with order and duplicates preserved. You can export again. The original file stays intact.</p>

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

<h2 align="center">Data coverage and interpretation</h2>

<p align="center">
  <a href="data/SOURCE.md"><img alt="Dataset dated 22 October 2026. View data source" src="https://img.shields.io/badge/Data%20as%20of-2026--10--22-546374"></a>
  <img alt="522,410 seven-digit prefixes" src="https://img.shields.io/badge/7--digit%20prefixes-522%2C410-315579">
  <img alt="517,515 prefixes with a known prefecture-level area" src="https://img.shields.io/badge/Prefecture%20known-517%2C515-176b50">
  <img alt="4,895 prefixes with province only" src="https://img.shields.io/badge/Province%20only-4%2C895-596574">
</p>

<p align="center">The carrier, province and prefecture reflect the original prefix allocation. The current carrier may differ after number porting. A prefix cannot identify a phone’s current location. Where available, the virtual operator’s brand appears alongside the underlying carrier.</p>

<p align="center">The area field uses the official name of the city, autonomous prefecture, prefecture, league or municipality. It stays blank when only the province is known. Invalid numbers and unlisted prefixes remain in the export with all three lookup fields blank. <a href="data/SOURCE.md">Data source</a></p>
