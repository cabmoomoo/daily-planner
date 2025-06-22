use std::{collections::HashMap, ops::Deref};

use chrono::TimeDelta;
use phf::phf_map;



#[derive(Debug, PartialEq, Default)]
pub struct Settings {
    app: AppSettings,
    print: PrintSettings,
} impl Settings {
    pub fn query_string(&self) -> String {
        let mut result = String::new();
        result = self.app.query_string(result);
        result = self.print.query_string(result);
        result
    }
    pub fn from_query(data: &str) -> Settings {
        let mut data_map = HashMap::new();
        let decoded = decode_query(data.to_string(), &[':', '(', ')']);
        let settings_groups: Vec<&str> = decoded.split(",").collect();
        for group in settings_groups {
            if group.is_empty() {
                continue;
            }
            let (name, all_raw_values) = group.split_at(group.find("(").unwrap());
            let mut values = HashMap::new();
            for name_value_vec in all_raw_values.split("|").map(|x| x.split(":").collect::<Vec<&str>>()) {
                if name_value_vec.len() != 2 {
                    continue;
                }
                values.insert(name_value_vec[0], name_value_vec[1]);
            }
            data_map.insert(name, values);
        }
        Settings { 
            app: AppSettings::from_data_map(&data_map), 
            print: PrintSettings::from_data_map(&data_map)
        }
    }
}

const APP_SETTINGS_KEY: &'static str = "app";
#[derive(Debug, PartialEq)]
pub struct AppSettings {
    shift_length: usize,
    lunch_duration: usize,
    block_size: TimeDelta,
} impl Default for AppSettings {
    fn default() -> Self {
        Self { shift_length: 4, lunch_duration: 2, block_size: TimeDelta::minutes(30) }
    }
} impl AppSettings {
    /// app(|shift_length:3|block_size:60|),
    fn query_string(&self, mut string: String) -> String {
        let default = AppSettings::default();
        if default.eq(self) {
            return string;
        }
        string += &format!("{}(|", APP_SETTINGS_KEY);
        if self.shift_length != default.shift_length {
            string += &format!("shift_length:{}|", self.shift_length.to_string());
        }
        if self.lunch_duration != default.lunch_duration {
            string += &format!("lunch_duration:{}|", self.lunch_duration.to_string());
        }
        if self.block_size != default.block_size {
            string += &format!("block_size:{}|", self.block_size.num_minutes());
        }
        string += "),";
        string
    }

    fn from_data_map(data_map: &HashMap<&str, HashMap<&str, &str>>) -> AppSettings {
        let default = AppSettings::default();
        match data_map.get(APP_SETTINGS_KEY) {
            Some(data) => AppSettings { 
                shift_length: {
                    match data.get("shift_length") {
                        Some(x) => x.parse().unwrap_or(default.shift_length), 
                        None => default.shift_length
                    }
                }, 
                lunch_duration: {
                    match data.get("lunch_duration") {
                        Some(x) => x.parse().unwrap_or(default.lunch_duration), 
                        None => default.lunch_duration
                    }
                }, 
                block_size: {
                    match data.get("block_size") {
                        Some(x) => {
                            let mins = x.parse();
                            match mins {
                                Ok(x) => TimeDelta::minutes(x),
                                Err(_) => default.block_size,
                            }
                        }, 
                        None => default.block_size
                    }
                }, 
            },
            None => default,
        }
    }
}

#[derive(Debug, PartialEq)]
/// Whole number, hundredths
pub struct Size(usize, usize);
impl ToString for Size {
    fn to_string(&self) -> String {
        format!("{}.{}",self.0,self.1)
    }
} impl TryFrom<&str> for Size {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(".").collect();
        let whole: usize = parts[0].parse()?;
        let decimal: usize = parts[1].parse()?;
        Ok(Size(whole, decimal))
    }
}
#[derive(Debug, PartialEq)]
pub enum PrintStyle {
    None,
    Table,
} impl ToString for PrintStyle {
    fn to_string(&self) -> String {
        match self {
            PrintStyle::None => "None".into(),
            PrintStyle::Table => "Table".into(),
        }
    }
} impl From<&str> for PrintStyle {
    fn from(value: &str) -> Self {
        match value {
            "Table" => Self::Table,
            "None" => Self::None,
            _ => Self::None
        }
    }
}
const PRINT_SETTINGS_KEY: &'static str = "print";
#[derive(Debug, PartialEq)]
pub struct PrintSettings {
    style: PrintStyle,
    width: Size,
    height: Size
} impl Default for PrintSettings {
    /// A note-card sized (5w4h) table
    fn default() -> Self {
        Self { style: PrintStyle::Table, width: Size(5, 0), height: Size(4, 0) }
    }
} impl PrintSettings {
    fn query_string(&self, mut string: String) -> String {
        let default = PrintSettings::default();
        if default.eq(self) {
            return string;
        }
        string += &format!("{}(|", PRINT_SETTINGS_KEY);
        if self.style != default.style {
            string += &format!("style:{}", self.style.to_string());
        }
        if self.width != default.width {
            string += &format!("width:{}", self.width.to_string());
        }
        if self.height != default.height {
            string += &format!("height:{}", self.height.to_string());
        }
        string += &format!("),");
        string
    }
    fn from_data_map(data_map: &HashMap<&str, HashMap<&str,&str>>) -> PrintSettings {
        let default = PrintSettings::default();
        match data_map.get(PRINT_SETTINGS_KEY) {
            Some(values) => {
                PrintSettings {
                    style: {
                        match values.get("style") {
                            Some(x) => Into::into(*x), 
                            None => default.style
                        }
                    },
                    width: {
                        match values.get("width") {
                            Some(x) => {
                                match x.deref().try_into() {
                                    Ok(x) => x,
                                    Err(_) => default.width
                                }
                            },
                            None => default.width
                        }
                    },
                    height: {
                        match values.get("height") {
                            Some(x) => {
                                match x.deref().try_into() {
                                    Ok(x) => x,
                                    Err(_) => default.height,
                                }
                            }, 
                            None => default.height
                        }
                    },
                }
            },
            None => default,
        }
    }
}

const PERCENT_ENCODING_DICTIONARY: phf::Map<char, &'static str> = phf_map!(
    ':' => "%3A",
    '/' => "%2F",
    '?' => "%3F",
    '#' => "%23",
    '[' => "%5B",
    ']' => "%5D",
    '@' => "%40",
    '!' => "%21",
    '$' => "%24",
    '&' => "%26",
    '\'' => "%27",
    '(' => "%28",
    ')' => "%29",
    '*' => "%2A",
    '+' => "%2B",
    ',' => "%2C",
    ';' => "%3B",
    '=' => "%3D",
    '%' => "%25",
    ' ' => "%20",
);

fn decode_query(mut string: String, chars: &[char]) -> String {
    for from in chars {
        let to = match PERCENT_ENCODING_DICTIONARY.get(from) {
            Some(s) => *s,
            None => continue,
        };
        string = string.replace(*from, to);
    }
    string
}