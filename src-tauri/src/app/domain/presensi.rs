use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presensi {
    pub id: i64,
    pub employee_id: i64,   // SAMA dengan kolom di Supabase
    pub tanggal: String,    // "YYYY-MM-DD"
    pub status: String,     // "hadir" | "sakit" | "cuti" | "absen"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPresensi {
    pub employee_id: i64,
    pub tanggal: String,
    pub status: String,
}
