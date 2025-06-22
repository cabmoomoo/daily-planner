use std::collections::HashMap;

use chrono::{NaiveTime, TimeDelta};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use yew::{AttrValue, Properties};

use crate::persistence::read_settings;

const DEFAULT_COLOR: &'static str = "#AAC406";

pub type Result<T> = std::result::Result<T, BusinessError>;
#[derive(Debug)]
pub enum BusinessError {
    EmployeeNotFound,
    RoleNotFound,
    EmployeeError(EmployeeError)
} impl std::fmt::Display for BusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusinessError::EmployeeNotFound => write!(f, "Employee not found"),
            BusinessError::RoleNotFound => write!(f, "Role not found"),
            BusinessError::EmployeeError(employee_error) => write!(f, "{}", employee_error),
        }
    }
}

#[derive(Clone, PartialEq, Properties, Debug, Serialize, Deserialize)]
pub struct Business {
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub roles: HashMap<usize, Role>,
    pub employees: HashMap<usize, Employee>,
    // In production, blocks and block_size may not need to be public
    pub blocks: usize, // The number of blocks within the current business hours
    pub block_size: TimeDelta, // The size of the schedule's blocks
    #[serde(skip)]
    pub role_colors: HashMap<usize, AttrValue>
} impl Business {
    pub fn init(&mut self) {
        for (_, role) in self.roles.iter() {
            self.role_colors.insert(role.id(), role.color());
        }
        self.update_business_hours(self.open, self.close);
        // business
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
        self.roles.insert(id.clone(), SingleRole::new(id, name, self.blocks).into());
        self.role_colors.insert(id.clone(), self.roles[&id].color());
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
        self.employees.insert(id.clone(), Employee::new(id, name, self.open.clone(), self.close.clone()).new_blank(self.blocks));
    }
    pub fn delete_role(&mut self, role: usize) {
        let role_get = self.roles.remove(&role);
        if role_get.is_none() {
            return;
        }
        let role = role_get.unwrap();
        let role_assigned = match role.assigned() {
            RoleAssigned::SingleAssinged(items) => vec![items],
            RoleAssigned::MultiAssigned(items) => items,
        };
        // let role_assigned = role.assigned();
        for time_slot in role_assigned {
            for i in 0..time_slot.len() {
                let x = time_slot[i];
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
        let empty_vec_enum = RoleAssigned::MultiAssigned(empty_vec);
        for (_, role) in self.roles.iter_mut() {
            role.assigned_set(empty_vec_enum.clone());
        }
        for (_, employee) in self.employees.iter_mut() {
            if employee.clock_in < self.open {employee.clock_in = self.open.clone()};
            if employee.clock_out > self.close || employee.clock_out < self.open {employee.clock_out = self.close.clone()};
            employee.clear_assigned(&self.open, &self.close, self.block_size.clone());
        }
    }
    pub fn update_role_color(&mut self, role_id: usize, color: AttrValue) {
        if let Some(role) =  self.roles.get_mut(&role_id) {
            role.color_set(color.clone());
            self.role_colors.insert(role_id, color);
        }
    }
    pub fn toggle_role_multi(&mut self, role_id: usize) {
        let role = match self.roles.remove(&role_id) {
            Some(x) => x,
            None => return,
        };
        let new_role = match role {
            Role::SingleRole(single_role) => {
                MultiRole::new(single_role.id(), single_role.name(), self.blocks).into()
            },
            Role::MultiRole(multi_role) => {
                SingleRole::new(multi_role.id(), multi_role.name(), self.blocks).into()
            },
        };
        self.roles.insert(role_id, new_role);
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
        let emp = match self.employees.get_mut(&employee) {
            Some(emp) => emp,
            None => return Err(BusinessError::EmployeeNotFound),
        };
        let role = match self.roles.get_mut(&role) {
            Some(x) => x,
            None => return Err(BusinessError::RoleNotFound),
        };
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

// dyn_clone::clone_trait_object!(RoleTrait);
#[enum_dispatch]
pub trait RoleTrait: std::fmt::Debug {
    // fn new(id: usize, name: AttrValue, blocks: usize) -> Self where Self: Sized;
    // // fn new_with_assigned(id: usize, name: AttrValue, assigned: Vec<usize>, empty: bool) -> Self where Self: Sized;
    // /// Only to be used when future init of assigned is planned
    // fn new_blank(id: usize, name: AttrValue, color: AttrValue) -> Self where Self: Sized;

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
    fn is_multi(&self) -> bool;
    /// Public for debug purposes only, table of roles will likely not be visible in final product
    fn assigned(&self) -> RoleAssigned;
    fn assigned_set(&mut self, assigned: RoleAssigned);
    fn color(&self) -> AttrValue;
    fn color_set(&mut self, color: AttrValue);
    fn is_empty(&self) -> bool;

    fn blank_out(&mut self, blocks: usize);
}

#[enum_dispatch(RoleTrait)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    SingleRole,
    MultiRole
}

impl PartialEq for Role {
    fn eq(&self, other: &Self) -> bool {
        // self.0 == other.0 && self.1 == other.1
        self.id() == other.id() &&
        self.name() == other.name() &&
        self.sort() == other.sort() &&
        self.assigned() == other.assigned() &&
        self.color() == other.color()
    }
}
impl Eq for Role {}
impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort().cmp(&other.sort())
            .then(self.name().cmp(&other.name()))
            .then(self.id().cmp(&other.id()))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RoleAssigned {
    SingleAssinged(Vec<usize>),
    MultiAssigned(Vec<Vec<usize>>)
} impl Into<Vec<usize>> for RoleAssigned {
    fn into(self) -> Vec<usize> {
        match self {
            RoleAssigned::SingleAssinged(items) => items,
            RoleAssigned::MultiAssigned(items) => items.into_iter().flatten().collect(),
        }
    }
} impl Into<Vec<Vec<usize>>> for RoleAssigned {
    fn into(self) -> Vec<Vec<usize>> {
        match self {
            RoleAssigned::SingleAssinged(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(vec![item]);
                }
                result
            },
            RoleAssigned::MultiAssigned(items) => items,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Properties, Serialize, Deserialize)]
struct SingleRole {
    id: usize,
    name: AttrValue,
    sort: usize,
    #[serde(skip)]
    assigned: Vec<usize>,
    color: AttrValue,
    #[serde(skip)]
    empty: bool
} impl RoleTrait for SingleRole {

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
    fn is_multi(&self) -> bool {false}
    fn assigned(&self) -> RoleAssigned {RoleAssigned::SingleAssinged(self.assigned.clone())}
    fn assigned_set(&mut self, assigned: RoleAssigned) {
        self.assigned = match assigned {
            RoleAssigned::MultiAssigned(items) => {
                        let flat: Vec<usize> = items.clone().into_iter().flatten().collect();
                        if flat.len().eq(&items.len()) {
                            flat
                        } else {
                            panic!("Improper init of role! Tried to assign {:#?} to role {}", items, self.name);
                        }
                    }
            RoleAssigned::SingleAssinged(items) => items,
        };
    }
    fn color(&self) -> AttrValue {self.color.clone()}
    fn color_set(&mut self, color: AttrValue) {self.color = color;}
    fn is_empty(&self) -> bool {self.empty}

    fn blank_out(&mut self, blocks: usize) {
        self.assigned = vec![0;blocks];
    }
} impl SingleRole {
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
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct MultiRole {
    id: usize,
    name: AttrValue,
    sort: usize,
    #[serde(skip)]
    assigned: Vec<Vec<usize>>,
    color: AttrValue,
    #[serde(skip)]
    empty: bool
} impl RoleTrait for MultiRole {

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
    fn is_multi(&self) -> bool {
        true
    }
    fn assigned(&self) -> RoleAssigned {
        RoleAssigned::MultiAssigned(self.assigned.clone())
    }
    fn assigned_set(&mut self, assigned: RoleAssigned) {
        self.assigned = match assigned {
            RoleAssigned::SingleAssinged(items) => {
                let mut multi_vec = vec![];
                for item in items {
                    multi_vec.push(vec![item]);
                }
                multi_vec
            },
            RoleAssigned::MultiAssigned(items) => items,
        };
    }
    fn color(&self) -> AttrValue {
        self.color.clone()
    }
    fn color_set(&mut self, color: AttrValue) {
        self.color = color;
    }
    fn is_empty(&self) -> bool {
        self.empty
    }

    fn blank_out(&mut self,blocks:usize) {
        let mut assigned = Vec::new();
        for _ in 0..blocks {
            assigned.push(Vec::new());
        }
        self.assigned = assigned;
    }
} impl MultiRole {
    fn new(id: usize, name: AttrValue, blocks: usize) -> Self where Self: Sized {
        let mut assigned = vec![];
        for _ in 0..blocks {
            assigned.push(vec![]);
        }
        MultiRole { id: id.clone(), name, sort: id, assigned, color: DEFAULT_COLOR.into(), empty: true }
    }
}

#[derive(Debug)]
pub enum EmployeeError {
    NotAssignedRole { failed: usize, allowed: Vec<usize> },
    NotClockedIn
} impl std::fmt::Display for EmployeeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmployeeError::NotAssignedRole { failed, allowed } => {
                write!(f, "Employee not assigned to role {}; can be assigned: ", failed)?;
                for allowed in allowed {
                    write!(f, "{},", allowed)?;
                }
                write!(f, "")
            },
            EmployeeError::NotClockedIn => write!(f, "Employee not clocked in"),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
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

#[derive(Clone, PartialEq, Debug, Properties, Serialize, Deserialize)]
pub struct Employee {
    pub id: usize,
    pub name: AttrValue,
    pub roles: Vec<usize>, // Roles an employee can be assigned to
    #[serde(skip)]
    pub scheduled: bool,
    pub lunch: usize, // Length of lunch break in 30-minute intervals (1 hour is 2)
    #[serde(skip)]
    pub clock_in: NaiveTime,
    #[serde(skip)]
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
    #[serde(skip)]
    pub assigned: Vec<usize> // Roles an employee is currently assigned to work
} impl Employee {
    pub fn new(id: usize, name: AttrValue, clock_in: NaiveTime, clock_out: NaiveTime) -> Employee {
        Employee { id: id, name: name, roles: vec![2], scheduled: true, lunch: 2, clock_in, clock_out, assigned: vec![] }
    }
    pub fn new_blank(mut self, blocks: usize) -> Self {
        self.assigned = vec![1;blocks];
        self
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
            if curr > 1 { 
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
                break;
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

    /// Helper function to find the first block where the employee is clocked in, but not currently assigned a role
    /// 
    /// Returns None when no such block exists
    pub fn first_open(&self) -> Option<usize> {
        for i in 0..self.assigned.len() {
            if let Some(role) = self.assigned.get(i) {
                if 1.eq(role) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn assign_area(&mut self, role: &mut Role, time_index: usize, preferred_length: usize) {
        for i in 0..preferred_length {
            match self.assigned.get(time_index + i) {
                None => return,
                Some(curr_role) => {
                    if 1.eq(curr_role) {
                        self.assigned[time_index + i] = role.id();
                        role.add_block(&self.id, vec![time_index+i]);
                    } else {
                        return;
                    }
                }
            }
        }
    }

    pub fn deschedule(&mut self, blocks: usize) {
        self.scheduled = false;
        self.assigned = vec![0;blocks];
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


fn business_base() -> Role {
    Role::MultiRole(MultiRole::new(2, "Lunch".into(), 0))
}

impl Business {
    /// Generate a sample business with 3 roles and 4 employees
    pub fn sample() -> Business {
        let open = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let close = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
        let role_vec = vec![
            business_base(),
            SingleRole::new_blank(3, "Role 1".into(), "#00AAFF".into()).into(),
            SingleRole::new_blank(4, "Role 2".into(), "#C70039".into()).into(),
            SingleRole::new_blank(5, "Role 3".into(), "#11E000".into()).into()
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
                name: "Employee 1".into(), 
                roles: vec![2,3,4],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 2, 
                name: "Employee 2".into(), 
                roles: vec![2,3,4,5],
                scheduled: true,
                lunch: 1, 
                clock_in: open.clone(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 3, 
                name: "Employee 3".into(), 
                roles: vec![2,3,4,5],
                scheduled: true,
                lunch: 2, 
                clock_in: open.clone(),
                clock_out: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                assigned: vec![]
            },
            Employee { 
                id: 4, 
                name: "Employee 4".into(), 
                roles: vec![2,3,4,5],
                scheduled: true,
                lunch: 2, 
                clock_in: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                clock_out: close.clone(),
                assigned: vec![]
            },
            Employee { 
                id: 5, 
                name: "Employee 5".into(), 
                roles: vec![2,3,4,5],
                scheduled: false,
                lunch: 2, 
                clock_in: open.clone(),
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
        // business.schedule_lunch();
        // business.schedule_roles();
        business
    }
}