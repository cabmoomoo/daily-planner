use std::{collections::HashMap, io::{Read, Write}};

use chrono::NaiveTime;

use crate::{data::{Business, RoleTrait}, settings::Settings, BusinessContext};

pub const SETTINGS_DELIMITER: char = '&';

/// Write current business info (roles, employees, rarely changing info) to the page hash (url query section)
pub fn write_business(business: &Business) {
    let serialized = match ron::to_string(business) {
        // Ok(s) => "business=".to_string() + &s,
        Ok(s) => s,
        Err(e) => {log::error!("Failed to serialize business! {:#?}", e); return;},
    };

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(serialized.as_bytes()).unwrap();
    let encoded = encoder.finish().unwrap();
    
    let location = web_sys::window()
        .expect("Could not pull window")
        .document()
        .expect("Could not pull document")
        .location()
        .expect("Could not pull location");
    let zip = "zip=".to_string() + &encoded_to_string(encoded);

    let mut hash = location.hash().expect("Could not pull hash");
    if hash.is_empty() {
        let _ = location.set_hash(&zip);
        return;
    }
    let mut split: Vec<&str> = hash.split(SETTINGS_DELIMITER).collect();
    if split.len().eq(&1) {
        if hash.contains("zip=") {
            let _ = location.set_hash(&zip);
            return;
        } else {
            hash.push(SETTINGS_DELIMITER);
            let _ = location.set_hash(&(hash + &zip));
            return;
        }
    } else {
        for i in 0..split.len() {
            if split[i].contains("zip=") {
                split[i] = &zip;
            }
        }
        let mut final_hash = split[0].to_string();
        final_hash.push(SETTINGS_DELIMITER);
        final_hash.push_str(split[1]);
        let _ = location.set_hash(&final_hash);
    }
}

pub fn write_settings(settings: &Settings) {
    let mut settings_query = settings.fragment_string();

    if settings_query.len() == 0 {
        return;
    }

    let location = web_sys::window()
        .expect("Could not pull window")
        .document()
        .expect("Could not pull document")
        .location()
        .expect("Could not pull location");

    let hash = location.hash().expect("Could not pull hash");
    if hash.is_empty() {
        let _ = location.set_hash(&settings_query);
        return;
    }
    let mut split: Vec<&str> = hash.split(SETTINGS_DELIMITER).collect();
    if split.len().eq(&1) {
        if hash.contains("zip=") {
            settings_query.push(SETTINGS_DELIMITER);
            let _ = location.set_hash(&(settings_query + &hash));
            return;
        } else {
            let _ = location.set_hash(&settings_query);
            return;
        }
    } else {
        for i in 0..split.len() {
            if split[i].contains("zip=") {
                continue;
            }
            split[i] = &settings_query;
        }
        let mut final_hash = split[0].to_string();
        final_hash.push(SETTINGS_DELIMITER);
        final_hash.push_str(split[1]);
        let _ = location.set_hash(&final_hash);
    }
}

/// Read business info from page hash
pub fn read_settings() -> (Option<Business>, Option<Settings>) {
    let mut result = (None, None);

    let location = web_sys::window()
        .expect("Could not pull window")
        .document()
        .expect("Could not pull document")
        .location()
        .expect("Could not pull location");

    let hash = location.hash().expect("Could not pull hash");
    let split: Vec<&str> = hash.split(SETTINGS_DELIMITER).collect();

    'zip: {
        let mut hash = match split.get(1) {
            Some(s) => s.to_string(),
            None => split[0].to_string(),
        };
        if hash.is_empty() {
            log::info!("Hash found, but empty. Assuming no business information was provided and proceeding with sample.");
            break 'zip;
        }
        let data = hash.split_off(match hash.find("=") {
            Some(i) => i + 1,
            None => {log::warn!("Hash found, but is not valid business information. Proceeding with sample."); break 'zip;},
        });
        match hash.as_str() {
            "#zip=" => {
                let from_string = match encoded_from_string(data) {
                    Ok(e) => e,
                    Err(e) => {log::warn!("Failed to prep incoming zip for decoding; {}", e); break 'zip;},
                };
                let mut decoder = flate2::read::ZlibDecoder::new(from_string.as_slice());
                let mut decoded = String::new();
                if let Err(e) = decoder.read_to_string(&mut decoded) {
                    log::warn!("Failed to decode data; {}", e); 
                    break 'zip;
                }
                match ron::from_str(&decoded) {
                    Ok(business) => result = (Some(business), result.1),
                    Err(e) => {log::warn!("Failed to deserialize decoded data; {}\n{}", e, decoded); break 'zip;},
                }
            },
            _ => {log::warn!("Hash found, but is not valid business information. Proceeding with sample."); break 'zip;}
        }
    }

    'settings: {
        let hash = split[0].trim_start_matches("#").to_string();
        if hash.is_empty() {
            log::info!("Hash found, but empty. Assuming no settings information was provided and proceeding with default.");
            break 'settings;
        }
        if !hash.contains("(|") {
            log::warn!("Query found, but is not valid settings information. Proceeding with default.");
            break 'settings;
        }
        // let data = hash.split_off(match hash.find("=") {
        //     Some(i) => i+1,
        //     None => {log::warn!("Query found, but is not valid settings information. Proceeding with default."); break 'settings;},
        // });
        // match hash.as_str() {
        //     "?settings=" => {
                result = (result.0, Some(Settings::from_fragment(&hash)));
        //     },
        //     _ => {log::warn!("Query found, but is not valid settings information. Proceeding with default."); break 'settings;}
        // }
    }

    result
}

fn encoded_to_string(bytes: Vec<u8>) -> String {
    let mut result = String::new();
    for byte in bytes {
        result += &(byte.to_string() + ",");
    }
    result.pop();
    result
}

fn encoded_from_string(encoded: String) -> Result<Vec<u8>, std::num::ParseIntError> {
    let mut result = Vec::new();
    for byte in encoded.split(",") {
        result.push(u8::from_str_radix(byte, 10)?);
    }
    Ok(result)
}

// CSV header:
// employee_id, clock_in, clock_out, assigned

pub enum ParseError {
    ParseIntError(std::num::ParseIntError),
    ParseError(chrono::ParseError)
} impl From<std::num::ParseIntError> for ParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
} impl From<chrono::ParseError> for ParseError {
    fn from(value: chrono::ParseError) -> Self {
        Self::ParseError(value)
    }
} impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseIntError(parse_int_error) => write!(f, "{}", parse_int_error),
            ParseError::ParseError(parse_error) => write!(f, "{}", parse_error),
        }
    }
}

const SEPERATOR: &'static str = ",";
const NEWLINE: &'static str = "--";
pub fn schedule_to_csv(business: BusinessContext) -> String {
    let mut result = String::new();
    for employee in business.employees.values() {
        result += &(employee.id.to_string() + SEPERATOR);
        if employee.scheduled {
            result += &(employee.clock_in.to_string() + SEPERATOR);
            result += &(employee.clock_out.to_string() + SEPERATOR);
            for time_index in employee.assigned.iter() {
                result += &(time_index.to_string() + SEPERATOR);
            }
        } else {
            result += &("false".to_string() + SEPERATOR)
        }
        result += NEWLINE;
    }
    result
}

pub enum Schedule {
    True((NaiveTime,NaiveTime,Vec<usize>)),
    False
} impl From<(NaiveTime,NaiveTime,Vec<usize>)> for Schedule {
    fn from(value: (NaiveTime,NaiveTime,Vec<usize>)) -> Self {
        Self::True(value)
    }
} impl Schedule {
    pub fn decompose(self) -> Option<(NaiveTime, NaiveTime, Vec<usize>)> {
        match self {
            Schedule::True(x) => Some(x),
            Schedule::False => None,
        }
    }
}

pub fn csv_to_schedule(csv: String) -> core::result::Result<HashMap<usize, Schedule>, ParseError> {
    let mut result = HashMap::new();
    for line in csv.split(&(SEPERATOR.to_string() + NEWLINE)) {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(SEPERATOR);
        let id: usize = parts.next().unwrap().parse()?;
        let part2 = parts.next().unwrap();
        if part2.eq("false") {
            result.insert(id, Schedule::False);
            continue;
        }
        let clock_in: NaiveTime = part2.parse()?;
        let clock_out: NaiveTime = parts.next().unwrap().parse()?;
        let mut assigned = vec![];
        for time in parts {
            assigned.push(time.parse::<usize>()?);
        }
        result.insert(id, (clock_in, clock_out, assigned).into());
    }
    Ok(result)
}

impl Business {
    pub fn load_schedule(&mut self, schedule: String) {
        let result = csv_to_schedule(schedule);
        match result {
            Ok(schedule) => {
                self.roles.values_mut().for_each(|role| role.blank_out(self.blocks));
                self.employees.values_mut().for_each(|emp| emp.deschedule(self.blocks));
                for (emp_id, scheduled) in schedule {
                    let emp_get = self.employees.get_mut(&emp_id);
                    let employee = match emp_get {
                        Some(e) => e,
                        None => {log::warn!("Attempted to load schedule for invalid employee id: {}", emp_id); continue;},
                    };
                    let (clock_in, clock_out, assigned) = match scheduled.decompose() {
                        Some(x) => x,
                        None => continue,
                    };
                    if employee.assigned.len() != assigned.len() {
                        log::warn!("Employee {} schedule is incorrect length; expected: {} recieved: {}", emp_id, employee.assigned.len(), assigned.len());
                        continue;
                    }
                    employee.scheduled = true;
                    employee.clock_in = clock_in;
                    employee.clock_out = clock_out;
                    let mut new_roles: HashMap<usize, Vec<usize>> = HashMap::new();
                    for i in 0..assigned.len() {
                        let role_id = assigned[i];
                        employee.assigned[i] = 1;
                        let curr_role = new_roles.get_mut(&role_id);
                        match curr_role {
                            Some(role) => role.push(i),
                            None => {new_roles.insert(role_id, vec![i]);}
                        }
                    }
                    match new_roles.remove(&0) {
                        Some(indexes) => {
                            for index in indexes {
                                employee.assigned[index] = 0;
                            }
                        },
                        None => {},
                    }
                    let _ = new_roles.remove(&1);
                    for (role_id, new_blocks) in new_roles {
                        if let Err(e) = self.assign_block(emp_id, role_id, new_blocks) {
                            log::warn!("Failed to assign role {} for employee {}; {}", role_id, emp_id, e);
                        }
                    }
                }
            },
            Err(e) => {log::warn!("Failed to parse schedule as csv; {}", e)}
        }
    }
}