// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;

use app::infra::db::init_db;

fn main() {
  // init database saat app start
  if let Err(e) = init_db() {
      eprintln!("Failed to init DB: {}", e);
  }

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::cmd_list_employees,
        commands::cmd_add_employee,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
