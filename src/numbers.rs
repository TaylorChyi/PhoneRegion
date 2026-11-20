use crate::lookup::normalize_phone;

fn separator(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ',' | '，' | ';' | '；' | '、')
}

/// Prefer a single formatted number. Expand a cell only if every part can
/// independently be read as a complete mainland mobile number.
pub(crate) fn batch_values(raw: &str) -> Vec<&str> {
    if normalize_phone(raw).is_some() {
        return vec![raw];
    }
    let mut rest = raw.trim_matches(separator);
    let mut values = Vec::new();
    while !rest.is_empty() {
        let mut found = None;
        for end in rest
            .char_indices()
            .filter_map(|(offset, ch)| separator(ch).then_some(offset))
            .chain(std::iter::once(rest.len()))
        {
            // Limit speculative parsing on long free-text cells.
            if end > 192 {
                break;
            }
            let candidate = &rest[..end];
            if normalize_phone(candidate).is_some() {
                found = Some((candidate, end));
                break;
            }
        }
        let Some((part, end)) = found else {
            return vec![raw];
        };
        values.push(part);
        rest = rest[end..].trim_start_matches(separator);
    }
    if values.len() > 1 { values } else { vec![raw] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_one_number_or_an_invalid_field_intact() {
        for value in [
            "",
            " +86 132 1300 0000 ",
            "13213000000 1234",
            "号码 13213000000",
            "1321300000013213000001",
        ] {
            assert_eq!(batch_values(value), vec![value]);
        }
    }

    #[test]
    fn splits_only_at_unambiguous_boundaries() {
        assert_eq!(
            batch_values(" +86 132 1300 0000　0086 13213000001、13213000002 "),
            vec!["+86 132 1300 0000", "0086 13213000001", "13213000002"]
        );
        assert_eq!(
            batch_values("13213000000\n13213000000"),
            vec!["13213000000", "13213000000"]
        );
    }
}
