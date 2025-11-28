# Manajemen Sistem Kepegawaian K5
_A Functional Programming Approach with Rust_  
**Authors:** Bagas Yoga Pratama Pramudika, Michael Peter Valentino Situmeang, Muhammad Zaki Afriza, Rafi Baydar Athaillah

---

## Abstract

Proyek ini bertujuan untuk mengembangkan Sistem Manajemen Karyawan menggunakan Bahasa Pemrograman Rust dan Framework Tauri. Sistem ini dirancang untuk menangani operasi CRUD (Create, Read, Update, Delete) terkait data karyawan, jabatan, serta pencatatan presensi harian. Backend aplikasi dibangun menggunakan Rust untuk memanfaatkan keamanan memori dan performa tingginya, sementara antarmuka pengguna berbasis desktop menggunakan Tauri. Penyimpanan data ditangani secara eksternal menggunakan Supabase melalui komunikasi REST API.

---

## Introduction

Pengelolaan data kepegawaian di banyak organisasi kecil masih dilakukan secara manual, misalnya dengan spreadsheet atau catatan terpisah. Hal ini menyulitkan ketika jumlah karyawan bertambah, karena:

- Data mudah tidak sinkron (antara presensi, gaji, dan data karyawan).
- Pencarian dan rekap membutuhkan waktu lama.
- Rentan terhadap kesalahan input dan penghitungan.

Project ini mencoba menyelesaikan masalah tersebut dengan sebuah aplikasi Manajemen Sistem Kepegawaian yang terintegrasi:

- Menyimpan data karyawan secara terstruktur.
- Mengelola presensi (hadir/tidak hadir) melalui tampilan kalender.
- Menjadi dasar untuk fitur penggajian dan laporan.

**Mengapa Rust?**

Rust digunakan pada proyek ini karena jaminan keamanan memori (memory safety) tanpa memerlukan garbage collector. Hal ini memastikan aplikasi berjalan dengan performa yang dapat diprediksi. Selain itu, sistem tipe data Rust yang kuat membantu mencegah runtime errors sejak tahap kompilasi.

**Mengapa memasukkan konsep Functional Programming?**

- Mempermudah reasoning terhadap logika bisnis karena fungsi dibuat pure (output hanya ditentukan oleh input).
- Mendorong immutability sehingga bug terkait shared state bisa dikurangi.
- Memanfaatkan iterators dan higher-order functions seperti `map`, `filter` untuk memproses list data karyawan dan presensi dengan cara yang deklaratif.

**Keunikan solusi:**

Aplikasi kami menggabungkan kecepatan Rust di belakang layar dengan tampilan modern yang ringan. Kami juga tidak perlu menginstall database ribet di komputer, karena semua data tersimpan aman di cloud (Supabase).

---

## Background and Concepts

### Technology Stack

- **Rust**
  Bahasa pemrograman sistem yang digunakan untuk menangani seluruh logika backend aplikasi. Rust dipilih karena kemampuannya mengelola memori secara aman tanpa Garbage Collector, menjamin performa tinggi dan stabilitas aplikasi.

- **Tauri**
  Framework untuk membangun aplikasi desktop yang sangat ringan. Tauri bekerja dengan memanfaatkan WebView bawaan sistem operasi untuk merender tampilan, sementara logika intinya dijalankan langsung oleh Rust. Pendekatan ini memungkinkan pembuatan aplikasi yang aman, sangat efisien dalam penggunaan memori, dan memiliki ukuran installer yang kecil.

- **Supabase**
  Platform Backend-as-a-Service (BaaS) yang menyediakan database PostgreSQL secara cloud. Dalam proyek ini, Supabase bertindak sebagai pusat penyimpanan data yang diakses melalui protokol REST API, sehingga kita tidak perlu mengelola server database lokal.

- **Tokio**
  Runtime asinkron (Asynchronous Runtime) untuk Rust. Tokio bertugas menangani operasi berat seperti permintaan jaringan (network request) di latar belakang (background thread), sehingga antarmuka aplikasi tidak macet (freeze) saat menunggu balasan dari server.

- **Reqwest**
  HTTP Client untuk Rust yang mudah digunakan. Library ini berfungsi untuk melakukan panggilan API (GET, POST, PATCH, DELETE) ke server Supabase, mengirimkan data input pengguna, dan menerima respon dari server.

- **Serde**
  Framework untuk memproses data JSON. Serde bertugas menerjemahkan objek data Rust (Struct) menjadi format JSON agar bisa dikirim ke API, dan sebaliknya menerjemahkan respon JSON dari API menjadi objek Rust yang bisa diolah oleh program.

### Functional Programming Concepts

Beberapa konsep functional programming yang menjadi panduan desain:

- **Immutability**  
  - Variabel menggunakan `let` sebisa mungkin tanpa `mut`.
  - Data karyawan diproses dengan membuat salinan baru (misal `map`/`filter`) daripada memodifikasi in-place.

- **Pure Functions**  
  - Fungsi bisnis seperti perhitungan total gaji atau filter karyawan aktif dibuat tanpa side effect:
    - Tidak membaca/menulis ke global state.
    - Hanya menerima parameter dan mengembalikan nilai.

- **Higher-Order Functions & Iterators**  
  - Penggunaan method seperti `.iter()`, `.map()`, `.filter()`, `.fold()` di koleksi karyawan dan presensi.
  - Mengurangi penggunaan loop imperatif dan index manual.

- **Pattern Matching**  
  - `match` pada `Result<T, E>` untuk menangani keberhasilan/gagalnya operasi database atau I/O.
  - Menghindari banyak `if-else` dan membuat alur error handling lebih jelas.

---

## Source Code and Explanation

### Main dan Konfigurasi

- **main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::cmd_list_employees,
            commands::cmd_add_employee,
            commands::cmd_admin_login,
            commands::cmd_update_employee,
            commands::cmd_delete_employee,
            commands::cmd_list_jabatan,
            commands::cmd_add_jabatan,
            commands::cmd_update_jabatan,
            commands::cmd_delete_jabatan,
            commands::cmd_list_presensi,
            commands::cmd_upsert_presensi,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

Penjelasan:
- Baris pertama menunjukkan atribut untuk seluruh crate yang mengatur perilaku aplikasi ketika dijalankan di Windows
- Baris ke tiga dan empat bertujuan untuk memanggil modul app dan coomand.
- Di fungsi `main`, aplikasi Tauri dibangun menggunakan pola builder. `tauri::Builder::default()` membuat instance default, lalu memanggil `.invoke_handler(tauri::generate_handler![ ... ])` untuk mendaftarkan fungsi-fungsi rust yang akan di ekspor sebagai command ke front-end Tauri.
- `tauri::generate_handler!` menerima daftar fungsi dari modul `commands`.
- `.run(tauri::generate_context!())` akan dipanggil dan membaca konfigurasi dari `tauri.conf.json` lalu menjalankan event loop Tauri: membuka window aplikasi, menghubungkan event, dan menangani command sampai aplikasi ditutup.
- Hasil dari `run` adalah `Result`, sehingga diakhiri dengan `expect(...);` yang akan membuat Tauri "Panic" dengan pesan itu, sehingga kita tau kalau ada masalah ketika kita menjalankan aplikasi.

- **lib.rs**

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- **commands.rs**

```rust
use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::services::{employee_service, admin_service};
use crate::app::domain::admin::Admin;      
use crate::app::domain::jabatan::{Jabatan, NewJabatan};
use crate::app::services::jabatan_service;
use crate::app::domain::presensi::{Presensi, NewPresensi};
use crate::app::services::presensi_service;


#[tauri::command]
pub async fn cmd_list_employees() -> Result<Vec<Employee>, String> {
    employee_service::list_employees().await
}

#[tauri::command]
pub async fn cmd_add_employee(new_emp: NewEmployee) -> Result<(), String> {
    employee_service::add_employee(new_emp).await
}

#[tauri::command]
pub async fn cmd_admin_login(email: String, password: String) -> Result<Admin, String> {
    admin_service::login_admin(email, password).await
}

#[tauri::command]
pub async fn cmd_update_employee(employee: Employee) -> Result<(), String> {
    employee_service::update_employee(employee).await
}

#[tauri::command]
pub async fn cmd_delete_employee(id: i64) -> Result<(), String> {
    employee_service::delete_employee(id).await
}

#[tauri::command]
pub async fn cmd_list_jabatan() -> Result<Vec<Jabatan>, String> {
    jabatan_service::list_jabatan().await
}

#[tauri::command]
pub async fn cmd_add_jabatan(jabatan: NewJabatan) -> Result<(), String> {
    jabatan_service::add_jabatan(jabatan).await
}

#[tauri::command]
pub async fn cmd_update_jabatan(nama: String, jabatan: NewJabatan) -> Result<(), String> {
    jabatan_service::update_jabatan(nama, jabatan).await
}

#[tauri::command]
pub async fn cmd_delete_jabatan(nama: String) -> Result<(), String> {
    jabatan_service::delete_jabatan(nama).await
}

#[tauri::command]
pub async fn cmd_list_presensi(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<Vec<Presensi>, String> {
    presensi_service::list_presensi_for_employee_month(employee_id, year, month).await
}

#[tauri::command]
pub async fn cmd_upsert_presensi(presensi: NewPresensi) -> Result<(), String> {
    presensi_service::upsert_presensi(presensi).await
}
```

- **src/app/mod.rs**

```rust
pub mod domain;
pub mod infra;
pub mod services;
```

- **build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

### Domain

- **admin.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub email: String,
}
```

- **employee.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: i64,
    pub nik: String,
    pub name: String,
    pub department: String,
    pub position: String,
    pub base_salary: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEmployee {
    pub nik: String,
    pub name: String,
    pub department: String,
    pub position: String,
    pub base_salary: i64,
}

```

- **jabatan.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jabatan {
    pub nama: String,
    pub tunjangan: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJabatan {
    pub nama: String,
    pub tunjangan: f64,
}

```

- **presensi.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presensi {
    pub id: i64,
    pub employee_id: i64,
    pub tanggal: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPresensi {
    pub employee_id: i64,
    pub tanggal: String,
    pub status: String,
}

```

- **src/app/domain/mod.rs**

```rust
pub mod employee;
pub mod admin;
pub mod jabatan;
pub mod presensi;
```

### Infrastructure

- **supabase.rs**

```rust
use dotenvy::dotenv;
use reqwest::Client;
use std::env;

pub struct Supabase {
    pub url: String,
    pub anon_key: String,
}

impl Supabase {
    pub fn new() -> Self {
        dotenv().ok();

        let url = env::var("SUPABASE_URL").expect("SUPABASE_URL not set");
        let anon_key = env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY not set");

        Self { url, anon_key }
    }

    pub fn client(&self) -> Client {
        Client::new()
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!("{}/rest/v1/{}", self.url.trim_end_matches('/'), path)
    }
}
```

- **src/app/infra/mod.rs**

```rust
pub mod supabase;
```

### Services

- **admin_service.rs**

```rust
use crate::app::domain::admin::Admin;
use crate::app::infra::supabase::Supabase;

pub async fn login_admin(email: String, password: String) -> Result<Admin, String> {
    let sb = Supabase::new();

    let query = format!(
        "admin?select=email&email=eq.{email}&password=eq.{password}&limit=1"
    );
    let url = sb.endpoint(&query);

    let res = sb
        .client()
        .get(url)
        .header("apikey", &sb.anon_key)
        .header("Authorization", format!("Bearer {}", sb.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Supabase login error {status}: {text}"));
    }

    let admins: Vec<Admin> = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    admins
        .into_iter()
        .next()
        .ok_or_else(|| "Email atau password salah".to_string())
}
```

- **employee_service.rs**

```rust
use crate::app::domain::employee::{Employee, NewEmployee};
use crate::app::infra::supabase::Supabase;
use serde_json::json;

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
```

- **jabatan_service.rs**

```rust
use crate::app::domain::jabatan::{Jabatan, NewJabatan};
use crate::app::infra::supabase::Supabase;
use urlencoding::encode;
use serde_json::json;

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
```

- **presensi_service.rs**

```rust
use crate::app::domain::presensi::{Presensi, NewPresensi};
use crate::app::infra::supabase::Supabase;
use chrono::NaiveDate;
use serde_json::json;

pub async fn list_presensi_for_employee_month(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<Vec<Presensi>, String> {
    let sb = Supabase::new();

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

pub async fn upsert_presensi(presensi: NewPresensi) -> Result<(), String> {
    let sb = Supabase::new();

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
```

- **src/app/services/mod.rs**

```rust
pub mod employee_service;
pub mod admin_service;
pub mod jabatan_service;
pub mod presensi_service;
```


## Screenshot

- **Halaman Login**

![Login Page](login_page.png)

- **Tampilan Tab Data Karyawan**

![Tambah Karyawan](tambah_karyawan.png)

![Manajemen Jabatan dan Tunjangannya](manajemen_jabatan.png)

![Daftar Karyawan](daftar_karyawan.png)

- **Tampilan Tab Laporan Gaji dan Jadwal**

![Data Presensi Karyawan](data_presensi_karyawan.png)

![Data Gaji Karyawan](data_gaji_karyawan.png)

## Conclusion

Berdasarkan hasil implementasi, pengembangan Sistem Manajemen Karyawan ini menunjukkan bahwa kombinasi Rust dan Tauri mampu menghasilkan aplikasi desktop yang jauh lebih ringan dan cepat dibandingkan solusi berbasis web biasa. Walaupun penerapan aturan memori Rust dan konsep pemrograman fungsional terasa ketat dan menantang saat penulisan kode, hal ini terbukti sangat efektif dalam mencegah bug fatal (seperti aplikasi menutup sendiri) ketika dijalankan.

Selain itu, penggunaan Supabase sebagai backend sangat membantu menyederhanakan arsitektur sistem karena kami tidak perlu membangun server database lokal yang ribet. Secara keseluruhan, sistem ini telah memenuhi kebutuhan fungsionalitas pengelolaan data karyawan dengan performa yang stabil dan penggunaan sumber daya komputer yang sangat minim.
