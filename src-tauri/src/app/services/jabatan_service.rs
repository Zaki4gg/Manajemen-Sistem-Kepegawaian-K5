use crate::app::domain::jabatan::{Jabatan, NewJabatan};
use crate::app::infra::supabase::Supabase;
use urlencoding::encode;

/// Ambil semua jabatan dari Supabase
pub async fn list_jabatan() -> Result<Vec<Jabatan>, String> {
    let sb = Supabase::new();

    sb.get_json::<Vec<Jabatan>>(
        "jabatan?select=*&order=nama.asc",
        "Supabase list jabatan error",
    )
    .await
}

/// Tambah jabatan baru
pub async fn add_jabatan(jabatan: NewJabatan) -> Result<(), String> {
    let sb = Supabase::new();

    sb.insert_json(
        "jabatan",
        &jabatan,
        "Supabase insert jabatan error",
    )
    .await
}

/// Update jabatan (pakai nama sebagai key)
pub async fn update_jabatan(nama: String, jabatan: NewJabatan) -> Result<(), String> {
    let sb = Supabase::new();

    let path = format!("jabatan?nama=eq.{}", encode(&nama));

    sb.patch_json(
        &path,
        &jabatan,
        "Supabase update jabatan error",
    )
    .await
}

/// Hapus jabatan berdasarkan nama
pub async fn delete_jabatan(nama: String) -> Result<(), String> {
    let sb = Supabase::new();

    let path = format!("jabatan?nama=eq.{}", encode(&nama));

    sb.delete(
        &path,
        "Supabase delete jabatan error",
    )
    .await
}
