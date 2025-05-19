use chrono::NaiveTime;
use log::warn;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum BusinessEvents {
    NewRole { name: AttrValue },
    NewEmployee { name: AttrValue },
    DeleteRole { role: usize },
    DeleteEmployee { emp: usize },
    UpdateBusinessHours { open: NaiveTime, close: NaiveTime },
    UpdateEmployeeHours { employee: usize, clock_in: String, clock_out: String },
    ToggleEmployeeRole { employee: usize, role: usize },
    AssignBlock { employee: usize, role: usize, blocks: Vec<usize> },
    RemoveBlock { employee: usize, blocks: Vec<usize> }
}

impl Reducible for crate::data::Business {
    type Action = BusinessEvents;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut business = std::rc::Rc::unwrap_or_clone(self);
        match action {
            BusinessEvents::NewRole { name } => business.new_role(name),
            BusinessEvents::NewEmployee { name } => business.new_employee(name),
            BusinessEvents::DeleteRole { role } => business.delete_role(role),
            BusinessEvents::DeleteEmployee { emp } => business.delete_employee(emp),
            BusinessEvents::UpdateBusinessHours { open, close } => business.update_business_hours(open, close),
            BusinessEvents::UpdateEmployeeHours { employee, clock_in, clock_out } => {
                business.update_employee_hours(employee, clock_in.parse().unwrap(), clock_out.parse().unwrap());
                },
            BusinessEvents::ToggleEmployeeRole { employee, role } => {
                let emp_get = business.employees.get(&employee);
                if let Some(emp) = emp_get {
                    if emp.roles.contains(&role) {
                        match business.restrict_role(employee, role) {
                            Ok(_) => (),
                            Err(e) => warn!("{:#?}", e)
                        }
                    } else {
                        business.assign_role(employee, role);
                    }
                }
            },
            BusinessEvents::AssignBlock { employee, role, blocks } => {
                match business.assign_block(employee, role, blocks) {
                    // Ok(modified) => {info!("Modified {} blocks", modified)},
                    Ok(_) => (),
                    Err(e) => warn!("{:#?}", e)
                }
            },
            BusinessEvents::RemoveBlock { employee, blocks } => {
                match business.remove_block(employee, blocks) {
                    _ => ()
                }
            },
        }
        return business.into()
    }
}