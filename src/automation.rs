use crate::{data::{Business, Role, RoleTrait}, SettingsContext};

impl Business {

    pub fn schedule_lunch(&mut self) {
        // Reset the schedule
        self.update_business_hours(self.open, self.close, self.block_size);

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
            let (mut over, mut under) = (0, 1);
            while i < emp.lunch {
                if i % 2 == 0 {
                    lunch_indexes.push((index + over).clamp(index, self.blocks));
                    over += 1;
                } else {
                    lunch_indexes.push((index - under).clamp(0, index));
                    under += 1;
                }
                i += 1;
            }
            let _ = emp.assign_block(lunch_indexes.clone(), 2);
            lunch.add_block(&emp.id, lunch_indexes);
        }
    }

    pub fn schedule_roles(&mut self, settings: SettingsContext) {
        let mut employees = vec![];
        for emp in self.employees.values_mut() {
            if emp.scheduled {
                employees.push(emp);
            }
        }
        let mut curr_employee = 0;
        let mut roles: Vec<&mut Role> = self.roles.values_mut().collect();
        roles.sort_by(|a,b| a.sort().cmp(&b.sort()));
        for role in roles {
            if role.is_multi() {continue;}
            let mut assigned: Vec<usize> = role.assigned().into();
            for time_index in 0.. assigned.len() {
                if assigned[time_index] > 0 {
                    continue;
                }
                for _attempt in 0.. employees.len() {
                    if curr_employee >= employees.len() {
                        curr_employee = 0;
                    }
                    if employees[curr_employee].roles.contains(&role.id()) && employees[curr_employee].assigned[time_index] == 1 {
                        employees[curr_employee].assign_area(role, time_index, settings.app.shift_length);
                        curr_employee += 1;
                        assigned = role.assigned().into();
                        break;
                    }
                    curr_employee += 1;
                }
            }
        }
    }

}