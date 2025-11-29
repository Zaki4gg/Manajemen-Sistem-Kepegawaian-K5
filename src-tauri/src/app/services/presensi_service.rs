use crate::app::domain::presensi::{Presensi, NewPresensi};
use crate::app::domain::presensi_summary::PresensiSummary;
use crate::app::infra::supabase::Supabase;
use chrono::NaiveDate;
use serde_json::json;

pub async fn list_presensi_for_employee_month(
    employee_id: i64,
    year: i32,
    month: i32, // 1..=12
) -> Result<Vec<Presensi>, String> {
    let sb = Supabase::new();

    // tanggal awal & akhir bulan
    let first = NaiveDate::from_ymd_opt(year, month as u32, 1)
        .ok_or_else(|| "Tanggal awal tidak valid".to_string())?;
    let last = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .ok_or_else(|| "Tanggal akhir tidak valid".to_string())?
            .pred_opt()
            .ok_or_else(|| "Tanggal akhir tidak valid".to_string())?
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1)
            .ok_or_else(|| "Tanggal akhir tidak valid".to_string())?
            .pred_opt()
            .ok_or_else(|| "Tanggal akhir tidak valid".to_string())?
    };

    let start_date = first.format("%Y-%m-%d").to_string();
    let end_date = last.format("%Y-%m-%d").to_string();

    // filter langsung di query
    let url = sb.endpoint(&format!(
        "presensi?select=*&employee_id=eq.{employee_id}\
        &tanggal=gte.{start_date}&tanggal=lte.{end_date}&order=tanggal.asc"
    ));

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
        return Err(format!("Supabase list presensi error {status}: {text}"));
    }

    let items: Vec<Presensi> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(items)
}

/// HITUNG ringkasan presensi per bulan di BACKEND
pub async fn get_presensi_summary_for_employee_month(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<PresensiSummary, String> {
    let items = list_presensi_for_employee_month(employee_id, year, month).await?;

    let mut summary = PresensiSummary {
        total_hadir: 0,
        total_sakit: 0,
        total_cuti: 0,
        total_absen: 0,
    };

    for p in items {
        match p.status.as_str() {
            "hadir" => summary.total_hadir += 1,
            "sakit" => summary.total_sakit += 1,
            "cuti"  => summary.total_cuti  += 1,
            "absen" => summary.total_absen += 1,
            other => {
                eprintln!("[get_presensi_summary] status tak dikenal: {}", other);
            }
        }
    }

    Ok(summary)
}

/// Insert / update presensi (upsert) berdasarkan (employee_id, tanggal)
pub async fn upsert_presensi(presensi: NewPresensi) -> Result<(), String> {
    let sb = Supabase::new();

    // kolom unik (employee_id, tanggal) sudah kamu buat di SQL
    let url = sb.endpoint("presensi?on_conflict=employee_id,tanggal");

    let payload = json!({
        "employee_id": presensi.employee_id,
        "tanggal": presensi.tanggal,
        "status": presensi.status,
    });

    let res = sb
        .client()
        .post(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();

    if status.is_success() {
        Ok(())
    } else {
        let body = res.text().await.unwrap_or_default();
        Err(format!("Supabase upsert presensi error: {} {}", status, body))
    }
}
