use log::{error, info};
use yew::{prelude::*, virtual_dom::VNode};
use crate::{data::*, scheduler::blocks::*, BusinessContext};

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

    let held_block = use_state_eq(|| TimeBlock::default());

    let table_header = table_header(business.clone());

    let mut role_rows: Vec<(usize, Vec<VNode>)> = vec![];
    for (_, role) in business.roles.iter() {
        let mut role_row: Vec<VNode> = vec![];
        role_row.push(html!(
            <td>
                {role.name()}
            </td>
        ));
        for i in 0..business.blocks {
            role_row.push(html!(
                <td>
                    {role.assigned()[i]}
                </td>
            ));
        }
        role_rows.push((role.id(), role_row));
    }
    role_rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut role_table: Vec<VNode> = vec![];
    for (key,row) in role_rows {
        role_table.push(html!(
            <tr key={key}>
                {row}
            </tr>
        ));
    }

    let mut emp_rows = vec![];
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
    for (id,employee) in business.employees.iter() {
        emp_rows.push((id.clone(), employee.make_row(business.clone(), held_block.clone())));
    }
    emp_rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut emp_table = vec![];
    for (_, row) in emp_rows {
        emp_table.push(row);
    }

    html!(<table class={"mui-table mui-table--bordered"}>
        {table_header.clone()}
        {role_table}
        {table_header}
        {emp_table}
    </table>)
}

impl Employee {
    pub fn make_row(&self, business: BusinessContext, held_block: UseStateHandle<TimeBlock>) -> Html {
        let colors = &business.role_colors;
        let mut row = vec![];
        row.push(html!(
            <td>
                {self.name.clone()}
            </td>
        ));
        let mut prev_role = 0;
        for i in 0..self.assigned.len() {
            let role = self.assigned[i].clone();

            if prev_role != 0 && prev_role == role {
                continue;
            } else if prev_role != 0 {
                prev_role = 0;
            }

            let mut style = None;
            if colors.contains_key(&role) {
                style = Some("background-color: #".to_string() + &colors[&role] + ";")
            }

            if role == 0 {
                // If not at work, give empty block
                row.push(html!(<td class="empty-block"></td>));
            } else if role == 1 {
                // If unassigned, give non-draggable block
                row.push(html!(
                    <td>
                        {static_block(TimeBlock::new_simple(self.id.clone(), i, role), style, business.clone(), held_block.clone())}
                    </td>
                ));
            // } else if prev_role == role {
            //     continue;
            } else {
                // If assigned, give draggable block

                let mut role_len = 1;
                let mut role_i = i+1;
                loop {
                    if let Some(next_role) = self.assigned.get(role_i) {
                        if role.eq(next_role) {
                            role_len += 1;
                            role_i += 1;
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if role_len == 1 {
                    row.push(html!(
                        <td>
                            {drag_block(TimeBlock::new_simple(self.id.clone(), i, role), style, business.clone(), held_block.clone())}
                        </td>
                    ));
                    continue;
                }
                prev_role = role.clone();
                row.push(html!(
                    <td colspan={role_len.to_string()}>
                        {multi_block(TimeBlock { emp_id: self.id.clone(), time_index: i, role, len: role_len, len_index: 0 }, style, business.clone(), held_block.clone())}
                    </td>
                ));

            }
        }
        html!(
            <tr key={self.id}>
                {row}
            </tr>
        )
    }
}

// fn emp_row(emp: &Employee) -> Html {
//     let mut row = vec![];
//     row.push(html!(
//         <td>
//             {emp.name.clone()}
//         </td>
//     ));
//     for i in 0..emp.assigned.len() {
//         row.push(html!(
//             <td>
//                 // {emp.assigned[i].clone()}
//             </td>
//         ));
//     }
//     html!(
//         <tr key={emp.id}>
//             {row}
//         </tr>
//     )
// }

