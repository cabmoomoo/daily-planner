use std::{collections::HashMap, io::{Read, Write}};

use chrono::NaiveTime;

use crate::{data::{Business, RoleTrait}, BusinessContext};

/// Write current business info (roles, employees, rarely changing info) to the page hash (url query section)
pub fn write_settings(business: &Business) {
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

    let _ = location.set_hash(match serialized.len() < zip.len() {
        // true => &serialized,
        // false => &zip,
        _ => &zip
    });
}

/// Read business info from page hash
pub fn read_settings() -> Option<Business> {
    let location = web_sys::window()
        .expect("Could not pull window")
        .document()
        .expect("Could not pull document")
        .location()
        .expect("Could not pull location");
    let mut hash = location.hash().expect("Could not pull hash");
    let data = hash.split_off(match hash.find("=") {
        Some(i) => i + 1,
        None => {log::warn!("Hash found, but is not valid business information. Proceeding with sample."); return None;},
    });
    match hash.as_str() {
        "#business=" => {
            match ron::from_str(&data) {
                Ok(business) => return Some(business),
                Err(e) => {log::warn!("Failed to deserialize data; {}\n{}", e, data); return None;},
            }
        },
        "#zip=" => {
            let from_string = match encoded_from_string(data) {
                Ok(e) => e,
                Err(e) => {log::warn!("Failed to prep incoming zip for decoding; {}", e); return None;},
            };
            let mut decoder = flate2::read::ZlibDecoder::new(from_string.as_slice());
            let mut decoded = String::new();
            if let Err(e) = decoder.read_to_string(&mut decoded) {
                log::warn!("Failed to decode data; {}", e); 
                return None;
            }
            match ron::from_str(&decoded) {
                Ok(business) => return Some(business),
                Err(e) => {log::warn!("Failed to deserialize decoded data; {}", e); return None;},
            }
        },
        _ => {log::warn!("Hash found, but is not valid business information. Proceeding with sample."); return None;}
    }
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
        result += &(employee.clock_in.to_string() + SEPERATOR);
        result += &(employee.clock_out.to_string() + SEPERATOR);
        for time_index in employee.assigned.iter() {
            result += &(time_index.to_string() + SEPERATOR);
        }
        result += NEWLINE;
    }
    result
}

pub fn csv_to_schedule(csv: String) -> core::result::Result<HashMap<usize, (NaiveTime, NaiveTime, Vec<usize>)>, ParseError> {
    let mut result = HashMap::new();
    for line in csv.split(&(SEPERATOR.to_string() + NEWLINE)) {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(SEPERATOR);
        let id: usize = parts.next().unwrap().parse()?;
        let clock_in: NaiveTime = parts.next().unwrap().parse()?;
        let clock_out: NaiveTime = parts.next().unwrap().parse()?;
        let mut assigned = vec![];
        for time in parts {
            assigned.push(time.parse::<usize>()?);
        }
        result.insert(id, (clock_in, clock_out, assigned));
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
                for (emp_id, (clock_in, clock_out, assigned)) in schedule {
                    let emp_get = self.employees.get_mut(&emp_id);
                    let employee = match emp_get {
                        Some(e) => e,
                        None => {log::warn!("Attempted to load schedule for invalid employee id: {}", emp_id); continue;},
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
                        let curr_role = new_roles.get_mut(&role_id);
                        match curr_role {
                            Some(role) => role.push(i),
                            None => {new_roles.insert(role_id, vec![i]);}
                        }
                    }
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