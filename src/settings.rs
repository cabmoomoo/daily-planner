use std::{collections::HashMap, ops::Deref};

use chrono::{NaiveTime, TimeDelta};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{events::BusinessEvents, print::PrintTable, BusinessContext, SettingsContext};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Settings {
    pub app: AppSettings,
    pub print: PrintSettings,
} impl Settings {
    pub fn fragment_string(&self) -> String {
        let mut result = String::new();
        result = self.app.fragment_string(result);
        result = self.print.fragment_string(result);
        result
    }
    pub fn from_fragment(data: &str) -> Settings {
        let mut data_map = HashMap::new();
        // let decoded = decode_query(data.to_string(), &[':', '(', ')']);
        let settings_groups: Vec<&str> = data.split(",").collect();
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
                values.insert(name_value_vec[0], name_value_vec[1].trim_end_matches(')'));
            }
            data_map.insert(name, values);
        }
        Settings { 
            app: AppSettings::from_data_map(&data_map), 
            print: PrintSettings::from_data_map(&data_map)
        }
    }

    pub fn app_set(mut self, app: AppSettings) -> Self {
        self.app = app;
        self
    }
    pub fn print_set(mut self, print: PrintSettings) -> Self {
        self.print = print;
        self
    }
}

#[function_component]
pub fn SettingsTab() -> Html {

    html!(<>
        <AppSettingsSection />
        <PrintSettingsSection />
        <h1 style="text-align: center;">{"Print Preview"}</h1>
        <div class="print-preview">
            <PrintTable />
        </div>
    </>)
}

const APP_SETTINGS_KEY: &'static str = "app";
#[derive(Debug, PartialEq, Clone)]
pub struct AppSettings {
    pub shift_length: usize,
    pub lunch_duration: usize,

    pub open: NaiveTime,
    pub close: NaiveTime,
    pub block_size: TimeDelta,
} impl Default for AppSettings {
    fn default() -> Self {
        Self { shift_length: 4, lunch_duration: 2, block_size: TimeDelta::minutes(30), open: NaiveTime::from_hms_opt(9, 0, 0).unwrap(), close: NaiveTime::from_hms_opt(19, 0, 0).unwrap() }
    }
} impl AppSettings {
    fn fragment_string(&self, mut string: String) -> String {
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
                open: default.open,
                close: default.close
            },
            None => default,
        }
    }

    fn business_set(&mut self, open: NaiveTime, close: NaiveTime, block_size: TimeDelta) {
        self.open = open;
        self.close = close;
        self.block_size = block_size;
    }
}

#[function_component]
fn AppSettingsSection() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");
    let settings = use_context::<SettingsContext>().expect("Settings context not found");
    let app = &settings.app;

    let (
        open_ref,
        close_ref,
        block_ref,
        shift_ref,
        lunch_ref
    ) = (
        use_node_ref(),
        use_node_ref(),
        use_node_ref(),
        use_node_ref(),
        use_node_ref()
    );

    let business_time_change_cb = {
        let b = business.clone();
        let settings = settings.clone();
        let (open_ref, close_ref, block_ref) = (open_ref.clone(), close_ref.clone(), block_ref.clone());
        Callback::from(move |_| {
            let (open, close, block_size) = (
                open_ref.cast::<HtmlInputElement>().unwrap().value().parse::<NaiveTime>().unwrap(),
                close_ref.cast::<HtmlInputElement>().unwrap().value().parse::<NaiveTime>().unwrap(),
                TimeDelta::minutes(block_ref.cast::<HtmlInputElement>().unwrap().value().parse().unwrap())
            );
            b.dispatch(BusinessEvents::UpdateBusinessHours { 
                open: open.clone(), 
                close: close.clone(), 
                block_size: block_size.clone()
            });
            let mut new_settings = settings.deref().clone();
            new_settings.app.business_set(open, close, block_size);
            settings.set(new_settings);
        })
    };

    let shift_cb = {
        let settings = settings.clone();
        let shift_ref = shift_ref.clone();
        Callback::from(move |_| {
            let mut new = settings.deref().clone();
            new.app.shift_length = shift_ref.cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
            settings.set(new);
        })
    };

    let lunch_cb = {
        let settings = settings.clone();
        let lunch_ref = lunch_ref.clone();
        Callback::from(move |_| {
            let mut new_settings = settings.deref().clone();
            new_settings.app.lunch_duration = lunch_ref.cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
            settings.set(new_settings);
        })
    };

    let step_size: AttrValue = app.block_size.num_seconds().to_string().into();

    html!(<table class="mui-table mui-table--bordered">
        <thead>
            <tr><th colspan="2">{"Application & Business Settings"}</th></tr>
        </thead>
        <tbody>
            <tr>
                <td>{"Open: "}</td>
                <td>
                    <input id="open" type="time" name="open" step={step_size.clone()} max={app.close.format("%H:%M").to_string()} value={app.open.format("%H:%M").to_string()} ref={open_ref} onblur={business_time_change_cb.clone()} />
                </td>
            </tr>
            <tr>
                <td>{"Close: "}</td>
                <td>
                    <input id="close" type="time" name="close" step={step_size} min={app.open.format("%H:%M").to_string()} value={app.close.format("%H:%M").to_string()} ref={close_ref} onblur={business_time_change_cb.clone()} />
                </td>
            </tr>
            <tr>
                <td><div class="tooltip">
                    {"Time Block Size: \u{24D8}"}
                    <span class="tooltiptext">{"The amount of time, in minutes, you wish each block to be. For example, the default value of 30 will split a 10 hour day into 20 blocks."}</span>
                </div></td>
                <td>
                    <input id="blocks" type="number" name="blocks" min={0} value={app.block_size.num_minutes().to_string()} onblur={business_time_change_cb} ref={block_ref} />
                </td>
            </tr>
            <tr>
                <td><div class="tooltip">
                    {"Preferred Shift Length: \u{24D8}"}
                    <span class="tooltiptext">{"The number of blocks to be considered a \"shift\". This is used as the desired number of blocks to assign at a time in automations, and is also the number of blocks provided in the role palette groups."}</span>
                </div></td>
                <td>
                    <input id="shift" type="number" name="shift" min={2} value={app.shift_length.to_string()} onblur={shift_cb} ref={shift_ref} />
                </td>
            </tr>
            <tr>
                <td><div class="tooltip">
                    {"Default Lunch Duration: \u{24D8}"}
                    <span class="tooltiptext">{"The number of blocks new employees to should assigned as a lunch break. Using a default block size of 30 minutes, the default lunch duration of 2 will be 1 hour."}</span>
                </div></td>
                <td>
                    <input id="lunch" type="number" name="lunch" min={1} value={app.lunch_duration.to_string()} onblur={lunch_cb} ref={lunch_ref} />
                </td>
            </tr>
        </tbody>
    </table>)
}

#[derive(Debug, PartialEq, Clone)]
/// Whole number, hundredths
pub struct Size(usize, usize);
impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
} impl std::str::FromStr for Size {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(".").collect();
        let whole: usize;
        let decimal: usize;
        if parts.len() == 1 {
            whole = s.parse()?;
            decimal = 0;
        } else {
            whole = parts[0].parse()?;
            decimal = parts[1].parse()?;
        }
        Ok(Size(whole, decimal))
    }
}
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
pub struct PrintSettings {
    pub style: PrintStyle,
    pub width: Size,
    pub height: Size,
    pub font_size: Size,
} impl Default for PrintSettings {
    /// A note-card sized (5w4h) table
    fn default() -> Self {
        Self { style: PrintStyle::Table, width: Size(5, 0), height: Size(4, 0), font_size: Size(8, 0) }
    }
} impl PrintSettings {
    fn fragment_string(&self, mut string: String) -> String {
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
        if self.font_size != default.font_size {
            string += &format!("font_size:{}", self.font_size.to_string());
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
                                match x.parse() {
                                    Ok(x) => x,
                                    Err(e) => {
                                        log::warn!("Could not parse print width! {}", e);
                                        default.width
                                    }
                                }
                            },
                            None => default.width
                        }
                    },
                    height: {
                        match values.get("height") {
                            Some(x) => {
                                match x.parse() {
                                    Ok(x) => x,
                                    Err(e) => {
                                        log::warn!("Could not parse print height! {}", e);
                                        default.height
                                    },
                                }
                            }, 
                            None => default.height
                        }
                    },
                    font_size: {
                        match values.get("font_size") {
                            Some(x) => {
                                match x.parse() {
                                    Ok(x) => x,
                                    Err(e) => {
                                        log::warn!("Could not parse print font size! {}", e);
                                        default.font_size
                                    },
                                }
                            },
                            None => default.font_size
                        }
                    }
                }
            },
            None => default,
        }
    }
}

#[function_component]
fn PrintSettingsSection() -> Html {
    let settings = use_context::<SettingsContext>().expect("Settings context not found");

    let (width_node, height_node, font_size_node) = (use_node_ref(), use_node_ref(), use_node_ref());
    let width_cb = {
        let settings = settings.clone();
        let width_node = width_node.clone();
        Callback::from(move |_| {
            let mut new_settings = settings.deref().clone();
            let size = match width_node.cast::<HtmlInputElement>().unwrap().value().parse() {
                Ok(s) => s,
                Err(_) => return
            };
            new_settings.print.width = size;
            settings.set(new_settings);
        })
    };
    let height_cb = {
        let settings = settings.clone();
        let height_node = height_node.clone();
        Callback::from(move |_| {
            let mut new_settings = settings.deref().clone();
            let size = match height_node.cast::<HtmlInputElement>().unwrap().value().parse() {
                Ok(s) => s,
                Err(_) => return
            };
            new_settings.print.height = size;
            settings.set(new_settings);
        })
    };
    let font_size_cb = {
        let settings = settings.clone();
        let font_size_node = font_size_node.clone();
        Callback::from(move |_| {
            let mut new_settings = settings.deref().clone();
            let size = match font_size_node.cast::<HtmlInputElement>().unwrap().value().parse() {
                Ok(s) => s,
                Err(_) => return
            };
            new_settings.print.font_size = size;
            settings.set(new_settings);
        })
    };

    html!(<>
        <table class="mui-table mui-table--bordered">
            <thead>
                <tr><th colspan="2">{"Print Settings"}</th></tr>
            </thead>
            <tbody>
                // <tr>
                //     <td>{"Style:"}</td>
                //     <td>

                //     </td>
                // </tr>
                <tr>
                    <td>{"Width:"}</td>
                    <td>
                        <input type="number" id="width" name="width" value={settings.print.width.to_string()} onchange={width_cb} ref={width_node} />
                    </td>
                </tr>
                <tr>
                    <td>{"Height:"}</td>
                    <td>
                        <input type="number" id="height" name="height" value={settings.print.height.to_string()} onchange={height_cb} ref={height_node} />
                    </td>
                </tr>
                <tr>
                    <td>{"Font Size:"}</td>
                    <td>
                        <input type="number" id="font_size" name="font_size" value={settings.print.font_size.to_string()} min="1.0" max="12.0" step="0.1" onchange={font_size_cb} ref={font_size_node} />
                    </td>
                </tr>
            </tbody>
        </table>
    </>)
}