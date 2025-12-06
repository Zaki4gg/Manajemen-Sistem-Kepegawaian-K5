use crate::app::domain::employee::Employee;
use crate::app::domain::jabatan::Jabatan;
use crate::app::domain::presensi_summary::PresensiSummary;
use crate::app::services::{employee_service, jabatan_service, presensi_service};

use chrono::{Datelike, Duration, NaiveDate};
use genpdf::Element;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use once_cell::sync::Lazy;

static SUPABASE_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(12));

pub enum GenerateMode {
    SingleCore,
    MultiCore,
}

pub async fn generate_slips_jan_2025_to_dec_2026(
    root_dir: &str,
    mode: GenerateMode,
) -> Result<(), String> {
    let employees = employee_service::list_employees().await?;
    let jabatans_vec = jabatan_service::list_jabatan().await?;
    let jabatans = Arc::new(jabatans_vec);

    let periods: Vec<(i32, u32)> = (2025..=2026)
        .flat_map(|year| (1..=12).map(move |month| (year, month)))
        .collect();

    fs::create_dir_all(root_dir).map_err(|e| e.to_string())?;

    match mode {
        GenerateMode::SingleCore => {
            for emp in employees {
                generate_slips_for_employee(emp, &jabatans, &periods, root_dir).await?;
            }
        }
        GenerateMode::MultiCore => {
            use tokio::task;

            let mut handles = Vec::new();

            for emp in employees {
                let jabatans = Arc::clone(&jabatans);
                let periods = periods.clone();
                let root = root_dir.to_string();
                let emp_name = emp.name.clone();

                let handle = task::spawn(async move {
                    if let Err(e) =
                        generate_slips_for_employee(emp, &jabatans, &periods, &root).await
                    {
                        eprintln!("[generate_slips_multi] {}: {}", emp_name, e);
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.await;
            }
        }
    }

    Ok(())
}

pub async fn generate_yearly_slips_2025_2026(
    root_dir: &str,
    mode: GenerateMode,
) -> Result<(), String> {
    let employees = employee_service::list_employees()
        .await
        .map_err(|e| format!("gagal load karyawan: {e}"))?;

    let jabatans = Arc::new(
        jabatan_service::list_jabatan()
            .await
            .map_err(|e| format!("gagal load jabatan: {e}"))?,
    );

    let years = vec![2025, 2026];

    fs::create_dir_all(root_dir)
        .map_err(|e| format!("gagal membuat folder root slip tahunan: {e}"))?;

    match mode {
        GenerateMode::SingleCore => {
            for emp in employees {
                generate_yearly_slips_for_employee(
                    emp,
                    &years,
                    Arc::clone(&jabatans),
                    root_dir,
                )
                .await?;
            }
        }
        GenerateMode::MultiCore => {
            let mut handles = Vec::new();

            for emp in employees {
                let jabatans_clone = Arc::clone(&jabatans);
                let years_clone = years.clone();
                let root_clone = root_dir.to_string();

                let handle = tokio::spawn(async move {
                    if let Err(e) = generate_yearly_slips_for_employee(
                        emp,
                        &years_clone,
                        jabatans_clone,
                        &root_clone,
                    )
                    .await
                    {
                        eprintln!("gagal generate slip tahunan: {e}");
                    }
                });

                handles.push(handle);
            }

            for h in handles {
                if let Err(e) = h.await {
                    eprintln!("task join error: {e}");
                }
            }
        }
    }

    Ok(())
}


async fn generate_slips_for_employee(
    emp: Employee,
    jabatans: &Arc<Vec<Jabatan>>,
    periods: &[(i32, u32)],
    root_dir: &str,
) -> Result<(), String> {
    let emp_dir = build_employee_dir(root_dir, &emp);
    fs::create_dir_all(&emp_dir).map_err(|e| e.to_string())?;

    for (year, month) in periods {
        let summary: PresensiSummary =
            get_presensi_summary_limited(emp.id, *year, *month as i32).await?;

        let working_days = count_working_days_in_month(*year, *month)?;
        let periode_text = build_periode_text(*year, *month)?;

        let slip = build_slip_data(&emp, jabatans, &periode_text, &summary, working_days);

        let filename = format!("slip-gaji-{}-{:04}-{:02}.pdf", emp.nik, year, month);
        let path = emp_dir.join(filename);

        write_payslip_pdf(&slip, &path)?;
    }

    Ok(())
}

async fn generate_yearly_slips_for_employee(
    emp: Employee,
    years: &[i32],
    jabatans: Arc<Vec<Jabatan>>,
    root_dir: &str,
) -> Result<(), String> {
    let emp_dir = build_employee_dir(root_dir, &emp);
    fs::create_dir_all(&emp_dir)
        .map_err(|e| format!("gagal buat folder karyawan: {e}"))?;

    for &year in years {
        let mut monthly_slips = Vec::new();

        for month in 1..=12_u32 {
            let periode = build_periode_text(year, month)?;
            let summary =
                get_presensi_summary_limited(emp.id, year, month as i32).await?;
            let working_days = count_working_days_in_month(year, month)?;

            let slip = build_slip_data(
                &emp,
                &jabatans,
                &periode,
                &summary,
                working_days,
            );

            monthly_slips.push(slip);
        }

        if let Some(yearly_data) = build_yearly_slip_data(year, &monthly_slips) {
            let file_name = format!("slip-gaji-tahunan-{}-{}.pdf", emp.nik, year);
            let path = emp_dir.join(file_name);

            write_yearly_payslip_pdf(&yearly_data, &path)
                .map_err(|e| format!("gagal tulis PDF tahunan: {e}"))?;
        }
    }

    Ok(())
}



fn build_employee_dir(root_dir: &str, emp: &Employee) -> PathBuf {
    let mut name_slug: String = emp
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '_'
            } else {
                '-'
            }
        })
        .collect();

    if name_slug.is_empty() {
        name_slug = "unknown".to_string();
    }

    Path::new(root_dir).join(format!("{}_{}", emp.nik, name_slug))
}

pub struct SlipData {
    pub periode: String,
    pub nama: String,
    pub nik: String,
    pub jabatan: String,
    pub departemen: String,

    pub gaji_pokok: f64,
    pub tunjangan_gaji: f64,
    pub total_pendapatan: f64,
    pub asuransi_kesehatan: f64,
    pub total_potongan: f64,
    pub gaji_setelah_asuransi: f64,

    pub total_hadir: i64,
    pub total_sakit: i64,
    pub total_cuti: i64,
    pub total_absen: i64,
    pub hari_kerja: u32,
    pub total_hadir_efektif: f64,
    pub faktor_kehadiran: f64,

    pub gaji_bersih_diterima: f64,
}

#[derive(Debug, Clone)]
pub struct YearlySlipMonthRow {
    pub bulan_nama: String,

    pub total_hadir: i64,
    pub total_sakit: i64,
    pub total_cuti: i64,
    pub total_absen: i64,
    pub hari_kerja: u32,
    pub total_hadir_efektif: f64,

    pub gaji_setelah_asuransi: f64,
    pub gaji_bersih_diterima: f64,
}

#[derive(Debug, Clone)]
pub struct YearlySlipData {
    pub tahun: i32,

    pub nama: String,
    pub nik: String,
    pub jabatan: String,
    pub departemen: String,

    // komponen gaji per bulan (konstan per bulan)
    pub gaji_pokok: f64,
    pub tunjangan_gaji: f64,
    pub total_pendapatan: f64,
    pub asuransi_kesehatan: f64,
    pub gaji_setelah_asuransi: f64,

    pub rows: Vec<YearlySlipMonthRow>,

    // ringkasan setahun
    pub total_kehadiran: i64,
    pub total_kehadiran_efektif: f64,
    pub total_hari_kerja: u32,
    pub total_gaji_bersih: f64,
}

fn build_slip_data(
    emp: &Employee,
    jabatans: &Arc<Vec<Jabatan>>,
    periode: &str,
    summary: &PresensiSummary,
    working_days: u32,
) -> SlipData {
    let gaji_pokok = emp.base_salary as f64;

    let jabatan_row = jabatans.iter().find(|j| j.nama == emp.position);
    let tunjangan_gaji = jabatan_row.map(|j| j.tunjangan).unwrap_or(0.0);

    let asuransi_kesehatan = 450_000.0;

    let total_pendapatan = gaji_pokok + tunjangan_gaji;
    let total_potongan = asuransi_kesehatan;
    let gaji_setelah_asuransi = total_pendapatan - total_potongan;

    let total_hadir = summary.total_hadir;
    let total_sakit = summary.total_sakit;
    let total_cuti = summary.total_cuti;
    let total_absen = summary.total_absen;

    let total_hadir_efektif =
        total_hadir as f64 * 1.0 + total_sakit as f64 * 0.8 + total_cuti as f64 * 0.4;

    let hari_kerja = working_days;

    let mut faktor_kehadiran = 1.0;
    let mut gaji_bersih_diterima = gaji_setelah_asuransi;

    if hari_kerja > 0 {
        faktor_kehadiran = total_hadir_efektif / hari_kerja as f64;

        if !faktor_kehadiran.is_finite() {
            faktor_kehadiran = 0.0;
        }
        if faktor_kehadiran < 0.0 {
            faktor_kehadiran = 0.0;
        }
        if faktor_kehadiran > 1.0 {
            faktor_kehadiran = 1.0;
        }

        gaji_bersih_diterima = gaji_setelah_asuransi * faktor_kehadiran;
    }

    SlipData {
        periode: periode.to_string(),
        nama: emp.name.clone(),
        nik: emp.nik.clone(),
        jabatan: emp.position.clone(),
        departemen: emp.department.clone(),

        gaji_pokok,
        tunjangan_gaji,
        total_pendapatan,
        asuransi_kesehatan,
        total_potongan,
        gaji_setelah_asuransi,

        total_hadir,
        total_sakit,
        total_cuti,
        total_absen,
        hari_kerja,
        total_hadir_efektif,
        faktor_kehadiran,

        gaji_bersih_diterima,
    }
}

fn build_yearly_slip_data(tahun: i32, slips: &[SlipData]) -> Option<YearlySlipData> {
    if slips.is_empty() {
        return None;
    }

    let first = &slips[0];

    let mut rows = Vec::with_capacity(slips.len());

    let mut total_kehadiran: i64 = 0;
    let mut total_kehadiran_efektif: f64 = 0.0;
    let mut total_hari_kerja: u32 = 0;
    let mut total_gaji_bersih: f64 = 0.0;

    for (idx, s) in slips.iter().enumerate() {
        let bulan_index = (idx as u32) + 1;
        let bulan_nama = MONTH_NAMES_ID[(bulan_index - 1) as usize].to_string();

        rows.push(YearlySlipMonthRow {
            bulan_nama,

            total_hadir: s.total_hadir,
            total_sakit: s.total_sakit,
            total_cuti: s.total_cuti,
            total_absen: s.total_absen,
            hari_kerja: s.hari_kerja,
            total_hadir_efektif: s.total_hadir_efektif,

            gaji_setelah_asuransi: s.gaji_setelah_asuransi,
            gaji_bersih_diterima: s.gaji_bersih_diterima,
        });

        total_kehadiran += s.total_hadir;
        total_kehadiran_efektif += s.total_hadir_efektif;
        total_hari_kerja += s.hari_kerja;
        total_gaji_bersih += s.gaji_bersih_diterima;
    }

    Some(YearlySlipData {
        tahun,

        nama: first.nama.clone(),
        nik: first.nik.clone(),
        jabatan: first.jabatan.clone(),
        departemen: first.departemen.clone(),

        gaji_pokok: first.gaji_pokok,
        tunjangan_gaji: first.tunjangan_gaji,
        total_pendapatan: first.total_pendapatan,
        asuransi_kesehatan: first.asuransi_kesehatan,
        gaji_setelah_asuransi: first.gaji_setelah_asuransi,

        rows,
        total_kehadiran,
        total_kehadiran_efektif,
        total_hari_kerja,
        total_gaji_bersih,
    })
}



static MONTH_NAMES_ID: [&str; 12] = [
    "Januari",
    "Februari",
    "Maret",
    "April",
    "Mei",
    "Juni",
    "Juli",
    "Agustus",
    "September",
    "Oktober",
    "November",
    "Desember",
];

fn build_periode_text(year: i32, month: u32) -> Result<String, String> {
    let month_index = (month - 1) as usize;
    let month_name = MONTH_NAMES_ID
        .get(month_index)
        .ok_or_else(|| "Bulan di luar jangkauan".to_string())?;

    let last_day = last_day_of_month(year, month)?;
    let first_str = "01";
    let last_str = format!("{:02}", last_day);

    Ok(format!("{first_str}–{last_str} {month_name} {year}"))
}

fn last_day_of_month(year: i32, month: u32) -> Result<u32, String> {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "Tanggal tidak valid".to_string())?;

    let last = first_next - Duration::days(1);
    Ok(last.day())
}

fn count_working_days_in_month(year: i32, month: u32) -> Result<u32, String> {
    let mut day = 1;
    let mut count = 0;

    loop {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => {
                let weekday = date.weekday().number_from_monday();
                if (1..=5).contains(&weekday) {
                    count += 1;
                }
                day += 1;
            }
            None => break,
        }
    }

    Ok(count)
}

fn format_rupiah_i64(amount: i64) -> String {
    let neg = amount < 0;
    let s: String = amount.abs().to_string();
    let mut result = String::new();

    let mut count = 0;
    for ch in s.chars().rev() {
        if count != 0 && count % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
        count += 1;
    }

    let mut formatted: String = result.chars().rev().collect();
    if neg {
        formatted.insert(0, '-');
    }

    formatted
}

pub trait ToRupiah {
    fn rp(&self) -> String;
}

impl ToRupiah for i64 {
    fn rp(&self) -> String {
        format_rupiah_i64(*self)
    }
}

impl ToRupiah for f64 {
    fn rp(&self) -> String {
        // kalau mau dibuletin dulu:
        let val = self.round() as i64;
        format_rupiah_i64(val)
    }
}

impl ToRupiah for u64 {
    fn rp(&self) -> String {
        format_rupiah_i64(*self as i64)
    }
}

fn write_payslip_pdf(data: &SlipData, path: &Path) -> Result<(), String> {
    use genpdf::elements::{Break, Paragraph};
    use genpdf::{Alignment, Document, style};

    // base path: folder src-tauri (crate ini)
    let mut fonts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fonts_dir.push("fonts");

    let font_family = genpdf::fonts::from_files(&fonts_dir, "LiberationSans", None)
        .map_err(|e| format!("Gagal load font di {:?}: {}", fonts_dir, e))?;

    let mut doc = Document::new(font_family);
    doc.set_title("Struk Gaji Karyawan");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let bold = style::Style::new().bold();

    // ====== HEADER ======
    doc.push(
        Paragraph::new("Struk Gaji Karyawan")
            .aligned(Alignment::Left)
            .styled(bold),
    );
    doc.push(Break::new(1));

    doc.push(Paragraph::new(format!("Periode: {}", data.periode)));
    doc.push(Paragraph::new(format!("Nama: {}", data.nama)));
    doc.push(Paragraph::new(format!("NIK: {}", data.nik)));
    doc.push(Paragraph::new(format!("Jabatan: {}", data.jabatan)));
    doc.push(Paragraph::new(format!("Departemen: {}", data.departemen)));

    doc.push(Break::new(1));

    // ====== KOMPONEN GAJI ======
    doc.push(Paragraph::new("Komponen Gaji").styled(bold));
    doc.push(Paragraph::new(format!(
        "Gaji Pokok: Rp {}",
        data.gaji_pokok.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Tunjangan Gaji: Rp {}",
        data.tunjangan_gaji.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Total Pendapatan: Rp {}",
        data.total_pendapatan.rp()
    )));

    doc.push(Break::new(1));

    // ====== POTONGAN ======
    doc.push(Paragraph::new("Potongan").styled(bold));
    doc.push(Paragraph::new(format!(
        "Asuransi Kesehatan: Rp {}",
        data.asuransi_kesehatan.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Total Potongan: Rp {}",
        data.total_potongan.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Gaji setelah potongan asuransi: Rp {}",
        data.gaji_setelah_asuransi.rp()
    )));

    doc.push(Break::new(1));

    // ====== REKAP PRESENSI ======
    doc.push(Paragraph::new("Rekap Presensi Bulan Ini").styled(bold));
    doc.push(Paragraph::new(format!(
        "Total kehadiran: {}",
        data.total_hadir
    )));
    doc.push(Paragraph::new(format!("Total sakit: {}", data.total_sakit)));
    doc.push(Paragraph::new(format!("Total cuti: {}", data.total_cuti)));
    doc.push(Paragraph::new(format!("Total absen: {}", data.total_absen)));
    doc.push(Paragraph::new(format!(
        "Total kehadiran efektif: {:.1} dari {} hari kerja",
        data.total_hadir_efektif, data.hari_kerja
    )));
    doc.push(Paragraph::new(format!(
        "Faktor kehadiran: {:.2}%",
        data.faktor_kehadiran * 100.0
    )));

    doc.push(Break::new(1));

    // ====== RINGKASAN GAJI ======
    doc.push(Paragraph::new(format!(
        "Gaji setelah potongan asuransi: Rp {}",
        data.gaji_setelah_asuransi.rp()
    )));
    doc.push(
        Paragraph::new(format!(
            "Gaji Bersih Diterima (berdasarkan kehadiran): Rp {}",
            data.gaji_bersih_diterima.rp()
        ))
        .styled(bold),
    );

    doc.render_to_file(path).map_err(|e| e.to_string())
}

fn write_yearly_payslip_pdf(
    data: &YearlySlipData,
    path: &Path,
) -> Result<(), String> {
    use genpdf::elements::{Break, Paragraph};
    use genpdf::{Alignment, Document, style};

    let mut fonts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fonts_dir.push("fonts");

    let font_family = genpdf::fonts::from_files(&fonts_dir, "LiberationSans", None)
        .map_err(|e| format!("Gagal load font di {:?}: {}", fonts_dir, e))?;

    let mut doc = Document::new(font_family);
    doc.set_title(format!("Struk Gaji Tahunan {}", data.nama));

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let bold = style::Style::new().bold();

    // Header
    doc.push(
        Paragraph::new("Struk Gaji Tahunan Karyawan")
            .aligned(Alignment::Center)
            .styled(bold),
    );
    doc.push(Break::new(1));
    doc.push(Paragraph::new(format!("Tahun: {}", data.tahun)));
    doc.push(Break::new(1));

    doc.push(Paragraph::new(format!("Nama: {}", data.nama)));
    doc.push(Paragraph::new(format!("NIK: {}", data.nik)));
    doc.push(Paragraph::new(format!("Jabatan: {}", data.jabatan)));
    doc.push(Paragraph::new(format!("Departemen: {}", data.departemen)));

    doc.push(Break::new(1));
    doc.push(Paragraph::new("Komponen Gaji (per bulan)").styled(bold));

    doc.push(Paragraph::new(format!(
        "Gaji Pokok: Rp {}",
        data.gaji_pokok.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Tunjangan Gaji: Rp {}",
        data.tunjangan_gaji.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Total Pendapatan: Rp {}",
        data.total_pendapatan.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Asuransi Kesehatan: Rp {}",
        data.asuransi_kesehatan.rp()
    )));
    doc.push(Paragraph::new(format!(
        "Gaji setelah potongan asuransi: Rp {}",
        data.gaji_setelah_asuransi.rp()
    )));

    doc.push(Break::new(1));
    doc.push(Paragraph::new("Rekap Presensi & Gaji per Bulan").styled(bold));

    for row in &data.rows {
        doc.push(Break::new(1));

        // Judul bulan
        doc.push(
            Paragraph::new(&row.bulan_nama)
                .styled(bold),
        );

        // Baris-baris detail
        doc.push(Paragraph::new(format!(
            "Total kehadiran: {}",
            row.total_hadir
        )));
        doc.push(Paragraph::new(format!(
            "Total sakit: {}",
            row.total_sakit
        )));
        doc.push(Paragraph::new(format!(
            "Total cuti: {}",
            row.total_cuti
        )));
        doc.push(Paragraph::new(format!(
            "Total absen: {}",
            row.total_absen
        )));
        doc.push(Paragraph::new(format!(
            "Total kehadiran efektif: {:.1} dari {} hari kerja",
            row.total_hadir_efektif,
            row.hari_kerja
        )));
        doc.push(Paragraph::new(format!(
            "Gaji akhir bulan: Rp {}",
            row.gaji_bersih_diterima.rp()
        )));
    }


    doc.push(Break::new(1));
    doc.push(Paragraph::new("Ringkasan Tahun Ini").styled(bold));
    doc.push(Paragraph::new(format!(
        "Total kehadiran: {} hari",
        data.total_kehadiran
    )));
    doc.push(Paragraph::new(format!(
        "Total kehadiran efektif: {:.1} dari {} hari kerja",
        data.total_kehadiran_efektif,
        data.total_hari_kerja
    )));
    doc.push(Paragraph::new(format!(
        "Total gaji bersih dibayarkan setahun: Rp {}",
        data.total_gaji_bersih.rp()
    )));

    doc.render_to_file(path)
        .map_err(|e| e.to_string())?;

    Ok(())
}


async fn get_presensi_summary_limited(
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<PresensiSummary, String> {
    use tokio::time::{sleep, Duration};

    let mut attempt = 0;

    loop {
        attempt += 1;

        // ambil "izin" 1 slot request Supabase
        let permit = SUPABASE_SEMAPHORE
            .acquire()
            .await
            .map_err(|e| format!("Semaphore error: {}", e))?;

        let result = presensi_service::get_presensi_summary_for_employee_month(
            employee_id,
            year,
            month,
        )
        .await;

        // lepas slot
        drop(permit);

        match result {
            Ok(summary) => return Ok(summary),
            Err(e) => {
                // kalau error koneksi, coba retry beberapa kali
                let is_last = attempt >= 3;
                eprintln!(
                    "[supabase] error attempt {} for emp {} {}-{}: {}",
                    attempt, employee_id, year, month, e
                );

                if is_last {
                    return Err(format!("Gagal ambil presensi setelah {}x: {}", attempt, e));
                } else {
                    // backoff sedikit supaya tidak nembak terus
                    sleep(Duration::from_millis(300 * attempt as u64)).await;
                }
            }
        }
    }
}
