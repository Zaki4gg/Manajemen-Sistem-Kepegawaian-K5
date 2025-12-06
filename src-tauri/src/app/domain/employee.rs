use serde::{Deserialize, Serialize};

macro_rules! define_employee_types {
    ($($common:tt)*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Employee {
            pub id: i64, // kolom id (bigint) di Supabase
            $($common)*
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct NewEmployee {
            $($common)*
        }
    };
}

define_employee_types! {
    pub nik: String,
    pub name: String,
    pub department: String,
    pub position: String,
    pub base_salary: i64,
}