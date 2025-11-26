use crate::app::domain::jabatan::{Jabatan, NewJabatan};
use crate::app::infra::supabase::Supabase;
use urlencoding::encode;
use serde_json::json;

/// Ambil semua jabatan dari Supabase
pub async fn list_jabatan() -> Result<Vec<Jabatan>, String> {
    let sb = Supabase::new();
    let url = sb.endpoint("jabatan?select=*&order=nama.asc");

    let res = sb
        .client()
        .get(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Supabase list jabatan error: {}", res.status()));
    }

    let data = res.json::<Vec<Jabatan>>().await.map_err(|e| e.to_string())?;
    Ok(data)
}

/// Tambah jabatan baru
pub async fn add_jabatan(jabatan: NewJabatan) -> Result<(), String> {
    let sb = Supabase::new();
    let url = sb.endpoint("jabatan");

    let body = json!({
        "nama": jabatan.nama,
        "tunjangan": jabatan.tunjangan,
    });

    let res = sb
        .client()
        .post(url)
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
        Err(format!("Supabase insert jabatan error: {}", res.status()))
    }
}

/// Update jabatan (kalau nama boleh diubah, kita pakai nama untuk where)
pub async fn update_jabatan(nama: String, jabatan: NewJabatan) -> Result<(), String> {
    let sb = Supabase::new();
    let url = sb.endpoint(&format!(
        "jabatan?nama=eq.{}",
        urlencoding::encode(&nama)
    ));

    let body = json!({
        "nama": jabatan.nama,
        "tunjangan": jabatan.tunjangan,
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
        Err(format!("Supabase update jabatan error: {}", res.status()))
    }
}

/// Hapus jabatan berdasarkan nama
pub async fn delete_jabatan(nama: String) -> Result<(), String> {
    let sb = Supabase::new();
    let url = sb.endpoint(&format!(
        "jabatan?nama=eq.{}",
        encode(&nama)
    ));

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
        Err(format!("Supabase delete jabatan error: {}", res.status()))
    }
}
