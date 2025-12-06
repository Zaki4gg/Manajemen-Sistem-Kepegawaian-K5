use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jabatan {
    pub nama: String,
    pub tunjangan: f64, // numeric(10,2) -> bisa pakai f64
}

pub type NewJabatan = Jabatan;
