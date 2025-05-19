

use std::ops::Deref;

use yew::prelude::*;

use crate::{data::*, events::BusinessEvents, BusinessContext};

#[function_component]
pub fn Settings() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");
    let mut emp_rows = vec![];
    for (_, emp) in business.employees.iter() {
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
    let mut roles_list: Vec<&usize> = business.roles.keys().collect();
    roles_list.sort();
    for id in roles_list {
        header_row.push(html!(
            <th>
                {business.roles[id].name.clone()}
            </th>
        ));
    }

    html!(<table class={"mui-table mui-table--bordered"}>
        <tr>
            {header_row}
        </tr>
        {for emp_rows}
    </table>)
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
    emp_row.push(html!(<>
        <td>
            <input id="scheduled" type="checkbox" name={emp.name.to_string() + "Scheduled"} value={emp.id.to_string()} checked={emp.scheduled} />
        </td>
        <td>
            {emp.name.clone()}
        </td>
        <td>
            <input id="clock_in" type="time" name="clock_in" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_in.format("%H:%M").to_string()} ref={clock_in_ref}/>
        </td>
        <td>
            <input id="clock_out" type="time" name="clock_out" step="1800" min={business.open.format("%H:%M").to_string()} max={business.close.format("%H:%M").to_string()} value={emp.clock_out.format("%H:%M").to_string()} ref={clock_out_ref}/>
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