use serde::{Deserialize, Serialize};

/// Ringkasan presensi per bulan untuk 1 karyawan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresensiSummary {
    pub total_hadir: i64,
    pub total_sakit: i64,
    pub total_cuti: i64,
    pub total_absen: i64,
}
