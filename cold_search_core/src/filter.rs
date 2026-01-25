use crate::RangeQuery;

#[derive(Debug, Clone)]
pub enum Filter {
    Equal(String, String),
    In(String, Vec<String>),
    NotEqual(String, String),
    NotIn(String, Vec<String>),
    Range(String, RangeQuery),
}

pub fn parse_filter_by(filter_str: &str) -> Vec<Filter> {
    let mut filters = Vec::new();
    if filter_str.is_empty() {
        return filters;
    }

    for part in filter_str.split("&&") {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some(f) = parse_single_filter(part) {
            filters.push(f);
        }
    }
    filters
}

fn parse_single_filter(part: &str) -> Option<Filter> {
    // Order matters for operator detection
    if let Some(pos) = part.find(":!=[") {
        if part.ends_with(']') {
            let field = part[..pos].trim().to_string();
            let values = part[pos + 4..part.len() - 1]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            return Some(Filter::NotIn(field, values));
        }
    }

    if let Some(pos) = part.find(":!=") {
        let field = part[..pos].trim().to_string();
        let value = part[pos + 3..].trim().to_string();
        return Some(Filter::NotEqual(field, value));
    }

    if let Some(pos) = part.find(":[") {
        if part.ends_with(']') {
            let field = part[..pos].trim().to_string();
            let content = &part[pos + 2..part.len() - 1];
            if let Some(range_pos) = content.find("..") {
                let min_str = content[..range_pos].trim();
                let max_str = content[range_pos + 2..].trim();
                let min = if min_str.is_empty() {
                    None
                } else {
                    Some(min_str.to_string())
                };
                let max = if max_str.is_empty() {
                    None
                } else {
                    Some(max_str.to_string())
                };
                return Some(Filter::Range(field, RangeQuery { min, max }));
            } else {
                let values = content.split(',').map(|s| s.trim().to_string()).collect();
                return Some(Filter::In(field, values));
            }
        }
    }

    if let Some(pos) = part.find(":>=") {
        let field = part[..pos].trim().to_string();
        let val = part[pos + 3..].trim().to_string();
        return Some(Filter::Range(
            field,
            RangeQuery {
                min: Some(val),
                max: None,
            },
        ));
    }
    if let Some(pos) = part.find(":<=") {
        let field = part[..pos].trim().to_string();
        let val = part[pos + 3..].trim().to_string();
        return Some(Filter::Range(
            field,
            RangeQuery {
                min: None,
                max: Some(val),
            },
        ));
    }
    if let Some(pos) = part.find(":>") {
        let field = part[..pos].trim().to_string();
        let val = part[pos + 2..].trim().to_string();
        return Some(Filter::Range(
            field,
            RangeQuery {
                min: Some(val),
                max: None,
            },
        ));
    }
    if let Some(pos) = part.find(":<") {
        let field = part[..pos].trim().to_string();
        let val = part[pos + 2..].trim().to_string();
        return Some(Filter::Range(
            field,
            RangeQuery {
                min: None,
                max: Some(val),
            },
        ));
    }

    if let Some(pos) = part.find(':') {
        let field = part[..pos].trim().to_string();
        let value = part[pos + 1..].trim().to_string();
        return Some(Filter::Equal(field, value));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filters() {
        let f = parse_filter_by("country:USA && value:[20..40] && type:!=[A,B]");
        println!("{:?}", f);
    }
}
