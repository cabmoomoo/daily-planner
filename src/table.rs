use yew::{prelude::*, virtual_dom::VNode};
use crate::{data::*, BusinessContext};

fn table_header(business: UseReducerHandle<Business>) -> Html {
    let mut table_header = vec![];
    let mut curr_time = business.open.clone();
    table_header.push(html!(
        <th>
        </th>
    ));
    loop {
        if curr_time >= business.close {
            break;
        }
        table_header.push(html!(
            <th>
                {curr_time.format("%-I:%M").to_string()}
            </th>
        ));
        curr_time += business.block_size;
    }
    html!(
        <tr>
            {table_header}
        </tr>
    )
}

#[function_component]
pub fn Table() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let table_header = table_header(business.clone());

    let mut role_rows: Vec<Vec<VNode>> = vec![];
    for (_, role) in business.roles.iter() {
        let mut role_row: Vec<VNode> = vec![];
        role_row.push(html!(
            <td>
                {role.name.clone()}
            </td>
        ));
        for i in 0..business.blocks {
            role_row.push(html!(
                <td>
                    {role.assigned[i]}
                </td>
            ));
        }
        role_rows.push(role_row);
    }
    let mut role_table: Vec<VNode> = vec![];
    for row in role_rows {
        role_table.push(html!(
            <tr>
                {row}
            </tr>
        ));
    }

    let mut emp_rows: Vec<VNode> = vec![];
    // let base = "background: #".to_string();
    // for (_,employee) in business.employees.iter() {
    //     let mut emp_row: Vec<VNode> = vec![];
    //     emp_row.push(html!(
    //         <td>
    //             {employee.name.clone()}
    //         </td>
    //     ));
    //     for i in 0..business.blocks {
    //         let color:String = match employee.assigned[i] {
    //             0 => "AAAAAA".into(), // Not clocked in
    //             2 => "808080".into(),
    //             x @ 3.. => business.role_colors[&x].clone().to_string(),
    //             1 => "FFFFFF".into()
    //         };
    //         let style = base.clone() + &color;
    //         emp_row.push(html!(
    //             <td style={style}>
    //                 {employee.assigned[i]}
    //             </td>
    //         ));
    //     }
    //     emp_rows.push(html!(
    //         <tr>
    //             {emp_row}
    //         </tr>
    //     ));
    // }
    for (_,employee) in business.employees.iter() {
        emp_rows.push(emp_row(employee));
    }

    html!(<table>
        {table_header.clone()}
        {role_table}
        {table_header}
        {emp_rows}
    </table>)
}

fn emp_row(emp: &Employee) -> Html {
    let mut row = vec![];
    row.push(html!(
        <td>
            {emp.name.clone()}
        </td>
    ));
    for i in 0..emp.assigned.len() {
        row.push(html!(
            <td>
                {emp.assigned[i].clone()}
            </td>
        ));
    }
    html!(
        <tr key={emp.id}>
            {row}
        </tr>
    )
}