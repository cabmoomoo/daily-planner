use chrono::NaiveTime;
use log::warn;
use yew::prelude::*;


pub enum BusinessEvents {
    NewRole { name: AttrValue },
    NewEmployee { name: AttrValue },
    UpdateBusinessHours { open: NaiveTime, close: NaiveTime },
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
            BusinessEvents::UpdateBusinessHours { open, close } => business.update_business_hours(open, close),
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