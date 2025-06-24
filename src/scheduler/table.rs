use web_sys::HtmlInputElement;
use yew::prelude::*;
use crate::{data::*, events::BusinessEvents, persistence::schedule_to_csv, scheduler::blocks::*, BusinessContext, SettingsContext, Sort};

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
    let settings = use_context::<SettingsContext>().expect("No settings context found");
    let sort = use_context::<Sort>().expect("Sort context not found");
    let held_block = use_state_eq(|| TimeBlock::default());

    let table_header = table_header(business.clone());

    let mut emp_rows = vec![];
    for (_id,employee) in business.employees.iter() {
        if !employee.scheduled {
            continue;
        }
        emp_rows.push((employee, employee.make_row(business.clone(), held_block.clone())));
    }
    emp_rows.sort_by(|a, b| a.0.cmp(&b.0, *sort));
    let mut emp_table = vec![];
    for (_, row) in emp_rows {
        emp_table.push(row);
    }
    if emp_table.is_empty() {
        emp_table.push(html!(<tr>
            <td colspan={(business.blocks + 1).to_string()} rowspan="3">{"No scheduled employees found!"}</td>
        </tr>));
    }

    html!(<>
        // <table class={"mui-table mui-table--bordered"}>
        //     {table_header.clone()}
        //     {role_table}
        // </table>
        {table_key(business.clone(), settings.clone(), held_block.clone(), sort.clone())}
        <br />
        {extra_controls(sort, business, settings)}
        <br />
        <table class={"mui-table mui-table--bordered"}>
            // <thead class="time">
                {table_header}
            // </thead>
            {emp_table}
        </table>
    </>)
}


fn table_key(business: BusinessContext, settings: SettingsContext, held_block: HeldBlock, sort: Sort) -> Html {
    let colors = &business.role_colors;

    let mut roles: Vec<&Role> = business.roles.values().collect();
    roles.sort();
    let mut role_columns = vec![];
    for role in roles {
        let block_single = TimeBlock::new_simple(0, 0, role.id());
        let block_multi = TimeBlock { emp_id: 0, time_index: 0, role: role.id(), len_index: 0,
            len: match role.id() == 2 {
                true => 2.max(settings.app.lunch_duration),
                false => settings.app.shift_length,
            }
        };
        let mut style = None;
        if colors.contains_key(&role.id()) {
            style = Some("background-color: ".to_string() + &colors[&role.id()] + ";")
        }
        let onclick;
        {
            let sort = sort.clone();
            let id = role.id();
            onclick = Callback::from(move |_| sort.set(EmployeeSort::Assigned { id }));
        }
        role_columns.push(html!(<div class="table-key-item">
            <div class="button-container">
                {role.name()}
                <input type="button" value='\u{21C5}' onclick={onclick} style="float: right; margin-right: 4px;"/>
            </div>
            {drag_block(block_single, style.clone(), business.clone(), held_block.clone())}
            {multi_block(block_multi, style, business.clone(), held_block.clone())}
        </div>));
    }

    html!(<div class="table-key">
        {role_columns}
    </div>)
}

fn extra_controls(sort: Sort, business: BusinessContext, settings: SettingsContext) -> Html {

    let (snc, scic, scoc);
    {
        let (sort1, sort2, sort3) = (sort.clone(), sort.clone(), sort.clone());
        snc = Callback::from(move |_| sort1.set(EmployeeSort::Name));
        scic = Callback::from(move |_| sort2.set(EmployeeSort::ClockIn));
        scoc = Callback::from(move |_| sort3.set(EmployeeSort::ClockOut));
    }

    let sort_buttons = html!(<>
        <input type="button" value="Name" onclick={snc} />
        <input type="button" value="Clock-In" onclick={scic} />
        <input type="button" value="Clock-Out" onclick={scoc} />
    </>);

    let lunch_callback;
    let schedule_callback;
    {
        let (b1, b2) = (business.clone(), business.clone());
        let settings = settings.clone();
        lunch_callback = Callback::from(move |_| b1.dispatch(BusinessEvents::ScheduleLunch));
        schedule_callback = Callback::from(move |_| b2.dispatch(BusinessEvents::ScheduleRoles { settings: settings.clone() }));
    }

    html!(<div class="controls">
        <p>{"Sort by: "}{sort_buttons}</p>

        <br />

        <div class="tooltip" style="border-bottom: none;">
            <input type="button" value="Guess Lunches" onclick={lunch_callback} />
            <span class="tooltiptext">{"Clear the current schedule and fill in a lunch block in the middle of every employee's day, appropriately size for that employee"}</span>
        </div>
        <div class="tooltip" style="border-bottom: none;">
            <input type="button" value="Fill in Roles" onclick={schedule_callback} />
            <span class="tooltiptext">{"For each role, in order of sort, find an available employee and attempt to assign up to 4 blocks to them"}</span>
        </div>
    </div>)
}

impl Employee {
    pub fn make_row(&self, business: BusinessContext, held_block: UseStateHandle<TimeBlock>, ) -> Html {
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
                style = Some("background-color: ".to_string() + &colors[&role] + ";")
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

#[function_component]
pub fn ScheduleCopy() -> Html {
    let business = use_context::<BusinessContext>().expect("No ctx found");

    let input_ref = use_node_ref();
    let schedule = schedule_to_csv(business.clone());

    let onclick;
    {
        let b = business.clone();
        let input_ref = input_ref.clone();
        onclick = Callback::from(move |_| b.dispatch(BusinessEvents::LoadSchedule { schedule: input_ref.cast::<HtmlInputElement>().unwrap().value() }))
    }

    html!(<div>
        <input ref={input_ref} value={schedule}/>
        <input type="button" value="Load Schedule" onclick={onclick} />
    </div>)
}