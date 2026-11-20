use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::path::Path;

use calamine::{Data, DataRef, Reader, SheetType, Xlsx, open_workbook_auto};
use encoding_rs::{GB18030, UTF_16BE, UTF_16LE};
use rust_xlsxwriter::{Format, Workbook};
use tempfile::NamedTempFile;

use crate::lookup::Database;

const HEADERS: [&str; 4] = ["手机号", "运营商", "归属地省份", "归属地地级市"];

pub struct Prepared {
    rows: Vec<[String; 4]>,
}

impl Prepared {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

pub fn prepare(input: &Path, db: &Database) -> Result<Prepared, String> {
    if !input.is_file() {
        return Err("找不到输入文件".into());
    }
    let source_type = extension(input);
    if !matches!(source_type.as_str(), "xls" | "xlsx" | "csv") {
        return Err("输入文件须为 .xls、.xlsx 或 .csv".into());
    }

    let source_rows = if source_type == "csv" {
        read_csv(input)?
    } else if source_type == "xlsx" {
        read_xlsx(input)?
    } else {
        read_excel(input)?
    };
    let header_index = source_rows
        .iter()
        .position(|value| !value.trim().is_empty());
    let mut rows = Vec::with_capacity(source_rows.len());
    for (index, value) in source_rows.into_iter().enumerate() {
        if Some(index) == header_index && is_header(&value) {
            continue;
        }
        for number in crate::numbers::batch_values(&value) {
            let source = match scientific_phone(number) {
                Some(phone) => Cow::Owned(phone),
                None => Cow::Borrowed(number),
            };
            let lookup = db.lookup(&source);
            rows.push([lookup.phone, lookup.carrier, lookup.province, lookup.city]);
        }
    }
    Ok(Prepared { rows })
}

pub fn export_prepared(prepared: &Prepared, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err("输出文件已存在，请选择另一个文件名".into());
    }
    let target_type = extension(output);
    if !matches!(target_type.as_str(), "xlsx" | "csv") {
        return Err("输出文件须为 .xlsx 或 .csv".into());
    }

    let directory = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !directory.is_dir() {
        return Err("输出文件夹不存在".into());
    }
    let mut temporary = NamedTempFile::new_in(directory)
        .map_err(|error| format!("无法在输出文件夹创建文件：{error}"))?;
    if target_type == "csv" {
        write_csv(&mut temporary, &prepared.rows)?;
    } else {
        write_excel(&mut temporary, &prepared.rows)?;
    }
    temporary
        .persist_noclobber(output)
        .map_err(|error| format!("无法保存输出文件：{}", error.error))?;
    Ok(())
}

#[cfg(test)]
fn convert(input: &Path, output: &Path, db: &Database) -> Result<(), String> {
    let prepared = prepare(input, db)?;
    export_prepared(&prepared, output)
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn is_header(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "phone" | "mobile" | "number"
    ) || matches!(
        value.trim(),
        "手机号" | "手机号码" | "手机" | "号码" | "电话" | "电话号码"
    )
}

fn read_csv(path: &Path) -> Result<Vec<String>, String> {
    let bytes = fs::read(path).map_err(|error| format!("无法读取 CSV：{error}"))?;
    let (mut decoded, had_errors) = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text = String::from_utf8(bytes[3..].to_vec())
            .map_err(|_| "CSV 的 UTF-8 内容无效".to_string())?;
        (text, false)
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        let (text, _, errors) = UTF_16LE.decode(&bytes[2..]);
        (text.into_owned(), errors)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        let (text, _, errors) = UTF_16BE.decode(&bytes[2..]);
        (text.into_owned(), errors)
    } else if let Ok(text) = String::from_utf8(bytes.clone()) {
        (text, false)
    } else {
        let (text, _, errors) = GB18030.decode(&bytes);
        (text.into_owned(), errors)
    };
    if had_errors {
        return Err("CSV 编码无法识别，请转成 UTF-8 或 GB18030".into());
    }
    let mut delimiter = b',';
    let first_line_end = decoded
        .find('\n')
        .map(|position| position + 1)
        .unwrap_or(decoded.len());
    let directive = decoded[..first_line_end].trim_end_matches(['\r', '\n']);
    let declared = if directive.eq_ignore_ascii_case("sep=;") {
        Some(b';')
    } else if directive.eq_ignore_ascii_case("sep=\t")
        || directive.eq_ignore_ascii_case("sep=\\t")
        || directive.eq_ignore_ascii_case("sep=tab")
    {
        Some(b'\t')
    } else if directive.eq_ignore_ascii_case("sep=,") {
        Some(b',')
    } else {
        None
    };
    if let Some(value) = declared {
        delimiter = value;
        decoded = decoded[first_line_end..].to_owned();
    }
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(decoded.as_bytes());
    let mut rows = Vec::new();
    let mut record = csv::StringRecord::new();
    let mut previous_end = 0usize;
    while reader
        .read_record(&mut record)
        .map_err(|error| format!("CSV 第 {} 行格式错误：{error}", rows.len() + 1))?
    {
        let next_end = reader.position().byte() as usize;
        rows.extend(std::iter::repeat_n(
            String::new(),
            leading_blank_lines(&decoded, previous_end, next_end),
        ));
        rows.push(record.get(0).unwrap_or_default().to_owned());
        previous_end = next_end;
    }
    rows.extend(std::iter::repeat_n(
        String::new(),
        leading_blank_lines(&decoded, previous_end, decoded.len()),
    ));
    Ok(rows)
}

fn leading_blank_lines(text: &str, start: usize, end: usize) -> usize {
    let bytes = text.as_bytes();
    let mut index = start;
    if start > 0 && bytes[start - 1] == b'\r' && bytes.get(start) == Some(&b'\n') {
        index += 1;
    }
    let mut count = 0;
    while index < end {
        match bytes[index] {
            b'\n' => index += 1,
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => index += 2,
            b'\r' => index += 1,
            _ => break,
        }
        count += 1;
    }
    count
}

fn read_excel(path: &Path) -> Result<Vec<String>, String> {
    let mut workbook =
        open_workbook_auto(path).map_err(|error| format!("无法打开 Excel：{error}"))?;
    let sheet = workbook
        .sheets_metadata()
        .iter()
        .find(|sheet| sheet.typ == SheetType::WorkSheet)
        .map(|sheet| sheet.name.clone())
        .ok_or("Excel 文件没有工作表")?;
    let range = workbook
        .worksheet_range(&sheet)
        .map_err(|error| format!("无法读取首张工作表：{error}"))?;
    let (first_row, first_column) = range.start().unwrap_or((0, 0));
    let first_column_present = first_column == 0;
    let mut rows = vec![String::new(); first_row as usize];
    rows.extend(range.rows().map(|row| {
        if first_column_present {
            row.first().map(cell_to_string).unwrap_or_default()
        } else {
            String::new()
        }
    }));
    Ok(rows)
}

fn read_xlsx(path: &Path) -> Result<Vec<String>, String> {
    let mut workbook: Xlsx<_> =
        calamine::open_workbook(path).map_err(|error| format!("无法打开 Excel：{error}"))?;
    let sheet = workbook
        .sheets_metadata()
        .iter()
        .find(|sheet| sheet.typ == SheetType::WorkSheet)
        .map(|sheet| sheet.name.clone())
        .ok_or("Excel 文件没有工作表")?;
    let mut cells = workbook
        .worksheet_cells_reader(&sheet)
        .map_err(|error| format!("无法读取首张工作表：{error}"))?;
    let mut rows = Vec::new();
    while let Some(cell) = cells
        .next_cell()
        .map_err(|error| format!("Excel 单元格读取失败：{error}"))?
    {
        let (row, column) = cell.get_position();
        if column != 0 {
            continue;
        }
        let index = row as usize;
        if rows.len() <= index {
            rows.resize(index + 1, String::new());
        }
        rows[index] = cell_ref_to_string(cell.get_value());
    }
    Ok(rows)
}

fn cell_to_string(value: &Data) -> String {
    match value {
        Data::Int(number) => number.to_string(),
        Data::Float(number) if number.is_finite() && number.fract() == 0.0 => {
            format!("{number:.0}")
        }
        Data::String(text) => text.clone(),
        Data::Empty => String::new(),
        _ => value.to_string(),
    }
}

fn cell_ref_to_string(value: &DataRef<'_>) -> String {
    match value {
        DataRef::Int(number) => number.to_string(),
        DataRef::Float(number) if number.is_finite() && number.fract() == 0.0 => {
            format!("{number:.0}")
        }
        DataRef::String(text) => text.clone(),
        DataRef::SharedString(text) => text.to_string(),
        DataRef::Bool(value) => value.to_string(),
        DataRef::DateTime(value) => value.to_string(),
        DataRef::DateTimeIso(text) | DataRef::DurationIso(text) => text.clone(),
        DataRef::Error(error) => format!("{error:?}"),
        DataRef::Empty => String::new(),
        DataRef::Float(number) => number.to_string(),
    }
}

fn scientific_phone(value: &str) -> Option<String> {
    let value = value.trim();
    let exponent_mark = value.find(['e', 'E'])?;
    let (mantissa, exponent) = value.split_at(exponent_mark);
    let exponent: i32 = exponent.get(1..)?.parse().ok()?;
    if !(-20..=20).contains(&exponent) {
        return None;
    }
    let (integer, fractional) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let decimal_position = i32::try_from(integer.len()).ok()? + exponent;
    if decimal_position != 11 {
        return None;
    }
    let digits = format!("{integer}{fractional}");
    if digits.len() < 11 || !digits[11..].bytes().all(|byte| byte == b'0') {
        return None;
    }
    Some(digits[..11].to_owned())
}

// Excel evaluates some CSV cell prefixes as formulae even when the CSV field is quoted.
fn export_safe_phone(phone: &str) -> String {
    let first_visible = phone.trim_start().chars().next();
    if first_visible.is_some_and(|first| matches!(first, '=' | '+' | '-' | '@')) {
        format!("'{phone}")
    } else {
        phone.to_owned()
    }
}

fn write_csv(temporary: &mut NamedTempFile, rows: &[[String; 4]]) -> Result<(), String> {
    temporary
        .write_all(&[0xEF, 0xBB, 0xBF])
        .map_err(|error| format!("CSV 写入失败：{error}"))?;
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(temporary.as_file_mut());
    writer
        .write_record(HEADERS)
        .map_err(|error| format!("CSV 写入失败：{error}"))?;
    for row in rows {
        let safe_row = [
            export_safe_phone(&row[0]),
            row[1].clone(),
            row[2].clone(),
            row[3].clone(),
        ];
        writer
            .write_record(&safe_row)
            .map_err(|error| format!("CSV 写入失败：{error}"))?;
    }
    writer
        .flush()
        .map_err(|error| format!("CSV 写入失败：{error}"))?;
    Ok(())
}

fn write_excel(temporary: &mut NamedTempFile, rows: &[[String; 4]]) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let blank_format = Format::new().set_num_format("@");
    for (column, header) in HEADERS.iter().enumerate() {
        worksheet
            .write_string(0, column as u16, *header)
            .map_err(|error| format!("Excel 写入失败：{error}"))?;
    }
    for (row_index, row) in rows.iter().enumerate() {
        let excel_row =
            u32::try_from(row_index + 1).map_err(|_| "Excel 超过最大行数".to_string())?;
        if excel_row >= 1_048_576 {
            return Err("Excel 超过最大行数，请改用 CSV 输出".into());
        }
        if row.iter().all(String::is_empty) {
            worksheet
                .write_blank(excel_row, 0, &blank_format)
                .map_err(|error| format!("Excel 写入失败：{error}"))?;
            continue;
        }
        for (column, value) in row.iter().enumerate() {
            worksheet
                .write_string(excel_row, column as u16, value)
                .map_err(|error| format!("Excel 写入失败：{error}"))?;
        }
    }
    worksheet
        .set_column_width(0, 20)
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    worksheet
        .set_column_width(1, 16)
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    worksheet
        .set_column_width(2, 16)
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    worksheet
        .set_column_width(3, 20)
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    let bytes = workbook
        .save_to_buffer()
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    temporary
        .write_all(&bytes)
        .map_err(|error| format!("Excel 写入失败：{error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn formula_prefixes_are_escaped() {
        assert_eq!(export_safe_phone("=1+1"), "'=1+1");
        assert_eq!(export_safe_phone("+8613812345678"), "'+8613812345678");
        assert_eq!(export_safe_phone("  =1+1"), "'  =1+1");
        assert_eq!(export_safe_phone("13812345678"), "13812345678");
    }

    #[test]
    fn header_detection_is_limited_to_first_cell_names() {
        assert!(is_header(" 手机号 "));
        assert!(is_header("PHONE"));
        assert!(!is_header("13812345678"));
    }

    #[test]
    fn scientific_notation_requires_all_eleven_significant_digits() {
        assert_eq!(
            scientific_phone("1.3213000000E+10").as_deref(),
            Some("13213000000")
        );
        assert_eq!(
            scientific_phone("13213000000.0E0").as_deref(),
            Some("13213000000")
        );
        assert_eq!(scientific_phone("1.32E+10"), None);
        assert_eq!(scientific_phone("1.32130000001E+10"), None);
        assert_eq!(scientific_phone("1.3213000000E+11"), None);
    }

    #[test]
    fn csv_read_keeps_blank_lines_and_quoted_fields() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        fs::write(&input, "手机号,备注\n\n\"13213000000\",\"跨\n行\"\n\n").unwrap();
        let rows = read_csv(&input).unwrap();
        assert_eq!(rows, ["手机号", "", "13213000000", ""]);
    }

    #[test]
    fn csv_read_accepts_utf8_bom_and_gb18030() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        fs::write(
            &input,
            [
                b"\xEF\xBB\xBF".as_slice(),
                "手机号\n13213000000\n".as_bytes(),
            ]
            .concat(),
        )
        .unwrap();
        assert_eq!(read_csv(&input).unwrap(), ["手机号", "13213000000"]);
        let (encoded, _, errors) = GB18030.encode("手机号\n13213000000\n");
        assert!(!errors);
        fs::write(&input, encoded.as_ref()).unwrap();
        assert_eq!(read_csv(&input).unwrap(), ["手机号", "13213000000"]);
    }

    #[test]
    fn csv_read_keeps_crlf_blank_rows_and_quoted_newline() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        fs::write(
            &input,
            "手机号,备注\r\n\r\n\"13213000000\",\"跨\r\n行\"\r\n\r\n",
        )
        .unwrap();
        assert_eq!(read_csv(&input).unwrap(), ["手机号", "", "13213000000", ""]);
    }

    #[test]
    fn csv_honours_explicit_excel_separator_directives() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        fs::write(&input, "sep=;\r\n手机号;备注\r\n13213000000;其他\r\n").unwrap();
        assert_eq!(read_csv(&input).unwrap(), ["手机号", "13213000000"]);
        fs::write(&input, "sep=\t\n手机号\t备注\n13213000000\t其他\n").unwrap();
        assert_eq!(read_csv(&input).unwrap(), ["手机号", "13213000000"]);
    }

    #[test]
    fn csv_export_has_only_requested_columns_and_keeps_order() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.csv");
        let mut file = fs::File::create(&input).unwrap();
        file.write_all("手机号,忽略\n13213000000,a\n13213000000,b\n不合法,c\n".as_bytes())
            .unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let bytes = fs::read(&output).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let mut reader = csv::Reader::from_reader(&bytes[3..]);
        assert_eq!(
            reader.headers().unwrap().iter().collect::<Vec<_>>(),
            HEADERS
        );
        let records = reader
            .records()
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].get(0), Some("13213000000"));
        assert_eq!(records[1].get(0), Some("13213000000"));
        assert!(!records[0].get(1).unwrap().is_empty());
        assert!(!records[1].get(1).unwrap().is_empty());
        assert_eq!(records[2].get(0), Some("不合法"));
        assert_eq!(records[2].get(1), Some(""));
        assert_eq!(records[2].len(), 4);
    }

    #[test]
    fn xlsx_numeric_first_column_is_read_as_phone_text() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.xlsx");
        let output = directory.path().join("output.xlsx");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "手机号").unwrap();
        sheet.write_number(1, 0, 13213000000.0).unwrap();
        sheet.write_string(1, 1, "忽略").unwrap();
        workbook.save(&input).unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let mut result = open_workbook_auto(&output).unwrap();
        let sheet = result.sheet_names()[0].clone();
        let range = result.worksheet_range(&sheet).unwrap();
        assert_eq!(range.get((1, 0)).unwrap().to_string(), "13213000000");
        assert!(!range.get((1, 1)).unwrap().to_string().is_empty());
        assert_eq!(range.get((0, 3)).unwrap().to_string(), "归属地地级市");
    }

    #[test]
    fn xlsx_preserves_leading_empty_rows_before_header() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.xlsx");
        let output = directory.path().join("output.csv");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(2, 0, "手机号").unwrap();
        sheet.write_string(3, 0, "13213000000").unwrap();
        workbook.save(&input).unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let bytes = fs::read(&output).unwrap();
        let records = csv::Reader::from_reader(&bytes[3..])
            .records()
            .map(|record| record.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert!(records[0].iter().all(str::is_empty));
        assert!(records[1].iter().all(str::is_empty));
        assert_eq!(records[2].get(0), Some("13213000000"));
        assert!(!records[2].get(1).unwrap().is_empty());
    }

    #[test]
    fn xlsx_export_keeps_trailing_empty_rows() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.xlsx");
        fs::write(&input, "手机号\n13213000000\n\n\n").unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let rows = read_xlsx(&output).unwrap();
        assert_eq!(rows, ["手机号", "13213000000", "", ""]);
    }

    #[test]
    fn xlsx_ignores_far_away_columns() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.xlsx");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "手机号").unwrap();
        sheet.write_string(1, 0, "13213000000").unwrap();
        sheet.write_string(900_000, 10_000, "无关内容").unwrap();
        workbook.save(&input).unwrap();
        assert_eq!(read_xlsx(&input).unwrap(), ["手机号", "13213000000"]);
    }

    #[test]
    fn xlsx_selects_first_actual_worksheet_after_chart_sheet() {
        use rust_xlsxwriter::{Chart, ChartType};
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.xlsx");
        let mut workbook = Workbook::new();
        let mut chart = Chart::new(ChartType::Column);
        chart.add_series().set_values("Sheet1!$A$2:$A$2");
        workbook
            .add_chartsheet()
            .insert_chart(0, 0, &chart)
            .unwrap();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "手机号").unwrap();
        sheet.write_string(1, 0, "13213000000").unwrap();
        workbook.save(&input).unwrap();
        assert_eq!(read_xlsx(&input).unwrap(), ["手机号", "13213000000"]);
    }

    #[test]
    fn existing_output_is_preserved() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.csv");
        fs::write(&input, "手机号\n13213000000\n").unwrap();
        fs::write(&output, "原有文件").unwrap();
        let database = Database::load().unwrap();
        assert!(convert(&input, &output, &database).is_err());
        assert_eq!(fs::read_to_string(&output).unwrap(), "原有文件");
    }

    #[test]
    fn prepare_keeps_source_folder_untouched_and_exports_twice() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        fs::write(&input, "手机号\n13213000000\n").unwrap();
        let database = Database::load().unwrap();
        let prepared = prepare(&input, &database).unwrap();
        assert!(!prepared.is_empty());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
        let csv_output = directory.path().join("out.csv");
        let xlsx_output = directory.path().join("out.xlsx");
        export_prepared(&prepared, &csv_output).unwrap();
        export_prepared(&prepared, &xlsx_output).unwrap();
        assert!(csv_output.is_file());
        let bytes = fs::read(&csv_output).unwrap();
        let records = csv::Reader::from_reader(&bytes[3..])
            .records()
            .map(|record| record.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0), Some("13213000000"));
        assert!(!records[0].get(1).unwrap().is_empty());
        assert_eq!(read_xlsx(&xlsx_output).unwrap()[1], "13213000000");
    }

    #[test]
    fn only_header_has_no_export_rows_but_invalid_value_does() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let database = Database::load().unwrap();
        fs::write(&input, "手机号\n").unwrap();
        assert!(prepare(&input, &database).unwrap().is_empty());
        fs::write(&input, "手机号\n无效号码\n").unwrap();
        assert!(!prepare(&input, &database).unwrap().is_empty());
    }

    #[test]
    fn one_cell_with_two_numbers_produces_two_rows() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.csv");
        fs::write(&input, "手机号\n\"13213000000、13213000001\"\n").unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let bytes = fs::read(&output).unwrap();
        let records = csv::Reader::from_reader(&bytes[3..])
            .records()
            .map(|record| record.unwrap().get(0).unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(records, ["13213000000", "13213000001"]);
    }

    #[test]
    fn xlsx_keeps_invalid_formula_like_text_verbatim() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.xlsx");
        fs::write(&input, "手机号\n=1+1\n").unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let mut result = open_workbook_auto(&output).unwrap();
        let sheet = result.sheet_names()[0].clone();
        let range = result.worksheet_range(&sheet).unwrap();
        assert_eq!(range.get((1, 0)).unwrap().to_string(), "=1+1");
    }

    #[test]
    fn genuine_biff8_xls_imports_numeric_and_multiple_numbers() {
        // The fixture is a BIFF8 workbook made with xlwt 1.3.0 from synthetic values.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/input.xls");
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("output.csv");
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let bytes = fs::read(&output).unwrap();
        let records = csv::Reader::from_reader(&bytes[3..])
            .records()
            .map(|record| record.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            records
                .iter()
                .map(|record| record.get(0).unwrap())
                .collect::<Vec<_>>(),
            ["13213000000", "13213000000", "13213000001", "无效号码"]
        );
        assert!(
            records[..3]
                .iter()
                .all(|record| !record.get(1).unwrap().is_empty())
        );
        assert!(records[3].iter().skip(1).all(str::is_empty));
    }

    #[test]
    fn scientific_text_converts_only_with_full_precision() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.csv");
        let output = directory.path().join("output.csv");
        fs::write(&input, "手机号\n1.3213000000E+10\n1.32E+10\n").unwrap();
        let database = Database::load().unwrap();
        convert(&input, &output, &database).unwrap();
        let bytes = fs::read(&output).unwrap();
        let records = csv::Reader::from_reader(&bytes[3..])
            .records()
            .map(|record| record.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get(0), Some("13213000000"));
        assert!(!records[0].get(1).unwrap().is_empty());
        assert_eq!(records[1].get(0), Some("1.32E+10"));
        assert!(records[1].iter().skip(1).all(str::is_empty));
    }
}
