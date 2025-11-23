use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::infra::{db::get_connection, employee_repo::EmployeeRepo};

pub fn list_employees() -> Result<Vec<Employee>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;
    let repo = EmployeeRepo { conn: &conn };
    repo.list_all().map_err(|e| e.to_string())
}

pub fn add_employee(new_emp: NewEmployee) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;
    let repo = EmployeeRepo { conn: &conn};
    repo.insert(&new_emp).map_err(|e| e.to_string())    
}