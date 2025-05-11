use std::collections::HashMap;

use chrono::{NaiveTime, TimeDelta};
use yew::{AttrValue, Properties};

const DEFAULT_COLOR: &'static str = "00AAFF";

pub type Result<T> = std::result::Result<T, BusinessError>;
#[derive(Debug)]
pub enum BusinessError {
    EmployeeNotFound,
    RoleNotFound,
    EmployeeError(EmployeeError)
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct Business {
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub roles: HashMap<usize, Role>,
    pub employees: HashMap<usize, Employee>,
    // In production, blocks and block_size may not need to be public
    pub blocks: usize, // The number of blocks within the current business hours
    pub block_size: TimeDelta, // The size of the schedule's blocks
    pub role_colors: HashMap<usize, AttrValue>
} impl Business {
    /// Generate a sample business with 3 roles and 4 employees
    pub fn sample() -> Business {
        let open = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let close = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
        let role_vec = vec![
            Role::new_blank(3, "Primary".into(), "00AAFF".into()),
            Role::new_blank(4, "Backup".into(), "C70039".into()),
            Role::new_blank(5, "Input".into(), "11E000".into())
        ];
        let mut roles = HashMap::new();
        let mut role_colors = HashMap::new();
        for role in role_vec {
            role_colors.insert(role.id.clone(), role.color.clone());
            roles.insert(role.id.clone(), role);
        }


        let employee_vec = vec![
            Employee { 
                id: 1, 
                name: "Caleb".into(), 
                roles: vec![3,4],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 2, 
                name: "Sherri".into(), 
                roles: vec![3,4,5],
                scheduled: true,
                lunch: 1, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 3, 
                name: "Brooke".into(), 
                roles: vec![3,4,5],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                assigned: vec![]
            },
            Employee { 
                id: 4, 
                name: "Jennifer".into(), 
                roles: vec![3,4,5],
                scheduled: true,
                lunch: 2, 
                clock_in: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                clock_out: close.clone(),
                assigned: vec![]
            },
        ];
        let mut employees = HashMap::new();
        for employee in employee_vec {
            employees.insert(employee.id.clone(), employee);
        }

        let mut business = Business { 
            open, 
            close, 
            roles, 
            employees,
            blocks: 0,
            block_size: TimeDelta::minutes(30),
            role_colors
        };
        business.update_business_hours(open, close);
        business
    }

    pub fn new_role(&mut self, name: AttrValue) {
        let mut id = 2;
        loop {
            if self.roles.contains_key(&id) {
                id += 1;
            } else {
                break;
            }
        }
        self.roles.insert(id.clone(), Role::new(id, name, self.blocks));
    }
    pub fn new_employee(&mut self, name: AttrValue) {
        let mut id = 1;
        loop {
            if self.employees.contains_key(&id) {
                id += 1;
            } else {
                break;
            }
        }
        self.employees.insert(id.clone(), Employee::new(id, name, self.open.clone(), self.close.clone()));
    }

    pub fn update_business_hours(&mut self, open: NaiveTime, close: NaiveTime) {
        self.open = open;
        self.close = close;
        self.blocks = 0;
        let mut test_time = self.open.clone();
        let mut empty_vec: Vec<usize> = vec![];
        while test_time < self.close {
            test_time += self.block_size;
            self.blocks += 1;
            empty_vec.push(0);
        }
        for (_, role) in self.roles.iter_mut() {
            role.assigned = empty_vec.clone();
        }
        for (_, employee) in self.employees.iter_mut() {
            if employee.clock_in < self.open {employee.clock_in = self.open.clone()};
            if employee.clock_out > self.close {employee.clock_out = self.close.clone()};
            employee.clear_assigned(&self.open, &self.close, self.block_size.clone());
        }
    }
    pub fn update_employee_hours(&mut self, id: usize, clock_in: NaiveTime, clock_out: NaiveTime) {
        let employee = self.employees.get_mut(&id).unwrap();
        employee.clock_in = clock_in;
        employee.clock_out = clock_out;
        employee.clear_assigned(&self.open, &self.close, self.block_size.clone());
        for (_, role) in self.roles.iter_mut() {
            role.clear_employee(&id);
        }
    }

    pub fn assign_block(&mut self, employee: usize, role: usize, blocks: Vec<usize>) -> Result<usize> {
        let emp_get = self.employees.get_mut(&employee);
        if emp_get.is_none() {
            return Err(BusinessError::EmployeeNotFound);
        }
        let emp = emp_get.unwrap();
        let role_get = self.roles.get_mut(&role);
        if role_get.is_none() {
            return Err(BusinessError::RoleNotFound);
        }
        let role = role_get.unwrap();
        // Assign the block to the target employee
        let mut modified_blocks = 0;
        match emp.assign_block(blocks, role.id.clone()) {
            Ok(x) => {
                let (successful_indexes,changed_roles) = x;
                // Assign the target employee to the role
                modified_blocks += successful_indexes.len();
                let displaced_employees = role.add_block(&emp.id, successful_indexes);
                // If this would cause an overlap within a role, remove over employees from that role for overlapping indexes
                if !displaced_employees.is_empty() {
                    modified_blocks += displaced_employees.len();
                    for (emp_id, index) in displaced_employees {
                        if let Some(emp) = self.employees.get_mut(&emp_id) {
                            emp.remove_block(vec![index]);
                        }
                    }
                }
                // If target employee was already assigned a role for an index, remove employee from that role
                for (role_id, index) in changed_roles {
                    if let Some(role) = self.roles.get_mut(&role_id) {
                        role.remove_block(vec![index]);
                    }
                }
            },
            Err(e) => return Err(e)
        }
        Ok(modified_blocks)
    }
    pub fn remove_block(&mut self, employee: usize, blocks: Vec<usize>) -> Result<()>{
        let emp_get = self.employees.get_mut(&employee);
        if emp_get.is_none() {
            return Err(BusinessError::EmployeeNotFound);
        }
        let emp = emp_get.unwrap();
        let roles_to_clear = emp.remove_block(blocks);
        if roles_to_clear.is_empty() {
            return Ok(());
        }
        for (role_id, index) in roles_to_clear {
            let role_get = self.roles.get_mut(&role_id.try_into().unwrap());
            if role_get.is_none() {
                return Err(BusinessError::RoleNotFound);
            }
            role_get.unwrap().remove_block(vec![index]);
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct Role {
    pub id: usize,
    pub name: AttrValue,
    pub sort: usize,
    pub assigned: Vec<usize>,
    pub color: AttrValue,
    empty: bool
} impl Role {
    pub fn new(id: usize, name: AttrValue, blocks: usize) -> Role {
        let mut assigned = vec![];
        for _ in 0..blocks {
            assigned.push(0);
        }
        Role { id: id.clone(), name: name, sort: id, assigned: assigned, color: DEFAULT_COLOR.into(), empty: true }
    }
    pub fn new_with_assigned(id: usize, name: AttrValue, assigned: Vec<usize>, empty: bool) -> Role {
        Role { id: id.clone(), name: name, sort: id, assigned: assigned, color: DEFAULT_COLOR.into(), empty: empty }
    }
    /// Only to be used when future init of assigned is planned
    fn new_blank(id: usize, name: AttrValue, color: AttrValue) -> Role {
        Role { id: id.clone(), name: name, sort: id, assigned: vec![], color: color, empty: true }
    }

    /// Replaces the employee ID at the given indexes, returning a list of (replaced employee id, index)
    pub fn add_block(&mut self, id: &usize, indexes: Vec<usize>) -> Vec<(usize, usize)> {
        let mut replaced_employees = vec![];
        for index in indexes {
            if self.assigned[index] > 0 {
                replaced_employees.push((self.assigned[index].clone(), index.clone()));
            }
            self.assigned[index] = id.clone();
        }
        if self.empty {
            self.empty = false;
        }
        replaced_employees
    }
    pub fn remove_block(&mut self, indexes: Vec<usize>) {
        for index in indexes {
            self.assigned[index] = 0;
        }
    }
    /// Completely remove all of an employee's scheduled times from the role
    pub fn clear_employee(&mut self, id: &usize) {
        for i in 0..self.assigned.len() {
            if self.assigned[i].eq(id) {
                self.assigned[i] = 0;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }
}

// pub type Result<T> = std::result::Result<T, EmployeeError>;

#[derive(Debug)]
pub enum EmployeeError {
    NotAssignedRole { failed: usize, allowed: Vec<usize> },
    NotClockedIn

}

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct Employee {
    pub id: usize,
    pub name: AttrValue,
    pub roles: Vec<usize>, // Roles an employee can be assigned to
    pub scheduled: bool,
    pub lunch: usize, // Length of lunch break in 30-minute intervals (1 hour is 2)
    pub clock_in: NaiveTime,
    pub clock_out: NaiveTime,
    ///```assigned = vec![0,0, 1, 1, 2, 2, 1, 0]```
    /// 
    ///       0 = clocked out
    /// 
    ///       1 = available
    /// 
    ///       2 = lunch
    /// 
    ///     etc = role id
    /// 
    pub assigned: Vec<usize> // Roles an employee is currently assigned to work
} impl Employee {
    pub fn new(id: usize, name: AttrValue, clock_in: NaiveTime, clock_out: NaiveTime) -> Employee {
        Employee { id: id, name: name, roles: vec![], scheduled: true, lunch: 2, clock_in, clock_out, assigned: vec![] }
    }
    pub fn clear_assigned(&mut self, open: &NaiveTime, close: &NaiveTime, block_size: TimeDelta) {
        let mut assigned = vec![];
        let mut curr_time = open.clone();
        while curr_time < self.clock_in {
            assigned.push(0);
            curr_time += block_size;
        }
        while curr_time < self.clock_out {
            assigned.push(1);
            curr_time += block_size;
        }
        while curr_time.lt(close) {
            assigned.push(0);
            curr_time += block_size;
        }
        self.assigned = assigned;
    }

    /// Assigns role to employee at given indexes. Returns tuple of a vector of all sucessfully changed indexes and every role that was replaced
    /// at each index. 
    pub fn assign_block(&mut self, indexes: Vec<usize>, role: usize) -> Result<(Vec<usize>, Vec<(usize, usize)>)> {
        if !self.roles.contains(&role) {
            return Err(BusinessError::EmployeeError(EmployeeError::NotAssignedRole { failed: role, allowed: self.roles.clone() }));
        }
        let mut successful_indexes = vec![];
        let mut swapped_roles = vec![];
        for index in indexes.iter() {
            match self.assigned.get(*index) {
                Some(x @ 3..) => {
                    successful_indexes.push(*index);
                    swapped_roles.push((*x, *index));
                    self.assigned[*index] = role;
                },
                Some(1..=2) => {
                    successful_indexes.push(*index);
                    self.assigned[*index] = role
                },
                Some(0) | None => continue,
            }
        }
        return Ok((successful_indexes, swapped_roles))
    }
    pub fn remove_block(&mut self, indexes: Vec<usize>) -> Vec<(usize, usize)> {
        let mut cleared = vec![];
        for index in indexes {
            let curr = self.assigned[index];
            if curr == 0 {
                continue;
            }
            if curr > 2 {
                cleared.push((curr, index));
            }
            self.assigned[index] = 1;
        }
        cleared
    }

    pub fn add_role(&mut self, role: usize) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }
    pub fn remove_role(&mut self, role: &mut Role) -> Result<()> {
        let index_find = self.roles.iter().find(|&&x| x.eq(&role.id));
        match index_find {
            None => return Err(BusinessError::EmployeeError(EmployeeError::NotAssignedRole { failed: role.id, allowed: self.roles.clone() })),
            Some(index) => {
                let index = index.clone();
                self.roles.remove(index.try_into().unwrap());
            }
        }
        role.clear_employee(&self.id);
        Ok(())
    }
}