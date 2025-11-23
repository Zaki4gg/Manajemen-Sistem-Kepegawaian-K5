use crate::app::services::employee_service;
use crate::app::domain::employee::{Employee, NewEmployee};

#[tauri::command]
pub fn cmd_list_employees() -> Result<Vec<Employee>, String> {
    employee_service::list_employees()
}

#[tauri::command]
pub fn cmd_add_employee(newEmp: NewEmployee) -> Result<(), String> {
    employee_service::add_employee(newEmp)
}