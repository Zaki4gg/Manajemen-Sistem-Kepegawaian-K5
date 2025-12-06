use crate::app::domain::presensi::{Presensi, NewPresensi};
use crate::app::domain::presensi_summary::PresensiSummary;
use crate::app::infra::supabase::Supabase;
use chrono::NaiveDate;

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
    let query = format!(
        "presensi?select=*&employee_id=eq.{employee_id}\
        &tanggal=gte.{start_date}&tanggal=lte.{end_date}&order=tanggal.asc"
    );

    // pakai helper Supabase
    sb.get_json::<Vec<Presensi>>(
        &query,
        "Supabase list presensi error",
    )
    .await
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
    let path = "presensi?on_conflict=employee_id,tanggal";

    sb.upsert_json(
        path,
        &presensi,
        "Supabase upsert presensi error",
    )
    .await
}
