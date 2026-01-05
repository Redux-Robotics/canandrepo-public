pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn indent(s: &String, indent: &str) -> String {
    s.split('\n')
        .map(|line| indent.to_owned() + line)
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn read_suffix(s: &String) -> String {
    let parts: Vec<&str> = s.split(":").collect();
    if parts.len() < 2 {
        panic!("where's the suffix dawg in \"{s}\"");
    }
    parts.get(1).unwrap().to_string()
}

pub fn read_suffix_as_usize(s: &String) -> usize {
    let parts: Vec<&str> = s.split(":").collect();
    if parts.len() < 2 {
        panic!("hey dumbass you forgot the bit length in \"{s}\"");
    }
    parts
        .get(1)
        .unwrap()
        .parse::<usize>()
        .expect("bit length specified but not an usize")
}

pub fn default_uint_max(width: usize) -> u64 {
    if width < 64 {
        (1u64 << width) - 1
    } else {
        std::u64::MAX
    }
}

pub fn default_sint_min(width: usize) -> i64 {
    if width < 64 {
        -(1i64 << (width - 1))
    } else {
        std::i64::MIN
    }
}

pub fn default_sint_max(width: usize) -> i64 {
    if width < 64 {
        (1i64 << (width - 1)) - 1
    } else {
        std::i64::MAX
    }
}

pub fn decode_bounds_f64(
    min: &Option<toml::Value>,
    max: &Option<toml::Value>,
) -> (Option<f64>, Option<f64>) {
    let min = opt_value_to_opt_f64(min);
    let max = opt_value_to_opt_f64(max);

    let (check_min, check_max) = (
        min.unwrap_or(f64::NEG_INFINITY),
        max.unwrap_or(f64::INFINITY),
    );
    if check_min > check_max {
        panic!("bounds decode failed: min {check_min} < max {check_max}");
    }
    (min, max)
}

pub fn opt_value_to_opt_u64(v: &Option<toml::Value>) -> Option<u64> {
    match v {
        Some(tv) => match tv {
            toml::Value::Integer(iv) => Some(*iv as u64),
            toml::Value::Float(fv) => Some(*fv as u64),
            _ => panic!("optional toml value uncastable to u64"),
        },
        _ => None,
    }
}

pub fn opt_value_to_opt_i64(v: &Option<toml::Value>) -> Option<i64> {
    match v {
        Some(tv) => match tv {
            toml::Value::Integer(iv) => Some(*iv as i64),
            toml::Value::Float(fv) => Some(*fv as i64),
            _ => panic!("optional toml value uncastable to i64"),
        },
        _ => None,
    }
}

pub fn opt_value_to_opt_f64(v: &Option<toml::Value>) -> Option<f64> {
    match v {
        Some(tv) => match tv {
            toml::Value::Integer(iv) => Some(*iv as f64),
            toml::Value::Float(fv) => Some(*fv as f64),
            _ => panic!("optional toml value uncastable to f64"),
        },
        _ => None,
    }
}

pub fn opt_value_to_opt_bool(v: &Option<toml::Value>) -> Option<bool> {
    match v {
        Some(tv) => match tv {
            toml::Value::Integer(iv) => Some(*iv > 0),
            toml::Value::Boolean(bv) => Some(*bv),
            _ => panic!("optional toml value uncastable to bool"),
        },
        _ => None,
    }
}
