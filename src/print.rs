use yew::prelude::*;

use crate::{data::{Role, RoleTrait}, BusinessContext, SettingsContext};

#[function_component]
pub fn PrintTable() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");
    let settings = use_context::<SettingsContext>().expect("Settings context not found");

    let mut columns = vec![];

    let mut roles: Vec<&Role> = business.roles.values().collect();
    roles.sort();
    for role in roles {
        if role.id() == 2 {
            continue;
        }

        let mut emps = vec![];
        let mut i = 0;
        let mut curr_time = business.open.clone();
        let mut last_group = (0, business.open.clone());
        let assigned: Vec<usize> = role.assigned().into();
        while curr_time < business.close {
            'block: {
                if last_group.0 != assigned[i] {
                    if last_group.0 != 0 {
                        let employee = match business.employees.get(&last_group.0) {
                            Some(emp) => emp,
                            None => {
                                log::warn!("Failed to find employee id {} when generating print table for role {}", assigned[last_group.0], role.name());
                                break 'block;},
                        };
                        // let row = format!("{} {}-{}", employee.name, last_group.1.format("%-I:%M").to_string(), curr_time.format("%-I:%M").to_string());
                        emps.push(html!(
                            <li>
                                {employee.name.clone()}
                                <span>{format!(" {}-{}", last_group.1.format("%-I:%M").to_string(), curr_time.format("%-I:%M").to_string())}</span>
                            </li>
                        ));
                    }
                    last_group = (assigned[i].clone(),curr_time.clone());
                }
            }

            i += 1;
            curr_time += business.block_size;
        }
        'block: {
            if last_group.0 != 0 {
                let employee = match business.employees.get(&last_group.0) {
                    Some(emp) => emp,
                    None => {
                        log::warn!("Failed to find employee id {} when generating print table for role {}", assigned[last_group.0], role.name());
                        break 'block;},
                };
                // let row = format!("{} {}-{}", employee.name, last_group.1.format("%-I:%M").to_string(), curr_time.format("%-I:%M").to_string());
                emps.push(html!(
                    <li>
                        {employee.name.clone()}
                        <span>{format!(" {}-{}", last_group.1.format("%-I:%M").to_string(), curr_time.format("%-I:%M").to_string())}</span>
                    </li>
                ));
            }
        }

        columns.push(html!(
            <div class="print-column">
                <h3>{role.name()}</h3>
                <ul class="no-bullets">
                    {for emps}
                </ul>
            </div>
        ));
    }
    let lunch = business.roles.get(&2).unwrap();
    {
        let mut emps = vec![];
        let mut i = 0;
        let mut curr_time = business.open.clone();
        let assigned: Vec<Vec<usize>> = lunch.assigned().into();
        let mut listed = vec![];
        while curr_time < business.close {
            let block = &assigned[i];
            for emp_id in block {
                if 0.eq(emp_id) {
                    continue;
                }
                if listed.contains(&emp_id) {
                    continue;
                } else {
                    listed.push(emp_id);
                }
                let employee = match business.employees.get(&emp_id) {
                    Some(emp) => emp,
                    None => {
                        log::warn!("Failed to get employee id {} while generating print table for lunch", emp_id);
                        continue;
                    },
                };
                // let row = format!("{} {}",employee.name,curr_time.format("%-I:%M").to_string());
                emps.push(html!(
                    <li>
                        {employee.name.clone()}{" "}
                        <span>{curr_time.format("%-I:%M").to_string()}</span>
                    </li>
                ));
            }

            i += 1;
            curr_time += business.block_size;
        }
        columns.push(html!(
            <div class="print-column" style="border: none;">
                <h3>{"Lunch"}</h3>
                <ul class="no-bullets">
                    {for emps}
                </ul>
            </div>
        ));
    }

    let style = format!(
        "width: {}in; height: {}in; font-size: {}pt", 
        settings.print.width, 
        settings.print.height,
        settings.print.font_size
    );

    html!(<div class="print-area" style={style}>
        {for columns}
    </div>)
}