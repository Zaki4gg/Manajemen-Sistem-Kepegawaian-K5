// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::cmd_list_employees,
            commands::cmd_add_employee,
            commands::cmd_admin_login,
            commands::cmd_update_employee,
            commands::cmd_delete_employee,
            commands::cmd_list_jabatan,
            commands::cmd_add_jabatan,
            commands::cmd_update_jabatan,
            commands::cmd_delete_jabatan,
            commands::cmd_list_presensi,
            commands::cmd_get_presensi_summary, // <- TAMBAHAN
            commands::cmd_upsert_presensi,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
