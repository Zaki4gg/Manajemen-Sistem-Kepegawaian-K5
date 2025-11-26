use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::infra::supabase::Supabase;
use serde_json::json;

/// Ambil semua karyawan dari Supabase
pub async fn list_employees() -> Result<Vec<Employee>, String> {
    let sb = Supabase::new();
    let url = sb.endpoint("employees?select=*&order=id.asc");

    let res = sb
        .client()
        .get(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("Supabase error {status}: {text}"));
    }

    let data: Vec<Employee> =
        serde_json::from_str(&text).map_err(|e| format!("parse error: {e} | body: {text}"))?;

    Ok(data)
}

/// Tambah satu karyawan baru ke Supabase
pub async fn add_employee(new_emp: NewEmployee) -> Result<(), String> {
    let sb = Supabase::new();
    let url = sb.endpoint("employees");

    let body = json!({
        "nik": new_emp.nik,
        "name": new_emp.name,
        "department": new_emp.department,
        "position": new_emp.position,
        "base_salary": new_emp.base_salary,
    });

    let res = sb
        .client()
        .post(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Supabase insert error {status}: {text}"));
    }

    Ok(())
}
pub async fn update_employee(emp: Employee) -> Result<(), String> {
    let sb = Supabase::new();

    // PATCH /rest/v1/employees?id=eq.<id>
    let url = sb.endpoint(&format!("employees?id=eq.{}", emp.id));

    let body = json!({
        "nik": emp.nik,
        "name": emp.name,
        "department": emp.department,
        "position": emp.position,
        "base_salary": emp.base_salary,
    });

    let res = sb
        .client()
        .patch(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Supabase update error: {}", res.status()))
    }
}

pub async fn delete_employee(id: i64) -> Result<(), String> {
    let sb = Supabase::new();

    // DELETE /rest/v1/employees?id=eq.<id>
    let url = sb.endpoint(&format!("employees?id=eq.{}", id));

    let res = sb
        .client()
        .delete(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Supabase delete error: {}", res.status()))
    }
}