use chrono::NaiveTime;
use log::warn;
use yew::prelude::*;

use crate::{data::RoleTrait, persistence::write_settings, scheduler::{blocks::HeldBlock, TimeBlock}};

#[derive(Clone, PartialEq)]
pub enum BusinessEvents {
    NewRole { name: AttrValue },
    NewEmployee { name: AttrValue },
    DeleteRole { role: usize },
    DeleteEmployee { emp: usize },
    UpdateBusinessHours { open: NaiveTime, close: NaiveTime },
    UpdateRoleSort { role_id: usize, increase_priority: bool },
    UpdateRoleColor { role_id: usize, color: String },
    ToggleRoleMulti { role_id: usize },
    UpdateEmployeeHours { employee: usize, clock_in: String, clock_out: String },
    ToggleEmployeeScheduled { employee: usize },
    UpdateEmployeeLunch { emp_id: usize, blocks: usize },
    ToggleEmployeeRole { employee: usize, role: usize },
    AssignBlock { employee: usize, role: usize, blocks: Vec<usize> },
    RemoveBlock { employee: usize, blocks: Vec<usize> },
    DragAssignBlock { target_block: TimeBlock, drag_block: TimeBlock, held_block: HeldBlock },

    ScheduleLunch,
    ScheduleRoles,
    LoadSchedule { schedule: String }
}

impl Reducible for crate::data::Business {
    type Action = BusinessEvents;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut business = std::rc::Rc::unwrap_or_clone(self);
        let mut update_fragment = true;
        match action {
            BusinessEvents::NewRole { name } => business.new_role(name),
            BusinessEvents::NewEmployee { name } => business.new_employee(name),
            BusinessEvents::DeleteRole { role } => business.delete_role(role),
            BusinessEvents::DeleteEmployee { emp } => business.delete_employee(emp),
            BusinessEvents::UpdateBusinessHours { open, close } => business.update_business_hours(open, close),
            BusinessEvents::UpdateRoleSort { role_id, increase_priority } => {
                // let curr_role = match business.roles.get_mut(&role_id) {
                //     Some(role) => role,
                //     None => return business.into()
                // };
                let curr_sort = business.roles[&role_id].sort();
                let mut best_swap_role: Option<&mut crate::data::Role> = None;
                for role in business.roles.values_mut() {
                    if increase_priority {
                        if let Some(curr_best) = best_swap_role {
                            if role.id() > curr_best.id() && curr_sort > role.sort() {
                                best_swap_role = Some(role);
                            } else {
                                best_swap_role = Some(curr_best)
                            }
                        } else if curr_sort > role.sort() {
                            best_swap_role = Some(role);
                        }
                    } else {
                        if let Some(curr_best) = best_swap_role {
                            if role.id() < curr_best.id() && curr_sort < role.sort() {
                                best_swap_role = Some(role);
                            } else {
                                best_swap_role = Some(curr_best)
                            }
                        } else if curr_sort < role.sort() {
                            best_swap_role = Some(role);
                        }
                    }
                }
                if let Some(op_role) = best_swap_role {
                    let new_sort = op_role.sort();
                    op_role.sort_set(curr_sort);
                    business.roles.get_mut(&role_id).unwrap().sort_set(new_sort);
                }
            },
            BusinessEvents::UpdateRoleColor { role_id, color } => business.update_role_color(role_id, color.into()),
            BusinessEvents::ToggleRoleMulti { role_id } => business.toggle_role_multi(role_id),
            BusinessEvents::UpdateEmployeeHours { employee, clock_in, clock_out } => {
                business.update_employee_hours(employee, clock_in.parse().unwrap(), clock_out.parse().unwrap());
            },
            BusinessEvents::ToggleEmployeeScheduled { employee } => business.toggle_employee_scheduled(employee),
            BusinessEvents::UpdateEmployeeLunch { emp_id, blocks } => {
                match business.employees.get_mut(&emp_id) {
                    Some(emp) => {emp.lunch = blocks},
                    None => {},
                }
            }
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
                update_fragment = false;
            },
            BusinessEvents::RemoveBlock { employee, blocks } => {
                match business.remove_block(employee, blocks) {
                    _ => ()
                }
                update_fragment = false;
            },

            BusinessEvents::DragAssignBlock { target_block, drag_block , held_block} => {
                if target_block.role != drag_block.role || target_block.time_index != held_block.time_index {
                    let mut target_block_time_indexes;
                    let mut drag_block_time_indexes;
                    if drag_block.len <= 1 {
                        target_block_time_indexes = vec![target_block.time_index];
                        drag_block_time_indexes = vec![drag_block.time_index]
                    } else {
                        target_block_time_indexes = vec![];
                        drag_block_time_indexes = vec![];
                        for i in 0..held_block.len {
                            if i <= held_block.len_index {
                                if target_block.emp_id != 0 {
                                    target_block_time_indexes.push(target_block.time_index - i);
                                }
                                drag_block_time_indexes.push(held_block.time_index - i);
                            } else {
                                if target_block.emp_id != 0 {
                                    target_block_time_indexes.push(target_block.time_index + (i - held_block.len_index));
                                }
                                drag_block_time_indexes.push(held_block.time_index + (i - held_block.len_index));
                            }
                        }
                    }
                    match business.assign_block(target_block.emp_id, drag_block.role, target_block_time_indexes.clone()) {
                        Err(e) => {
                            if target_block.emp_id == 0 {
                                let _ = business.remove_block(drag_block.emp_id, drag_block_time_indexes);
                            } else {
                                warn!("Could not assign drag block {:#?}", e);
                            }
                        },
                        Ok(_) => {
                            if drag_block.emp_id != 0 {
                                if drag_block.emp_id == target_block.emp_id {
                                    let mut old_time_indexes = vec![];
                                    for index in drag_block_time_indexes {
                                        if !target_block_time_indexes.contains(&index) {
                                            old_time_indexes.push(index);
                                        }
                                    }
                                    let _ = business.remove_block(drag_block.emp_id, old_time_indexes);
                                } else {
                                    let _ = business.remove_block(drag_block.emp_id, drag_block_time_indexes);
                                }
                            }
                        },
                    }
                }
                update_fragment = false;
            }
            BusinessEvents::ScheduleLunch => {business.schedule_lunch(); update_fragment = false;},
            BusinessEvents::ScheduleRoles => {business.schedule_roles(); update_fragment = false;},
            BusinessEvents::LoadSchedule { schedule } => {business.load_schedule(schedule); update_fragment = false;}
        }
        if update_fragment {
            write_settings(&business);
        }
        return business.into()
    }
}