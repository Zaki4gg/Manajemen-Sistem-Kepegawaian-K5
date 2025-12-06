use serde::{Deserialize, Serialize};

macro_rules! define_presensi_types {
    ($($common:tt)*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Presensi {
            pub id: i64,
            $($common)*
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct NewPresensi {
            $($common)*
        }
    };
}

define_presensi_types! {
    pub employee_id: i64,
    pub tanggal: String,
    pub status: String,
}
