use std::convert::TryInto;

const DATA: &[u8] = include_bytes!("../data/phone.dat");
const REGIONS: &str = include_str!("../data/regions.tsv");
const INDEX_SIZE: usize = 12;

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Found,
    Invalid,
    Missing,
    Empty,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lookup {
    pub status: Status,
    pub phone: String,
    pub carrier: String,
    pub province: String,
    pub city: String,
}

struct Entry {
    prefix: u32,
    city_id: usize,
    carrier_id: u16,
    base_carrier_id: u16,
}

pub struct Database {
    entries: Vec<Entry>,
    cities: Vec<(String, String)>,
}

impl Database {
    pub fn load() -> Result<Self, String> {
        Self::parse(DATA)
    }

    fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 36 || &data[0..4] != b"JPHN" || u16_at(data, 4)? != 3 {
            return Err("号段文件格式或版本无效".into());
        }
        let count = u32_at(data, 8)? as usize;
        let index_start = u32_at(data, 12)? as usize;
        let index_len = u32_at(data, 16)? as usize;
        let dict_start = u32_at(data, 20)? as usize;
        let dict_len = u32_at(data, 24)? as usize;
        let city_count = u32_at(data, 28)? as usize;
        if count == 0
            || index_start != 36
            || count.checked_mul(INDEX_SIZE) != Some(index_len)
            || index_start.checked_add(index_len) != Some(dict_start)
            || data.len().checked_sub(dict_start) != Some(dict_len)
            || city_count
                .checked_mul(4)
                .is_none_or(|minimum| minimum > dict_len)
        {
            return Err("号段文件长度无效".into());
        }

        let mut position = dict_start;
        let mut cities = Vec::with_capacity(city_count);
        let mut region_lines = REGIONS.lines();
        for index in 0..city_count {
            let province = read_string(data, &mut position)?;
            let city = read_string(data, &mut position)?;
            let line = region_lines.next().ok_or("标准地名表记录不足")?;
            let columns: Vec<&str> = line.split('\t').collect();
            if columns.len() != 4 || columns[0] != province || columns[1] != city {
                return Err(format!("标准地名表第 {} 条与号段库不一致", index + 1));
            }
            cities.push((columns[2].to_string(), columns[3].to_string()));
        }
        if position != data.len() || region_lines.next().is_some() {
            return Err("号段文件城市字典长度无效".into());
        }

        let mut entries = Vec::with_capacity(count);
        let mut previous = 0;
        for index in 0..count {
            let offset = index_start + index * INDEX_SIZE;
            let prefix = u32_at(data, offset)?;
            let city_id = u32_at(data, offset + 4)? as usize;
            let carrier_id = u16_at(data, offset + 8)?;
            let base_carrier_id = u16_at(data, offset + 10)?;
            if prefix <= previous
                || !(1_000_000..=1_999_999).contains(&prefix)
                || city_id >= city_count
                || carrier_name(carrier_id).is_none()
                || base_carrier_id > 4
                || (carrier_id > 4 && base_carrier_id == 0)
            {
                return Err(format!("号段文件第 {} 条索引无效", index + 1));
            }
            previous = prefix;
            entries.push(Entry {
                prefix,
                city_id,
                carrier_id,
                base_carrier_id,
            });
        }
        Ok(Self { entries, cities })
    }

    pub fn lookup(&self, input: &str) -> Lookup {
        let phone = match normalise_phone(input) {
            Ok(phone) => phone,
            Err(status) => {
                return Lookup {
                    status,
                    phone: input.to_string(),
                    carrier: String::new(),
                    province: String::new(),
                    city: String::new(),
                };
            }
        };
        let prefix: u32 = phone[..7].parse().expect("normalised digits");
        let index = match self
            .entries
            .binary_search_by_key(&prefix, |entry| entry.prefix)
        {
            Ok(index) => index,
            Err(_) => {
                return Lookup {
                    status: Status::Missing,
                    phone,
                    carrier: String::new(),
                    province: String::new(),
                    city: String::new(),
                };
            }
        };
        let entry = &self.entries[index];
        let (province, city) = &self.cities[entry.city_id];
        let base_id = if entry.carrier_id <= 4 {
            entry.carrier_id
        } else {
            entry.base_carrier_id
        };
        let base = carrier_name(base_id).expect("validated base carrier");
        let carrier =
            if (5..=59).contains(&entry.carrier_id) || (62..=64).contains(&entry.carrier_id) {
                let brand = carrier_name(entry.carrier_id).expect("validated virtual carrier");
                format!("中国{}（{}）", base, brand)
            } else {
                format!("中国{}", base)
            };
        Lookup {
            status: Status::Found,
            phone,
            carrier,
            province: province.clone(),
            city: city.clone(),
        }
    }
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes: [u8; 4] = data
        .get(offset..offset + 4)
        .ok_or("号段文件读取越界")?
        .try_into()
        .map_err(|_| "号段文件字段无效")?;
    Ok(u32::from_le_bytes(bytes))
}

fn u16_at(data: &[u8], offset: usize) -> Result<u16, String> {
    let bytes: [u8; 2] = data
        .get(offset..offset + 2)
        .ok_or("号段文件读取越界")?
        .try_into()
        .map_err(|_| "号段文件字段无效")?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_string(data: &[u8], position: &mut usize) -> Result<String, String> {
    let length = u16_at(data, *position)? as usize;
    *position += 2;
    let bytes = data
        .get(*position..*position + length)
        .ok_or("号段文件字符串越界")?;
    *position += length;
    String::from_utf8(bytes.to_vec()).map_err(|_| "号段文件字符串编码无效".into())
}

fn normalise_phone(input: &str) -> Result<String, Status> {
    let mut value = input.trim();
    if value.is_empty() {
        return Err(Status::Empty);
    }
    value = value.strip_prefix('\'').unwrap_or(value).trim_start();
    if value
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("tel:"))
    {
        value = value[4..].trim();
    }
    if value.chars().next().is_some_and(is_hyphen) || value.chars().last().is_some_and(is_hyphen) {
        return Err(Status::Invalid);
    }

    let mut compact = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '0'..='9' | '+' => compact.push(character),
            '０'..='９' => {
                compact.push(char::from_u32(character as u32 - '０' as u32 + '0' as u32).unwrap())
            }
            '＋' => compact.push('+'),
            '-' | '－' | '‐' | '‑' | '–' | '(' | ')' | '（' | '）' => (),
            c if c.is_whitespace() => (),
            _ => return Err(Status::Invalid),
        }
    }
    let digits = if let Some(rest) = compact.strip_prefix("+86") {
        rest
    } else if let Some(rest) = compact.strip_prefix("0086") {
        rest
    } else if compact.len() == 13 && compact.starts_with("86") {
        &compact[2..]
    } else {
        &compact
    };
    if digits.len() != 11
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || !digits.starts_with('1')
        || !matches!(digits.as_bytes()[1], b'3'..=b'9')
    {
        return Err(Status::Invalid);
    }
    Ok(digits.to_string())
}

fn is_hyphen(character: char) -> bool {
    matches!(character, '-' | '－' | '‐' | '‑' | '–')
}

pub(crate) fn normalize_phone(raw: &str) -> Option<String> {
    normalise_phone(raw).ok()
}

fn carrier_name(id: u16) -> Option<&'static str> {
    const NAMES: [&str; 64] = [
        "移动",
        "联通",
        "电信",
        "广电",
        "铁通",
        "天音移动",
        "小米移动",
        "星美通讯",
        "星美通信",
        "长江时代",
        "海航通信",
        "国美极信",
        "民生通讯",
        "朗玛移动",
        "迪信通",
        "京东通信",
        "分享在线",
        "话机通信",
        "话机世界",
        "用友通信",
        "三五互联",
        "联想懂的",
        "连连科技",
        "丰信通信",
        "乐语通信",
        "爱施德",
        "虚拟运营商",
        "中兴视通",
        "中邮世纪",
        "世纪互联",
        "银盛通信",
        "中邮普泰",
        "华翔联信",
        "苏宁互联",
        "鹏博士移动",
        "国美通信",
        "优友互联",
        "博元讯息",
        "阿里通信",
        "中期移动",
        "远特信时空",
        "广东恒大",
        "红豆集团",
        "星美生活",
        "网信移动",
        "分享通信",
        "无限互联",
        "日日顺通信",
        "北纬通信",
        "远特通信",
        "长城移动",
        "中麦通信",
        "巴士在线",
        "普泰移动",
        "蜗牛移动",
        "全民优打",
        "豆电信",
        "优酷移动",
        "华云互联",
        "中国移动卫星电话",
        "电信增值业务",
        "易索得移动",
        "银盛移动",
        "迪信通通信",
    ];
    id.checked_sub(1)
        .and_then(|index| NAMES.get(index as usize).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_complete_database() {
        let database = Database::load().unwrap();
        assert_eq!(database.entries.len(), 522_410);
        assert_eq!(database.cities.len(), 371);
        for entry in &database.entries {
            let number = format!("{}0000", entry.prefix);
            assert_eq!(database.lookup(&number).status, Status::Found, "{number}");
        }
        assert_eq!(database.lookup("13213000000").province, "河南省");
        assert_eq!(database.lookup("13213000000").city, "郑州市");
        assert_eq!(database.lookup("13213000000").carrier, "中国联通");
        let with_city = database
            .entries
            .iter()
            .filter(|entry| !database.cities[entry.city_id].1.is_empty())
            .count();
        assert_eq!(with_city, 517_515);
        assert_eq!(database.entries.len() - with_city, 4_895);
    }

    #[test]
    fn accepts_explicit_country_code_and_formatting() {
        let database = Database::load().unwrap();
        for input in [
            "13213000000",
            "+86 132 1300 0000",
            "0086-132-1300-0000",
            "86 132 1300 0000",
            "(+86) 132 1300 0000",
            "（＋８６）１３２　１３００　００００",
            "tel:+86\u{202f}132\u{00a0}1300-0000",
            "'13213000000",
            "'+86 13213000000",
        ] {
            let result = database.lookup(input);
            assert_eq!(result.status, Status::Found, "{input}");
            assert_eq!(result.phone, "13213000000", "{input}");
            assert_eq!(normalize_phone(input).as_deref(), Some("13213000000"));
        }
    }

    #[test]
    fn rejects_partial_and_combined_numbers() {
        let database = Database::load().unwrap();
        for input in [
            "+87 13213000000",
            "1321300000",
            "13213000000 13213000000",
            "abc13213000000",
            "13213000000ext1",
            "13213000000+86",
            "0013213000000",
            "-13213000000",
            "13213000000-",
            "'－13213000000",
            "''13213000000",
        ] {
            assert_eq!(database.lookup(input).status, Status::Invalid, "{input}");
        }
        assert_eq!(database.lookup("  ").status, Status::Empty);
        assert_eq!(normalize_phone("  "), None);
        assert_eq!(database.lookup("13447100000").status, Status::Missing);
    }

    #[test]
    fn ambiguous_area_does_not_claim_a_prefecture() {
        let database = Database::load().unwrap();
        for (source_province, source_city, expected_province, expected_city) in [
            ("河南", "郑州", "河南省", "郑州市"),
            ("北京", "北京", "北京市", "北京市"),
            ("四川", "凉山", "四川省", "凉山彝族自治州"),
            ("四川", "西昌", "四川省", "凉山彝族自治州"),
            ("青海", "海南", "青海省", "海南藏族自治州"),
            ("青海", "海南州", "青海省", "海南藏族自治州"),
            ("新疆", "巴州", "新疆维吾尔自治区", "巴音郭楞蒙古自治州"),
            ("新疆", "库尔勒", "新疆维吾尔自治区", "巴音郭楞蒙古自治州"),
            ("吉林", "梅河口", "吉林省", "通化市"),
            ("山东", "莱芜", "山东省", "济南市"),
            ("海南", "海南", "海南省", ""),
            ("河南", "焦作济源", "河南省", ""),
            ("湖北", "仙桃", "湖北省", ""),
        ] {
            let id = REGIONS
                .lines()
                .position(|line| {
                    let mut columns = line.split('\t');
                    columns.next() == Some(source_province) && columns.next() == Some(source_city)
                })
                .unwrap();
            assert_eq!(
                database.cities[id],
                (expected_province.into(), expected_city.into())
            );
        }
    }

    #[test]
    fn damaged_file_lengths_are_rejected() {
        let mut data = DATA.to_vec();
        data[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Database::parse(&data).is_err());
        data[20..24].copy_from_slice(&u32_at(DATA, 20).unwrap().to_le_bytes());
        data[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Database::parse(&data).is_err());
        assert!(Database::parse(&data[..36]).is_err());
    }
}
