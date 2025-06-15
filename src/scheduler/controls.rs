use std::str::FromStr;

use chrono::NaiveTime;
use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};

use crate::{data::RoleTrait, events::BusinessEvents, BusinessContext};

#[function_component]
pub fn Controls() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let mut scheduled = vec![];
    for (id, emp) in business.employees.iter() {
        if emp.scheduled {
            scheduled.push((id.clone(), emp.name.clone()));
        }
    }
    scheduled.sort_by(|a, b| a.1.cmp(&b.1));
    let mut emp_options = vec![];
    for (id, name) in scheduled {
        emp_options.push(html!(
            <option value={id.to_string()}>{name}</option>
        ));
    }

    let mut roles = vec![];
    for (id, role) in business.roles.iter() {
        roles.push((id.clone(), role.name(), role.sort()))
    }
    roles.sort_by(|a, b| a.2.cmp(&b.2));
    let mut role_options = vec![];
    for (id, name, _) in roles {
        role_options.push(html!(
            <option value={id.to_string()}>{name}</option>
        ));
    }

    let (emp_ref, role_ref, time_ref, block_ref) = (use_node_ref(),use_node_ref(),use_node_ref(),use_node_ref());

    let submit;
    {
        let business = business.clone();
        let (emp_ref, role_ref, time_ref, block_ref) = (emp_ref.clone(), role_ref.clone(), time_ref.clone(), block_ref.clone());

        submit = Callback::from(move |_| {
            // info!("Submit button was pressed!");
            let time = NaiveTime::from_str(&time_ref.cast::<HtmlInputElement>().unwrap().value()).unwrap();
            let blocks_input: usize = block_ref.cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
            let mut curr_time = business.open.clone();
            let mut i = 0;
            loop {
                if curr_time >= time {
                    break;
                }
                i += 1;
                curr_time += business.block_size;
            }
            let mut blocks = vec![];
            for _ in 0..blocks_input {
                blocks.push(i.clone());
                i += 1
            }
            // info!("{:#?}", blocks);
            business.dispatch(BusinessEvents::AssignBlock {
                employee: emp_ref.cast::<HtmlSelectElement>().unwrap().value().parse().unwrap(),
                role: role_ref.cast::<HtmlSelectElement>().unwrap().value().parse().unwrap(),
                blocks
            });
            let _ = emp_ref.cast::<HtmlSelectElement>().unwrap().focus();
        })
    }

    html!(<div>
        <label for="emp_select">{"Employee:"}</label>
        <select name="employee" id="emp_select" ref={emp_ref}>
            {emp_options}
        </select>

        <label for="role_select">{"Role:"}</label>
        <select name="role" id="role_select" ref={role_ref}>
            {role_options}
        </select>

        <label for="time">{"Time:"}</label>
        <input id="time" type="time" name="time" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={business.open.format("%H:%M").to_string()} ref={time_ref}/>

        <label for="blocks">{"Blocks:"}</label>
        <input id="blocks" type="number" name="blocks" min="1" value="4" ref={block_ref} />

        <input type="button" value="Submit" onclick={submit} />
    </div>)
}