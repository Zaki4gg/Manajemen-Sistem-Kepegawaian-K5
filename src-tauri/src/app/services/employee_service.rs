use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::infra::supabase::Supabase;

/// Ambil semua karyawan dari Supabase
pub async fn list_employees() -> Result<Vec<Employee>, String> {
    let sb = Supabase::new();

    sb.get_json::<Vec<Employee>>(
        "employee?select=*&order=id.asc",
        "Supabase list employees error",
    )
    .await
}

/// Tambah satu karyawan baru ke Supabase
pub async fn add_employee(new_emp: NewEmployee) -> Result<(), String> {
    let sb = Supabase::new();

    sb.insert_json(
        "employee",
        &new_emp,
        "Supabase insert employee error",
    )
    .await
}

/// Update data karyawan berdasar id
pub async fn update_employee(emp: Employee) -> Result<(), String> {
    let sb = Supabase::new();
    let path = format!("employee?id=eq.{}", emp.id);
    let payload = NewEmployee {
        nik: emp.nik,
        name: emp.name,
        department: emp.department,
        position: emp.position,
        base_salary: emp.base_salary,
    };

    sb.patch_json(
        &path,
        &payload,
        "Supabase update employee error",
    )
    .await
}

pub async fn delete_employee(id: i64) -> Result<(), String> {
    let sb = Supabase::new();

    let path = format!("employee?id=eq.{id}");

    sb.delete(
        &path,
        "Supabase delete employee error",
    )
    .await
}
