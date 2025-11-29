use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::domain::admin::Admin;
use crate::app::domain::jabatan::{Jabatan, NewJabatan};
use crate::app::domain::presensi::{Presensi, NewPresensi};
use crate::app::domain::presensi_summary::PresensiSummary;
use crate::app::services::{
    employee_service,
    admin_service,
    jabatan_service,
    presensi_service,
};

#[tauri::command]
pub async fn cmd_list_employees() -> Result<Vec<Employee>, String> {
    employee_service::list_employees().await
}

#[tauri::command]
pub async fn cmd_add_employee(new_emp: NewEmployee) -> Result<(), String> {
    employee_service::add_employee(new_emp).await
}

#[tauri::command]
pub async fn cmd_admin_login(email: String, password: String) -> Result<Admin, String> {
    admin_service::login_admin(email, password).await
}

#[tauri::command]
pub async fn cmd_update_employee(employee: Employee) -> Result<(), String> {
    employee_service::update_employee(employee).await
}

#[tauri::command]
pub async fn cmd_delete_employee(id: i64) -> Result<(), String> {
    employee_service::delete_employee(id).await
}

#[tauri::command]
pub async fn cmd_list_jabatan() -> Result<Vec<Jabatan>, String> {
    jabatan_service::list_jabatan().await
}

#[tauri::command]
pub async fn cmd_add_jabatan(jabatan: NewJabatan) -> Result<(), String> {
    jabatan_service::add_jabatan(jabatan).await
}

#[tauri::command]
pub async fn cmd_update_jabatan(nama: String, jabatan: NewJabatan) -> Result<(), String> {
    jabatan_service::update_jabatan(nama, jabatan).await
}

#[tauri::command]
pub async fn cmd_delete_jabatan(nama: String) -> Result<(), String> {
    jabatan_service::delete_jabatan(nama).await
}

#[tauri::command]
pub async fn cmd_list_presensi(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<Vec<Presensi>, String> {
    presensi_service::list_presensi_for_employee_month(employee_id, year, month).await
}

#[tauri::command]
pub async fn cmd_get_presensi_summary(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<PresensiSummary, String> {
    presensi_service::get_presensi_summary_for_employee_month(employee_id, year, month).await
}

#[tauri::command]
pub async fn cmd_upsert_presensi(presensi: NewPresensi) -> Result<(), String> {
    presensi_service::upsert_presensi(presensi).await
}
