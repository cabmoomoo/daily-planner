use std::collections::HashMap;

use chrono::{NaiveTime, TimeDelta};
use dyn_clone::DynClone;
use log::info;
use yew::{AttrValue, prelude::*, Properties};

const DEFAULT_COLOR: &'static str = "00AAFF";

pub type Result<T> = std::result::Result<T, BusinessError>;
#[derive(Debug)]
pub enum BusinessError {
    EmployeeNotFound,
    RoleNotFound,
    EmployeeError(EmployeeError)
}

fn business_base() -> Box<dyn Role> {
    Box::new(MultiRole::new(2, "Lunch".into(), 0))
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub struct Business {
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub roles: HashMap<usize, Box<dyn Role>>,
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
        let role_vec: Vec<Box<dyn Role>> = vec![
            business_base(),
            Box::new(SingleRole::new_blank(3, "Primary".into(), "00AAFF".into())),
            Box::new(SingleRole::new_blank(4, "Backup".into(), "C70039".into())),
            Box::new(SingleRole::new_blank(5, "Input".into(), "11E000".into()))
        ];
        let mut roles = HashMap::new();
        let mut role_colors = HashMap::new();
        for role in role_vec {
            role_colors.insert(role.id(), role.color());
            roles.insert(role.id(), role);
        }


        let employee_vec = vec![
            Employee { 
                id: 1, 
                name: "Caleb".into(), 
                roles: vec![2,3,4],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 2, 
                name: "Sherri".into(), 
                roles: vec![2,3,4,5],
                scheduled: true,
                lunch: 1, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 3, 
                name: "Brooke".into(), 
                roles: vec![2,3,4,5],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                assigned: vec![]
            },
            Employee { 
                id: 4, 
                name: "Jennifer".into(), 
                roles: vec![2,3,4,5],
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
        self.roles.insert(id.clone(), Box::new(SingleRole::new(id, name, self.blocks)));
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
    pub fn delete_role(&mut self, role: usize) {
        let role_get = self.roles.remove(&role);
        if role_get.is_none() {
            return;
        }
        let role = role_get.unwrap();
        let role_assigned = role.assigned();
        for i in 0..role_assigned.len() {
            let x = role_assigned[i];
            if x.eq(&0) {
                continue;
            }
            let employee = self.employees.get_mut(&x);
            if employee.is_none() {
                continue;
            }
            employee.unwrap().remove_block(vec![i]);
        }
    }
    pub fn delete_employee(&mut self, emp: usize) {
        let emp_get = self.employees.remove(&emp);
        if emp_get.is_none() {
            return;
        }
        let emp = emp_get.unwrap();
        for (_,role) in self.roles.iter_mut() {
            role.clear_employee(&emp.id);
        }
    }

    pub fn update_business_hours(&mut self, open: NaiveTime, close: NaiveTime) {
        self.open = open;
        self.close = close;
        self.blocks = 0;
        let mut test_time = self.open.clone();
        let mut empty_vec: Vec<Vec<usize>> = vec![];
        while test_time < self.close {
            test_time += self.block_size;
            self.blocks += 1;
            empty_vec.push(vec![0]);
        }
        for (_, role) in self.roles.iter_mut() {
            role.assigned_set(empty_vec.clone());
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
    pub fn toggle_employee_scheduled(&mut self, emp_id: usize) {
        let employee = self.employees.get_mut(&emp_id).expect("Attempted to toggle scheduled on non-existant employee");
        employee.scheduled = !employee.scheduled;
    }
    pub fn assign_role(&mut self, emp: usize, role: usize) {
        let employee = self.employees.get_mut(&emp).unwrap();
        employee.add_role(role);
    }
    pub fn restrict_role(&mut self, emp: usize, role: usize) -> Result<()> {
        let employee = self.employees.get_mut(&emp).unwrap();
        let role = self.roles.get_mut(&role).unwrap();
        match employee.remove_role(role.id()) {
            Ok(_) => {role.clear_employee(&emp); Ok(())},
            Err(e) => Err(e)
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
        match emp.assign_block(blocks, role.id()) {
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
                        role.remove_block(&employee, vec![index]);
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
            role_get.unwrap().remove_block(&employee, vec![index]);
        }
        Ok(())
    }
}

dyn_clone::clone_trait_object!(Role);
pub trait Role: DynClone + std::fmt::Debug {
    fn new(id: usize, name: AttrValue, blocks: usize) -> Self where Self: Sized;
    // fn new_with_assigned(id: usize, name: AttrValue, assigned: Vec<usize>, empty: bool) -> Self where Self: Sized;
    /// Only to be used when future init of assigned is planned
    fn new_blank(id: usize, name: AttrValue, color: AttrValue) -> Self where Self: Sized;

    /// Replaces the employee ID at the given indexes, returning a list of (replaced employee id, at time index)
    fn add_block(&mut self, id: &usize, indexes: Vec<usize>) -> Vec<(usize, usize)>;
    fn remove_block(&mut self, emp: &usize, indexes: Vec<usize>);
    /// Completely remove all of an employee's scheduled times from the role
    fn clear_employee(&mut self, id: &usize);

    // Getters and setters
    fn id(&self) -> usize;
    fn name(&self) -> AttrValue;
    fn sort(&self) -> usize;
    fn sort_set(&mut self, sort: usize);
    /// Public for debug purposes only, table of roles will likely not be visible in final product
    fn assigned(&self) -> Vec<usize>;
    fn assigned_set(&mut self, assigned: Vec<Vec<usize>>);
    fn color(&self) -> AttrValue;
    fn color_set(&mut self, color: String);
    fn is_empty(&self) -> bool;
} impl dyn Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort().cmp(&other.sort())
            .then(self.name().cmp(&other.name()))
            .then(self.id().cmp(&other.id()))
    }
}
impl PartialEq for Box<dyn Role> {
    fn eq(&self, other: &Self) -> bool {
        // self.0 == other.0 && self.1 == other.1
        self.id() == other.id() &&
        self.name() == other.name() &&
        self.sort() == other.sort() &&
        self.assigned() == other.assigned() &&
        self.color() == other.color()
    }
}
impl Eq for Box<dyn Role> {}
impl PartialOrd for Box<dyn Role> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Box<dyn Role> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort().cmp(&other.sort())
            .then(self.name().cmp(&other.name()))
            .then(self.id().cmp(&other.id()))
    }
}

#[derive(Clone, PartialEq, Debug, Properties)]
struct SingleRole {
    id: usize,
    name: AttrValue,
    sort: usize,
    assigned: Vec<usize>,
    color: AttrValue,
    empty: bool
} impl Role for SingleRole {
    fn new(id: usize, name: AttrValue, blocks: usize) -> SingleRole {
        let mut assigned = vec![];
        for _ in 0..blocks {
            assigned.push(0);
        }
        SingleRole { id: id.clone(), name: name, sort: id, assigned: assigned, color: DEFAULT_COLOR.into(), empty: true }
    }
    // fn new_with_assigned(id: usize, name: AttrValue, assigned: Vec<usize>, empty: bool) -> SingleRole {
    //     SingleRole { id: id.clone(), name: name, sort: id, assigned: assigned, color: DEFAULT_COLOR.into(), empty: empty }
    // }
    fn new_blank(id: usize, name: AttrValue, color: AttrValue) -> SingleRole {
        SingleRole { id: id.clone(), name: name, sort: id, assigned: vec![], color: color, empty: true }
    }

    fn add_block(&mut self, id: &usize, indexes: Vec<usize>) -> Vec<(usize, usize)> {
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
    fn remove_block(&mut self, _emp: &usize, indexes: Vec<usize>) {
        for index in indexes {
            self.assigned[index] = 0;
        }
    }
    fn clear_employee(&mut self, id: &usize) {
        for i in 0..self.assigned.len() {
            if self.assigned[i].eq(id) {
                self.assigned[i] = 0;
            }
        }
    }

    fn id(&self) -> usize {self.id.clone()}
    fn name(&self) -> AttrValue {self.name.clone()}
    fn sort(&self) -> usize {self.sort.clone()}
    fn sort_set(&mut self, sort: usize) {self.sort = sort;}
    fn assigned(&self) -> Vec<usize> {self.assigned.clone()}
    fn assigned_set(&mut self, assigned: Vec<Vec<usize>>) {
        self.assigned = {
            let flat: Vec<usize> = assigned.clone().into_iter().flatten().collect();
            if flat.len().eq(&assigned.len()) {
                flat
            } else {
                panic!("Improper init of role! Tried to assign {:#?} to role {}", assigned, self.name);
            }
        };
    }
    fn color(&self) -> AttrValue {self.color.clone()}
    fn color_set(&mut self, color: String) {self.color = color.into();}
    fn is_empty(&self) -> bool {self.empty}

    // fn cmp(&self, other: &impl Role) -> std::cmp::Ordering {
    //     self.sort()
    // }
} impl SingleRole {

}

#[derive(Clone, PartialEq, Debug)]
struct MultiRole {
    id: usize,
    name: AttrValue,
    sort: usize,
    assigned: Vec<Vec<usize>>,
    color: AttrValue,
    empty: bool
} impl Role for MultiRole {
    fn new(id: usize, name: AttrValue, blocks: usize) -> Self where Self: Sized {
        let mut assigned = vec![];
        for _ in 0..blocks {
            assigned.push(vec![0]);
        }
        MultiRole { id: id.clone(), name, sort: id, assigned, color: DEFAULT_COLOR.into(), empty: true }
    }
    // fn new_with_assigned(id: usize, name: AttrValue, assigned: Vec<usize>, empty: bool) -> Self where Self: Sized {
    //     MultiRole { id: id.clone(), name, sort: id, assigned, color: DEFAULT_COLOR.into(), empty }
    // }
    fn new_blank(id: usize, name: AttrValue, color: AttrValue) -> Self where Self: Sized {
        MultiRole { id: id.clone(), name, sort: id, assigned: vec![], color, empty: true }
    }

    fn add_block(&mut self, id: &usize, indexes: Vec<usize>) -> Vec<(usize, usize)> {
        for index in indexes {
            let block_get = self.assigned.get_mut(index);
            if let Some(block) = block_get {
                if !block.contains(id) {
                    block.push(id.clone());
                }
            }
        }
        if self.empty {
            self.empty = false;
        }
        vec![]
    }
    fn remove_block(&mut self, emp: &usize, indexes: Vec<usize>) {
        for time_block_index in indexes {
            if let Some(time_block) = self.assigned.get_mut(time_block_index) {
                for i in 0..time_block.len() {
                    if time_block[i].eq(emp) {
                        time_block.remove(i);
                        break;
                    }
                }
            }
        }
    }
    fn clear_employee(&mut self, id: &usize) {
        for block in self.assigned.iter_mut() {
            for i in block.len()..0 {
                if block[i].eq(id) {
                    block.remove(i);
                }
            }
        }
    }

    fn id(&self) -> usize {
        self.id.clone()
    }
    fn name(&self) -> AttrValue {
        self.name.clone()
    }
    fn sort(&self) -> usize {
        self.sort.clone()
    }
    fn sort_set(&mut self, sort: usize) {
        self.sort = sort;
    }
    fn assigned(&self) -> Vec<usize> {
        self.assigned.clone().into_iter().flatten().collect()
    }
    fn assigned_set(&mut self, assigned: Vec<Vec<usize>>) {
        self.assigned = assigned;
    }
    fn color(&self) -> AttrValue {
        self.color.clone()
    }
    fn color_set(&mut self, color: String) {
        self.color = color.into();
    }
    fn is_empty(&self) -> bool {
        self.empty
    }
}

// pub type Result<T> = std::result::Result<T, EmployeeError>;

#[derive(Debug)]
pub enum EmployeeError {
    NotAssignedRole { failed: usize, allowed: Vec<usize> },
    NotClockedIn

}

#[derive(Default)]
pub enum EmployeeSort {
    #[default]
    Name,
    ClockIn,
    ClockOut,
    /// Sort by when an employee is schduled to work a role
    Assigned { id: usize },
    /// Sort by if an employee is allowed to work a role
    Role { id: usize }
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
        Employee { id: id, name: name, roles: vec![2], scheduled: true, lunch: 2, clock_in, clock_out, assigned: vec![] }
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
                    if role.ne(x) {
                        successful_indexes.push(*index);
                        swapped_roles.push((*x, *index));
                        self.assigned[*index] = role;
                    }
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
            if curr > 2 { // A role that isn't lunch
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
    pub fn remove_role(&mut self, role: usize) -> Result<()> {
        // let index_find = self.roles.iter().find(|&&x| x.eq(&role));
        // match index_find {
        //     None => return Err(BusinessError::EmployeeError(EmployeeError::NotAssignedRole { failed: role, allowed: self.roles.clone() })),
        //     Some(index) => {
        //         let index = index.clone();
        //         self.roles.remove(index.try_into().unwrap());
        //     }
        // }
        let mut failed = true;
        for i in 0..self.roles.len() {
            let item = &self.roles[i];
            if role.eq(item) {
                self.roles.remove(i);
                failed = false;
            }
        }
        if failed {
            return Err(BusinessError::EmployeeError(EmployeeError::NotAssignedRole { failed: role, allowed: self.roles.clone() }))
        }
        for i in 0..self.assigned.len() {
            // let assigned_get = self.assigned.get(i);
            if let Some(assigned) = self.assigned.get(i) {
                if role.eq(assigned) {
                    self.remove_block(vec![i]);
                }
            }
        }
        Ok(())
    }

    pub fn cmp(&self, other: &Employee, order: EmployeeSort) -> std::cmp::Ordering{
        self.scheduled.cmp(&other.scheduled).reverse() // true > false, but we want scheduled employees first
        .then(match order {
            EmployeeSort::Name => std::cmp::Ordering::Equal,
            EmployeeSort::ClockIn => self.clock_in.cmp(&other.clock_in),
            EmployeeSort::ClockOut => self.clock_out.cmp(&other.clock_out),
            EmployeeSort::Assigned { id } => {
                for i in 0..self.assigned.len() {
                    if self.assigned[i] != id && other.assigned[i] != id {
                        continue;
                    } else if self.assigned[i] == id {
                        return std::cmp::Ordering::Less
                    } else if other.assigned[i] == id {
                        return std::cmp::Ordering::Greater
                    }
                }
                std::cmp::Ordering::Equal
            }
            EmployeeSort::Role { id } => self.roles.contains(&id).cmp(&other.roles.contains(&id)),
        }).then(self.name.cmp(&other.name))
        .then(self.id.cmp(&other.id))
    }
}