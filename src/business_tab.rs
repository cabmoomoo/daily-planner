use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{data::*, events::BusinessEvents, BusinessContext};

pub const DEFAULT_SHIFT: usize = 4;

#[function_component]
pub fn BusinessTab() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let mut role_rows = vec![];
    let mut roles_list = business.roles.values().collect::<Vec<&Role>>();
    roles_list.sort();
    let mut role_sorts = vec![];
    for role in roles_list.iter() {
        if role.id() == 2 {
            continue;
        }
        role_sorts.push(role.sort());
    }
    role_sorts.sort();
    for role in roles_list.iter() {
        role_rows.push(html!(<RoleRow role_id={role.id()}/>));
    }

    let mut emp_rows = vec![];
    let mut emp_list = business.employees.values().collect::<Vec<&Employee>>();
    emp_list.sort_by(|a,b| a.cmp(&b, EmployeeSort::Name));
    for emp in emp_list {
        emp_rows.push(html!(
            <EmpRow emp={emp.clone()} />
        ));
    }

    let mut header_row = vec![];
    let mut lunch_header_text = "Lunch ".to_string();
    lunch_header_text.push('\u{24D8}');
    header_row.push(html!(<>
        <th>
            {"Scheduled"}
        </th>
        <th>
            {"Name"}
        </th>
        <th>
            {"Clock-in"}
        </th>
        <th>
            {"Clock-out"}
        </th>
        <th>
            <div class="tooltip">
                {lunch_header_text}
                <span class="tooltiptext">{"Duration of employee lunch in minutes (will be rounded to nearest 30)"}</span>
            </div>
        </th>
    </>));
    for role in roles_list {
        if role.id() == 2 {
            continue;
        }
        header_row.push(html!(
            <th>
                {business.roles[&role.id()].name()}
            </th>
        ));
    }
    let mut multi_role_text = "Multi-Role ".to_string();
    multi_role_text.push('\u{24D8}');

    html!(<>
        <table class={classes!("mui-table","mui-table--bordered")}>
            <thead>
                <tr>
                    <th>{"Role"}</th>
                    <th>{"Priority"}</th>
                    <th>{"Color"}</th>
                    <th>
                        <div class="tooltip">
                            {multi_role_text}
                            <span class="tooltiptext">{"A multi-role is a role which can have multiple employees assigned in a given time block. The default single-role enforces a maximum of one employee assigned in any given time block."}</span>
                        </div>

                    </th>
                </tr>
            </thead>
            <ContextProvider<Vec<usize>> context={role_sorts}>
            <tbody>
                {role_rows}
                <RoleNew />
            </tbody>
            </ContextProvider<Vec<usize>>>
        </table>
        <table class={classes!("mui-table","mui-table--bordered","employee-table")}>
            <thead><tr>
                {header_row}
            </tr></thead>
            <tbody>
                {for emp_rows}
                <EmpNew />
            </tbody>
        </table>
    </>)
}

#[derive(Properties, PartialEq)]
struct RoleRowProps {
    role_id: usize
}

#[function_component]
fn RoleRow(props: &RoleRowProps) -> Html {
    let business = use_context::<BusinessContext>().expect("No context found");
    let role_sorts = use_context::<Vec<usize>>().expect("No sorts context found");
    let role_id = props.role_id;
    let role = business.roles.get(&role_id).expect("It should not be possible to pass an invalid role id at this point");
    let sort = role.sort();

    let mut buttons = vec![];
    if sort.ne(role_sorts.first().unwrap_or(&sort)) && role.id() != 2 {
        let b = business.clone();
        let onclick = Callback::from(move |_| b.dispatch(BusinessEvents::UpdateRoleSort { role_id: role_id.clone(), increase_priority: true }));
        buttons.push(html!(
            <input type="button" value='\u{2191}' onclick={onclick}/>
        ));
    }
    if sort.ne(role_sorts.last().unwrap_or(&sort)) && role.id() != 2 {
        let b = business.clone();
        let onclick = Callback::from(move |_| b.dispatch(BusinessEvents::UpdateRoleSort { role_id: role_id.clone(), increase_priority: false }));
        buttons.push(html!(
            <input type="button" value='\u{2193}' onclick={onclick} />
        ));
    }
    if role.id() != 2 {
        let b = business.clone();
        let onclick = Callback::from(move |_| b.dispatch(BusinessEvents::DeleteRole { role: role_id.clone() }));
        buttons.push(html!(
            <input type="button" value='\u{2715}' onclick={onclick} />
        ));
    }

    let color_node = use_node_ref();
    let color_onkeyup;
    let color_blur;
    {
        let (b1, b2) = (business.clone(), business.clone());
        let (cn1, cn2) = (color_node.clone(), color_node.clone());
        color_onkeyup = Callback::from(move |e: KeyboardEvent| {
            if e.key().eq("Enter") {
                b1.dispatch(BusinessEvents::UpdateRoleColor { role_id: role_id, color: cn1.cast::<HtmlInputElement>().unwrap().value()});
            }
        });
        color_blur = Callback::from(move |_| b2.dispatch(BusinessEvents::UpdateRoleColor { role_id: role_id, color: cn2.cast::<HtmlInputElement>().unwrap().value()}))
    }

    let multi_cb = {
        let b = business.clone();
        let role_id = role.id();
        move |_| {b.dispatch(BusinessEvents::ToggleRoleMulti { role_id: role_id });}
    };

    html!(<tr key={role_id}>
        <td>
            {role.name()}
        </td>
        <td>
            {role.sort()}
        </td>
        <td>
            <input type="color" value={role.color()} onkeyup={color_onkeyup} onblur={color_blur} ref={color_node} />
        </td>
        <td>
            <input type="checkbox" id="scheduled" name={role.name().to_string() + "Multi"} value={role.id().to_string()} checked={role.is_multi()} onchange={multi_cb} disabled={role_id == 2} />
        </td>
        <td>
            {buttons}
        </td>
    </tr>)
}

#[function_component]
fn RoleNew() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let name_ref = use_node_ref();

    let onclick;
    {
        let b = business.clone();
        let name = name_ref.clone();
        onclick = Callback::from(move |_| b.dispatch(BusinessEvents::NewRole { name: name.cast::<HtmlInputElement>().unwrap().value().into() }));
    }
    let onkeydown;
    {
        let b = business.clone();
        let name = name_ref.clone();
        onkeydown = Callback::from(move |e: KeyboardEvent| {
            if e.key().eq("Enter") {
                let name = name.cast::<HtmlInputElement>().unwrap();
                b.dispatch(BusinessEvents::NewRole { name: name.value().into() });
                name.set_value("");
            }
        });
    }

    html!(<tr key={"RoleNew"}>
        <td>
            <input ref={name_ref} onkeydown={onkeydown} />
            <input type="button" value='\u{1F5F8}' onclick={onclick} />
        </td>
    </tr>)
}

#[derive(Properties, PartialEq)]
struct EmpProp {
    emp: Employee
}

#[function_component]
fn EmpRow(props: &EmpProp) -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");
    let emp = &props.emp;
    let mut emp_row = vec![];
    let (clock_in_ref, clock_out_ref, lunch_ref) = (use_node_ref(), use_node_ref(), use_node_ref());
    // let b2 = business.clone();
    let scheduled_cb = {
        let b2 = business.clone(); 
        let scheduled_cb_event = BusinessEvents::ToggleEmployeeScheduled { employee: emp.id.clone() };
        move |_| b2.dispatch(scheduled_cb_event.clone())
    };
    let clock_cb = {
        let b2 = business.clone();
        let emp_id = emp.id.clone();
        let (clock_in_ref, clock_out_ref) = (clock_in_ref.clone(), clock_out_ref.clone());
        move |_| b2.dispatch(BusinessEvents::UpdateEmployeeHours {
            employee: emp_id,
            clock_in: clock_in_ref.cast::<HtmlInputElement>().unwrap().value(),
            clock_out: clock_out_ref.cast::<HtmlInputElement>().unwrap().value()
        })
    };
    let lunch = business.block_size.checked_mul(emp.lunch.try_into().unwrap()).unwrap();
    let lunch_cb = {
        let b = business.clone();
        let emp_id = emp.id.clone();
        let block_mins = b.block_size.num_minutes();
        let curr_lunch_mins = lunch.num_minutes();
        let lunch_ref = lunch_ref.clone();
        move |_| {
            let lunch_node = lunch_ref.cast::<HtmlInputElement>().unwrap();
            let input: i64 = match lunch_node.value().parse() {
                Ok(num) => num,
                Err(e) => {
                    warn!("Could not parse input lunch as number; {}", e);
                    lunch_node.set_value(&curr_lunch_mins.to_string());
                    return;
                }
            };
            let mut full_blocks = input / block_mins;
            let remainder = input % block_mins;
            if remainder > 0 {
                let block_half = b.block_size.checked_div(2).unwrap().num_minutes();
                if remainder > block_half {
                    full_blocks += 1;
                }
            }
            b.dispatch(BusinessEvents::UpdateEmployeeLunch { emp_id: emp_id, blocks: full_blocks.try_into().unwrap() });
        }
    };
    emp_row.push(html!(<>
        <td>
            <input id="scheduled" type="checkbox" name={emp.name.to_string() + "Scheduled"} value={emp.id.to_string()} checked={emp.scheduled} onchange={scheduled_cb}/>
        </td>
        <td>
            {emp.name.clone()}
        </td>
        <td>
            <input id="clock_in" type="time" name="clock_in" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_in.format("%H:%M").to_string()} ref={clock_in_ref} onblur={clock_cb.clone()} disabled={!emp.scheduled} />
        </td>
        <td>
            <input id="clock_out" type="time" name="clock_out" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_out.format("%H:%M").to_string()} ref={clock_out_ref} onblur={clock_cb.clone()} disabled={!emp.scheduled} />
        </td>
        <td>
            <input id="lunch_time" type="number" name="lunch_time" min={0} value={lunch.num_minutes().to_string()} onblur={lunch_cb} ref={lunch_ref} />
        </td>
    </>));
    let mut roles_list: Vec<&Role> = business.roles.values().collect();
    roles_list.sort();
    for role in roles_list {
        let id = role.id();
        if 2.eq(&id) {
            continue;
        }
        let box_id = emp.id.to_string() + "/" + &id.to_string();
        let b2 = business.clone();
        let change_event = BusinessEvents::ToggleEmployeeRole { employee: emp.id.clone(), role: id.clone() };
        let cb = {move |_| b2.dispatch(change_event.clone())};
        emp_row.push(html!(
            <td>
                <input type="checkbox" name={box_id.clone()} id={box_id.clone()} value={id.to_string()} checked={emp.roles.contains(&id)} onchange={cb}/>
                // <label for={box_id}>{role.name.clone()}</label>
            </td>
        ));
    }
    let onclick;
    {
        let b = business.clone();
        let id = emp.id.clone();
        onclick = Callback::from(move |_| b.dispatch(BusinessEvents::DeleteEmployee { emp: id }));
    }
    emp_row.push(html!(
        <td>
            <input type="button" value='\u{2715}' onclick={onclick} />
        </td>
    ));
    let key = emp.id.to_string() + &emp.name.to_string();
    html!(<tr key={key}>
        {emp_row}
    </tr>)
}

#[function_component]
fn EmpNew() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let name_ref = use_node_ref();

    let onclick;
    {
        let b = business.clone();
        let name = name_ref.clone();
        onclick = Callback::from(move |_| b.dispatch(BusinessEvents::NewEmployee { name: name.cast::<HtmlInputElement>().unwrap().value().into() }))
    }
    let onkeydown;
    {
        let b = business.clone();
        let name = name_ref.clone();
        onkeydown = Callback::from(move |e: KeyboardEvent| {
            if e.key().eq("Enter") {
                let name = name.cast::<HtmlInputElement>().unwrap();
                b.dispatch(BusinessEvents::NewEmployee { name: name.value().into() });
                name.set_value("");
            }
        })
    }

    html!(<tr key={"EmpNew"}>
        <td></td>
        <td>
            <input ref={name_ref} onkeydown={onkeydown}/>
            <input type="button" value='\u{1F5F8}' onclick={onclick} />
        </td>
        // <td>
        // </td>
    </tr>)
}