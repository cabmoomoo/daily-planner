use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{data::*, events::BusinessEvents, BusinessContext};

pub const DEFAULT_SHIFT: usize = 4;

#[function_component]
pub fn Settings() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let mut role_rows = vec![];
    let mut roles_list = business.roles.values().collect::<Vec<&Box<dyn Role>>>();
    roles_list.sort();
    for role in roles_list.iter() {
        role_rows.push(role_row(role.id(), business.clone()));
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
    </>));
    for role in roles_list {
        header_row.push(html!(
            <th>
                {business.roles[&role.id()].name()}
            </th>
        ));
    }

    html!(<>
        <table class={classes!("mui-table","mui-table--bordered")}>
            <thead>
                <tr>
                    <th>{"Role"}</th>
                    <th>{"Priority"}</th>
                    <th>{"Color"}</th>
                </tr>
            </thead>
            <tbody>
                {role_rows}
            </tbody>
        </table>
        <table class={classes!("mui-table","mui-table--bordered","employee-table")}>
            <thead><tr>
                {header_row}
            </tr></thead>
            <tbody>
                {for emp_rows}
            </tbody>
        </table>
    </>)
}

// #[derive(Properties, PartialEq)]
// #[derive(Properties, PartialEq)]
// struct RoleProp {
//     role: impl Role
// }

// #[function_component]
// fn RoleRow(props: &RoleProp) -> Html {

//     html!(

//     )
// }

fn role_row(role_id: usize, business: BusinessContext) -> Html {
    let role = business.roles.get(&role_id).expect("It should not be possible to pass an invalid role id at this point");
    html!(<tr key={role_id}>
        <td>
            {role.name()}
        </td>
        <td>
            {role.sort()}
        </td>
        <td>
            {role.color()}
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
    let (clock_in_ref, clock_out_ref) = (use_node_ref(), use_node_ref());
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
    emp_row.push(html!(<>
        <td>
            <input id="scheduled" type="checkbox" name={emp.name.to_string() + "Scheduled"} value={emp.id.to_string()} checked={emp.scheduled} onchange={scheduled_cb}/>
        </td>
        <td>
            {emp.name.clone()}
        </td>
        <td>
            <input id="clock_in" type="time" name="clock_in" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_in.format("%H:%M").to_string()} ref={clock_in_ref} onblur={clock_cb.clone()} />
        </td>
        <td>
            <input id="clock_out" type="time" name="clock_out" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_out.format("%H:%M").to_string()} ref={clock_out_ref} onblur={clock_cb.clone()} />
        </td>
    </>));
    let mut roles_list: Vec<&usize> = business.roles.keys().collect();
    roles_list.sort();
    for id in roles_list {
        let box_id = emp.id.to_string() + "/" + &id.to_string();
        let b2 = business.clone();
        let change_event = BusinessEvents::ToggleEmployeeRole { employee: emp.id.clone(), role: *id };
        let cb = {move |_| b2.dispatch(change_event.clone())};
        emp_row.push(html!(
            <td>
                <input type="checkbox" name={box_id.clone()} id={box_id.clone()} value={id.to_string()} checked={emp.roles.contains(id)} onchange={cb}/>
                // <label for={box_id}>{role.name.clone()}</label>
            </td>
        ));
    }
    let key = emp.id.to_string() + &emp.name.to_string();
    html!(<tr key={key}>
        {emp_row}
    </tr>)
}