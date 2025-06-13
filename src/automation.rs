use log::info;

use crate::data::{Business, Role};

impl Business {

    pub fn schedule_lunch(&mut self) {
        // Clear the schedule of all but lunch
        for (_, role) in self.employees.iter_mut() {
            role.clear_assigned(&self.open, &self.close, self.block_size.clone());
        }

        let lunch = self.roles.get_mut(&2).unwrap();

        for (_, emp) in self.employees.iter_mut() {
            if !emp.scheduled {
                continue;
            }
            let mid_time = emp.clock_in + (emp.clock_out - emp.clock_in) / 2;
            let mut curr_time = self.open.clone();
            let mut index = 0;
            while curr_time < mid_time {
                curr_time += self.block_size;
                index += 1;
            }
            let mut lunch_indexes = vec![];
            let mut i = 0;
            while i < emp.lunch {
                if i % 2 == 0 {
                    lunch_indexes.push(index + i);
                } else {
                    lunch_indexes.push((index - i).clamp(0, index));
                }
                i += 1;
            }
            let _ = emp.assign_block(lunch_indexes.clone(), 2);
            lunch.add_block(&emp.id, lunch_indexes);
        }
    }

    pub fn schedule_roles(&mut self) {
        let mut employees = vec![];
        for emp in self.employees.values_mut() {
            if emp.scheduled {
                employees.push(emp);
            }
        }
        let mut curr_employee = 0;
        let mut roles: Vec<&mut Box<dyn Role>> = self.roles.values_mut().collect();
        roles.sort_by(|a,b| a.sort().cmp(&b.sort()));
        for role in roles {
            if role.is_multi() {info!("Role {} is a multi", role.name());continue;}
            let mut assigned: Vec<usize> = role.assigned().into();
            for time_index in 0.. assigned.len() {
                if assigned[time_index] > 0 {
                    continue;
                }
                for _attempt in 0.. employees.len() {
                    if curr_employee >= employees.len() {
                        curr_employee = 0;
                    }
                    // info!("role {}, time index {}, attempt {}, employee {}", role.name(), time_index, _attempt, employees[curr_employee].name);
                    if employees[curr_employee].roles.contains(&role.id()) && employees[curr_employee].assigned[time_index] == 1 {
                        employees[curr_employee].assign_area(role, time_index, crate::settings::DEFAULT_SHIFT);
                        curr_employee += 1;
                        assigned = role.assigned().into();
                        break;
                    }
                    curr_employee += 1;
                }
            }
            // info!("{:#?}", role);
        }
        // info!("{:#?}", self);
    }

}