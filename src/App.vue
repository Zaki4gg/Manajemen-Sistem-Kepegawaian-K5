<script setup>
import { ref, reactive, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AdminLogin from "./AdminLogin.vue";
import "./App.css";

const MONTH_NAMES_ID = [
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

const MIN_YEAR = 2025;
const MAX_YEAR = 2026;

// ======================
// Helper
// ======================

function buildPeriodeText(monthIndex, year) {
  const monthName = MONTH_NAMES_ID[monthIndex] ?? "";
  const firstDay = 1;
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();

  const firstStr = String(firstDay).padStart(2, "0");
  const lastStr = String(lastDay).padStart(2, "0");

  return `${firstStr}–${lastStr} ${monthName} ${year}`;
}

// Hitung jumlah hari kerja (Senin–Jumat) dalam 1 bulan
function countWorkingDaysInMonth(year, monthIndex) {
  const daysInMonth = new Date(year, monthIndex + 1, 0).getDate();
  let count = 0;
  for (let day = 1; day <= daysInMonth; day += 1) {
    const date = new Date(year, monthIndex, day);
    const dow = date.getDay(); // 0=Minggu, 6=Sabtu
    if (dow >= 1 && dow <= 5) {
      count += 1;
    }
  }
  return count;
}

// Hitung struk gaji berdasarkan data employees + jabatan + presensi
function hitungStrukGaji(
  employee,
  jabatanList,
  periode,
  attendanceSummary,
  workingDays,
) {
  const gajiPokok = Number(employee.base_salary ?? 0);

  const jabatanRow =
    jabatanList?.find((j) => j.nama === employee.position) || null;

  const tunjanganGaji = jabatanRow ? Number(jabatanRow.tunjangan ?? 0) : 0;

  // potongan tetap (asuransi kesehatan)
  const asuransiKesehatan = 450_000;

  const totalPendapatan = gajiPokok + tunjanganGaji;
  const totalPotongan = asuransiKesehatan;
  const gajiSetelahAsuransi = totalPendapatan - totalPotongan;

  // rekap presensi
  const totalHadir = attendanceSummary?.hadir ?? 0;
  const totalSakit = attendanceSummary?.sakit ?? 0;
  const totalCuti = attendanceSummary?.cuti ?? 0;
  const totalAbsen = attendanceSummary?.absen ?? 0;
  const totalHadirEfektif = attendanceSummary?.effective ?? 0; // hadir=1, sakit=0.8, cuti=0.4, absen=0

  const hariKerja = workingDays ?? 0;

  let faktorKehadiran = 1;
  let gajiBersihDiterima = gajiSetelahAsuransi;

  if (hariKerja > 0) {
    faktorKehadiran = totalHadirEfektif / hariKerja;

    if (!Number.isFinite(faktorKehadiran)) faktorKehadiran = 0;
    if (faktorKehadiran < 0) faktorKehadiran = 0;
    if (faktorKehadiran > 1) faktorKehadiran = 1;

    // gaji akhir = gaji setelah asuransi × (totalHadirEfektif / hariKerja)
    gajiBersihDiterima = gajiSetelahAsuransi * faktorKehadiran;
  }

  return {
    periode,
    nama: employee.name,
    nik: employee.nik,
    jabatan: employee.position,
    departemen: employee.department,

    gajiPokok,
    tunjanganGaji,
    asuransiKesehatan,
    totalPendapatan,
    totalPotongan,
    gajiSetelahAsuransi,

    totalHadir,
    totalSakit,
    totalCuti,
    totalAbsen,
    hariKerja,
    totalHadirEfektif,
    faktorKehadiran,

    gajiBersihDiterima,
  };
}

function formatRupiah(n) {
  return (
    "Rp " +
    Number(n).toLocaleString("id-ID", {
      minimumFractionDigits: 2,
    })
  );
}

// ======================
// State
// ======================

// auth
const isLoggedIn = ref(false);
const admin = ref(null);

// tab
const activeTab = ref("karyawan"); // "karyawan" | "laporan"
const activeReportTab = ref("jadwal"); // "jadwal" | "gaji"

// data karyawan
const employees = ref([]);
const form = reactive({
  nik: "",
  name: "",
  department: "",
  position: "",
  base_salary: "",
});

const editingEmployee = ref(null);
const editForm = reactive({
  id: "",
  nik: "",
  name: "",
  department: "",
  position: "",
  base_salary: "",
});

// data jabatan
const jabatanList = ref([]);
const jabatanForm = reactive({
  nama: "",
  tunjangan: "",
});
const editingJabatanNama = ref(null);

// presensi & gaji
const selectedEmployeeId = ref("");
const periodYear = ref(2025);
const periodMonth = ref(0); // 0=Jan
const attendanceByDate = ref({});
const attendanceDetail = reactive({
  open: false,
  date: null,
  status: "hadir",
});

// RINGKASAN PRESENSI DARI BACKEND (cmd_get_presensi_summary)
const presensiSummary = reactive({
  totalHadir: 0,
  totalSakit: 0,
  totalCuti: 0,
  totalAbsen: 0,
});

const dayLabels = ["Sen", "Sel", "Rab", "Kam", "Jum", "Sab", "Min"];

// ======================
// Computed
// ======================

const selectedEmployee = computed(
  () =>
    employees.value.find(
      (e) => String(e.id) === String(selectedEmployeeId.value),
    ) || null,
);

const periodeText = computed(() =>
  buildPeriodeText(periodMonth.value, periodYear.value),
);

// pakai angka dari backend
const attendanceSummary = computed(() => {
  const hadir = presensiSummary.totalHadir;
  const sakit = presensiSummary.totalSakit;
  const cuti = presensiSummary.totalCuti;
  const absen = presensiSummary.totalAbsen;

  const effective = hadir * 1 + sakit * 0.8 + cuti * 0.4 + absen * 0;

  return { hadir, sakit, cuti, absen, effective };
});

const workingDaysInMonth = computed(() =>
  countWorkingDaysInMonth(periodYear.value, periodMonth.value),
);

const selectedPayslip = computed(() => {
  if (!selectedEmployee.value || jabatanList.value.length === 0) return null;

  return hitungStrukGaji(
    selectedEmployee.value,
    jabatanList.value,
    periodeText.value,
    attendanceSummary.value,
    workingDaysInMonth.value,
  );
});

const canGoBack = computed(
  () => !(periodYear.value === MIN_YEAR && periodMonth.value === 0),
);
const canGoNext = computed(
  () => !(periodYear.value === MAX_YEAR && periodMonth.value === 11),
);

const calendarCells = computed(() => {
  const firstDay = new Date(
    periodYear.value,
    periodMonth.value,
    1,
  ).getDay();
  const daysInMonth = new Date(
    periodYear.value,
    periodMonth.value + 1,
    0,
  ).getDate();
  const startOffset = (firstDay + 6) % 7;

  const cells = [];
  for (let i = 0; i < startOffset; i += 1) {
    cells.push(null);
  }
  for (let day = 1; day <= daysInMonth; day += 1) {
    cells.push(day);
  }
  return cells;
});

// ======================
// Load data (invoke)
// ======================

async function loadEmployees() {
  try {
    const res = await invoke("cmd_list_employees");
    employees.value = Array.isArray(res) ? res : [];
  } catch (e) {
    console.error("Error load employees:", e);
  }
}

async function loadJabatan() {
  try {
    const res = await invoke("cmd_list_jabatan");
    jabatanList.value = Array.isArray(res) ? res : [];
  } catch (e) {
    console.error("Error load jabatan:", e);
  }
}

async function loadPresensiForCurrentMonth() {
  try {
    const employee_id = Number(selectedEmployeeId.value);

    console.log("[loadPresensi] param", {
      employee_id,
      year: periodYear.value,
      month: periodMonth.value + 1,
    });

    if (!employee_id) {
      attendanceByDate.value = {};
      return;
    }

    const res = await invoke("cmd_list_presensi", {
      employee_id,
      year: periodYear.value,
      month: periodMonth.value + 1, // 1–12
    });

    console.log("[loadPresensi] result from backend:", res);

    const map = {};

    (res || []).forEach((row) => {
      const rawTanggal =
        row.tanggal ??
        row.tgl ??
        row.date ??
        row.tanggal_presensi;

      if (!rawTanggal) {
        console.warn("[loadPresensi] row tanpa tanggal:", row);
        return;
      }

      const key =
        typeof rawTanggal === "string"
          ? rawTanggal.slice(0, 10)
          : String(rawTanggal);

      let rawStatus =
        row.status ??
        row.status_presensi ??
        row.presensi_status ??
        row.kehadiran;

      if (!rawStatus) {
        console.warn("[loadPresensi] row tanpa status:", row);
        return;
      }

      const status = String(rawStatus).toLowerCase().trim();

      map[key] = status;
    });

    console.log("[loadPresensi] attendanceByDate map:", map);
    attendanceByDate.value = map;
  } catch (e) {
    console.error("Error load presensi:", e);
    attendanceByDate.value = {};
  }
}

async function loadPresensiSummaryForCurrentMonth() {
  try {
    const employee_id = Number(selectedEmployeeId.value);
    if (!employee_id) {
      presensiSummary.totalHadir = 0;
      presensiSummary.totalSakit = 0;
      presensiSummary.totalCuti = 0;
      presensiSummary.totalAbsen = 0;
      return;
    }

    const res = await invoke("cmd_get_presensi_summary", {
      employee_id,
      year: periodYear.value,
      month: periodMonth.value + 1, // 1..12
    });

    presensiSummary.totalHadir = res.total_hadir ?? 0;
    presensiSummary.totalSakit = res.total_sakit ?? 0;
    presensiSummary.totalCuti = res.total_cuti ?? 0;
    presensiSummary.totalAbsen = res.total_absen ?? 0;

    console.log("[summary] ", { ...presensiSummary });
  } catch (e) {
    console.error("Error get presensi summary:", e);
    presensiSummary.totalHadir = 0;
    presensiSummary.totalSakit = 0;
    presensiSummary.totalCuti = 0;
    presensiSummary.totalAbsen = 0;
  }
}

// ======================
// Watchers
// ======================

watch(isLoggedIn, (val) => {
  if (val) {
    loadEmployees();
    loadJabatan();
  }
});

watch(
  [isLoggedIn, selectedEmployeeId, periodYear, periodMonth],
  ([loggedIn]) => {
    if (!loggedIn) return;

    if (!selectedEmployeeId.value) {
      attendanceByDate.value = {};
      presensiSummary.totalHadir = 0;
      presensiSummary.totalSakit = 0;
      presensiSummary.totalCuti = 0;
      presensiSummary.totalAbsen = 0;
      return;
    }

    loadPresensiForCurrentMonth();          // kalender
    loadPresensiSummaryForCurrentMonth();   // total hadir/sakit/cuti/absen
  },
);

// ======================
// Handlers
// ======================

function handleLoginSuccess(adminData) {
  admin.value = adminData;
  isLoggedIn.value = true;
}

// karyawan
async function handleAddEmployee() {
  try {
    await invoke("cmd_add_employee", {
      newEmp: {
        nik: form.nik,
        name: form.name,
        department: form.department,
        position: form.position,
        base_salary: parseInt(form.base_salary || "0", 10),
      },
    });

    form.nik = "";
    form.name = "";
    form.department = "";
    form.position = "";
    form.base_salary = "";

    await loadEmployees();
  } catch (err) {
    console.error("Error add employee:", err);
    window.alert("Gagal menambah karyawan");
  }
}

function openEditModal(emp) {
  editingEmployee.value = emp;
  editForm.id = emp.id;
  editForm.nik = emp.nik;
  editForm.name = emp.name;
  editForm.department = emp.department;
  editForm.position = emp.position;
  editForm.base_salary = String(emp.base_salary ?? "");
}

function closeEditModal() {
  editingEmployee.value = null;
}

async function handleUpdateEmployee() {
  try {
    await invoke("cmd_update_employee", {
      employee: {
        id: editForm.id,
        nik: editForm.nik,
        name: editForm.name,
        department: editForm.department,
        position: editForm.position,
        base_salary: Number(editForm.base_salary),
      },
    });

    await loadEmployees();
    closeEditModal();
    window.alert("Data karyawan berhasil diperbarui");
  } catch (err) {
    console.error("Error update employee:", err);
    window.alert("Gagal memperbarui data karyawan");
  }
}

async function handleDeleteEmployee(emp) {
  const ok = window.confirm(
    `Yakin ingin menghapus karyawan dengan NIK ${emp.nik}?`,
  );
  if (!ok) return;

  try {
    await invoke("cmd_delete_employee", { id: emp.id });
    await loadEmployees();
    window.alert("Data karyawan berhasil dihapus");
  } catch (err) {
    console.error("Error delete employee:", err);
    window.alert("Gagal menghapus data karyawan");
  }
}

// jabatan
function startEditJabatan(jabatanOrNull) {
  if (!jabatanOrNull) {
    editingJabatanNama.value = null;
    jabatanForm.nama = "";
    jabatanForm.tunjangan = "";
    return;
  }

  editingJabatanNama.value = jabatanOrNull.nama;
  jabatanForm.nama = jabatanOrNull.nama;
  jabatanForm.tunjangan = String(jabatanOrNull.tunjangan ?? "");
}

async function handleSubmitJabatan() {
  try {
    const payload = {
      nama: jabatanForm.nama,
      tunjangan: Number(jabatanForm.tunjangan),
    };

    if (editingJabatanNama.value) {
      await invoke("cmd_update_jabatan", {
        nama: editingJabatanNama.value,
        jabatan: payload,
      });
    } else {
      await invoke("cmd_add_jabatan", { jabatan: payload });
    }

    jabatanForm.nama = "";
    jabatanForm.tunjangan = "";
    editingJabatanNama.value = null;
    await loadJabatan();
  } catch (err) {
    console.error("Error simpan jabatan:", err);
    window.alert("Gagal menyimpan jabatan");
  }
}

async function handleDeleteJabatan(jabatan) {
  const ok = window.confirm(
    `Yakin menghapus jabatan "${jabatan.nama}"?`,
  );
  if (!ok) return;

  try {
    await invoke("cmd_delete_jabatan", { nama: jabatan.nama });
    await loadJabatan();
  } catch (err) {
    console.error("Error delete jabatan:", err);
    const msg = String(err || "");
    if (msg.includes("409")) {
      window.alert(
        `Tidak bisa menghapus jabatan "${jabatan.nama}" karena masih digunakan oleh data karyawan.\n` +
          `Ubah atau hapus dulu karyawan yang memakai jabatan ini.`,
      );
    } else {
      window.alert("Gagal menghapus jabatan");
    }
  }
}

// periode
function handlePrevMonth() {
  if (!canGoBack.value) return;

  if (periodMonth.value === 0) {
    periodMonth.value = 11;
    periodYear.value -= 1;
  } else {
    periodMonth.value -= 1;
  }
}

function handleNextMonth() {
  if (!canGoNext.value) return;

  if (periodMonth.value === 11) {
    periodMonth.value = 0;
    periodYear.value += 1;
  } else {
    periodMonth.value += 1;
  }
}

function handleChangeYear(year) {
  if (year < MIN_YEAR || year > MAX_YEAR) return;
  periodYear.value = year;
}

// presensi
function handleSelectDay(dateObj) {
  if (!selectedEmployeeId.value) return;

  const yyyy = dateObj.getFullYear();
  const mm = String(dateObj.getMonth() + 1).padStart(2, "0");
  const dd = String(dateObj.getDate()).padStart(2, "0");
  const key = `${yyyy}-${mm}-${dd}`;

  const existingStatus = attendanceByDate.value[key] || "hadir";

  attendanceDetail.open = true;
  attendanceDetail.date = key;
  attendanceDetail.status = existingStatus;
}

function getStatusForDay(day) {
  if (!day) return null;
  const yyyy = periodYear.value;
  const mm = String(periodMonth.value + 1).padStart(2, "0");
  const dd = String(day).padStart(2, "0");
  const key = `${yyyy}-${mm}-${dd}`;
  return attendanceByDate.value[key] || null;
}

function calendarCellClass(day) {
  if (!day) return "calendar__cell";

  const status = getStatusForDay(day);
  return [
    "calendar__cell",
    status ? `calendar__cell--status-${status}` : "",
    selectedEmployee.value ? "calendar__cell--clickable" : "",
  ]
    .filter(Boolean)
    .join(" ");
}

function handleClickDay(day) {
  if (!day || !selectedEmployee.value) return;
  const dateObj = new Date(periodYear.value, periodMonth.value, day);
  handleSelectDay(dateObj);
}

function handleCloseAttendanceModal() {
  attendanceDetail.open = false;
  attendanceDetail.date = null;
  attendanceDetail.status = "hadir";
}

async function handleSaveAttendance() {
  if (!selectedEmployeeId.value || !attendanceDetail.date) return;

  try {
    await invoke("cmd_upsert_presensi", {
      presensi: {
        employee_id: Number(selectedEmployeeId.value),
        tanggal: attendanceDetail.date,
        status: attendanceDetail.status,
      },
    });

    attendanceByDate.value = {
      ...attendanceByDate.value,
      [attendanceDetail.date]: attendanceDetail.status,
    };

    handleCloseAttendanceModal();
  } catch (e) {
    console.error("Error upsert presensi:", e);
    window.alert("Gagal menyimpan presensi");
  }
}
</script>

<template>
  <div class="app-root">
    <!-- Login -->
    <AdminLogin
      v-if="!isLoggedIn"
      @success="handleLoginSuccess"
    />

    <!-- App utama -->
    <div
      v-else
      class="app-container"
    >
      <header class="app-header">
        <h1 class="app-title">Manajemen Sistem Kepegawaian</h1>
        <p class="app-subtitle">
          Kelola data karyawan, presensi, dan gaji dalam satu aplikasi.
        </p>
        <p
          v-if="admin"
          class="app-subtitle"
        >
          Login sebagai: <strong>{{ admin.email }}</strong>
        </p>
      </header>

      <nav class="tab-nav">
        <button
          type="button"
          class="tab-nav__item"
          :class="{ 'is-active': activeTab === 'karyawan' }"
          @click="activeTab = 'karyawan'"
        >
          Data Karyawan
        </button>
        <button
          type="button"
          class="tab-nav__item"
          :class="{ 'is-active': activeTab === 'laporan' }"
          @click="activeTab = 'laporan'"
        >
          Laporan
        </button>
      </nav>

      <!-- TAB DATA KARYAWAN -->
      <main
        v-if="activeTab === 'karyawan'"
        class="content-layout"
      >
        <!-- Form karyawan -->
        <section class="card form-card">
          <h2 class="card-title">Data Karyawan</h2>
          <p class="card-description">
            Isi data karyawan lalu tekan tombol tambah untuk menyimpan.
          </p>

          <form
            class="employee-form"
            @submit.prevent="handleAddEmployee"
          >
            <div class="form-row">
              <label for="nik">NIK</label>
              <input
                id="nik"
                v-model="form.nik"
                required
                placeholder="Contoh: 11231234"
              />
            </div>

            <div class="form-row">
              <label for="name">Nama</label>
              <input
                id="name"
                v-model="form.name"
                required
                placeholder="Nama lengkap"
              />
            </div>

            <div class="form-row">
              <label for="department">Departemen</label>
              <input
                id="department"
                v-model="form.department"
                required
                placeholder="Contoh: Marketing"
              />
            </div>

            <div class="form-row">
              <label for="position">Jabatan</label>
              <input
                id="position"
                v-model="form.position"
                required
                placeholder="Contoh: Staff, Analis, Manager"
              />
            </div>

            <div class="form-row">
              <label for="base_salary">Gaji Pokok</label>
              <input
                id="base_salary"
                type="number"
                min="0"
                v-model="form.base_salary"
                required
                placeholder="Contoh: 3000000"
              />
            </div>

            <div class="form-actions">
              <button
                type="submit"
                class="btn-primary"
              >
                Tambah Karyawan
              </button>
            </div>
          </form>
        </section>

        <!-- Master Jabatan -->
        <section class="card table-card">
          <h2 class="card-title">Master Jabatan &amp; Tunjangan Gaji</h2>
          <p class="card-description">
            Kelola daftar jabatan dan besaran tunjangannya. Data ini dipakai
            untuk perhitungan struk gaji.
          </p>

          <form
            class="employee-form"
            @submit.prevent="handleSubmitJabatan"
          >
            <div class="form-row">
              <label for="jabatan-nama">Nama Jabatan</label>
              <input
                id="jabatan-nama"
                v-model="jabatanForm.nama"
                required
                placeholder="Contoh: Staff, Supervisor, Manager"
              />
            </div>

            <div class="form-row">
              <label for="jabatan-tunjangan">Tunjangan Gaji</label>
              <input
                id="jabatan-tunjangan"
                type="number"
                min="0"
                v-model="jabatanForm.tunjangan"
                required
                placeholder="Contoh: 300000"
              />
            </div>

            <div class="form-actions">
              <button
                type="submit"
                class="btn-primary"
              >
                {{
                  editingJabatanNama
                    ? "Simpan Perubahan"
                    : "Tambah Jabatan"
                }}
              </button>

              <button
                v-if="editingJabatanNama"
                type="button"
                class="btn-secondary-outline"
                @click="startEditJabatan(null)"
              >
                Batal
              </button>
            </div>
          </form>

          <div
            class="table-wrapper"
            style="margin-top: 16px"
          >
            <table class="employee-table">
              <thead>
                <tr>
                  <th>Nama Jabatan</th>
                  <th>Tunjangan</th>
                  <th>Aksi</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="j in jabatanList"
                  :key="j.nama"
                >
                  <td>{{ j.nama }}</td>
                  <td>{{ formatRupiah(j.tunjangan) }}</td>
                  <td class="action-cell action-cell--jabatan">
                    <button
                      type="button"
                      class="btn-secondary"
                      @click="startEditJabatan(j)"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      class="btn-danger"
                      @click="handleDeleteJabatan(j)"
                    >
                      Hapus
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Tabel karyawan -->
        <section class="card table-card">
          <div class="table-card-header">
            <h2 class="card-title">Daftar Karyawan</h2>
            <span class="badge-count">
              {{ employees.length }} karyawan
            </span>
          </div>

          <div class="table-wrapper">
            <table class="employee-table">
              <thead>
                <tr>
                  <th>NIK</th>
                  <th>Nama</th>
                  <th>Departemen</th>
                  <th>Jabatan</th>
                  <th>Gaji Pokok</th>
                  <th>Aksi</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="emp in employees"
                  :key="emp.id"
                >
                  <td>{{ emp.nik }}</td>
                  <td>{{ emp.name }}</td>
                  <td>{{ emp.department }}</td>
                  <td>{{ emp.position }}</td>
                  <td>{{ formatRupiah(emp.base_salary) }}</td>
                  <td class="action-cell action-cell--karyawan">
                    <button
                      type="button"
                      class="btn-secondary"
                      @click="openEditModal(emp)"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      class="btn-danger"
                      @click="handleDeleteEmployee(emp)"
                    >
                      Hapus
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </main>

      <!-- TAB LAPORAN -->
      <main
        v-else-if="activeTab === 'laporan'"
        class="content-layout"
      >
        <nav class="tab-nav tab-nav--inner">
          <button
            type="button"
            class="tab-nav__item"
            :class="{ 'is-active': activeReportTab === 'jadwal' }"
            @click="activeReportTab = 'jadwal'"
          >
            Data Presensi Karyawan
          </button>
          <button
            type="button"
            class="tab-nav__item"
            :class="{ 'is-active': activeReportTab === 'gaji' }"
            @click="activeReportTab = 'gaji'"
          >
            Data Gaji Karyawan
          </button>
        </nav>

        <!-- Sub-tab PRESENSI -->
        <section
          v-if="activeReportTab === 'jadwal'"
          class="card form-card"
        >
          <h2 class="card-title">Data Presensi Karyawan</h2>
          <p class="card-description">
            Pilih karyawan lalu lihat presensinya dalam bentuk kalender.
          </p>

          <div class="form-row">
            <label for="employee-schedule-select">Pilih Karyawan</label>
            <select
              id="employee-schedule-select"
              v-model="selectedEmployeeId"
            >
              <option value="">-- Pilih karyawan --</option>
              <option
                v-for="emp in employees"
                :key="emp.id"
                :value="String(emp.id)"
              >
                {{ emp.nik }} - {{ emp.name }}
              </option>
            </select>
          </div>

          <div class="calendar-presensi">
            <div class="calendar-presensi__header">
              <div class="calendar-presensi__periode">
                Periode:
                <strong>
                  {{ MONTH_NAMES_ID[periodMonth] }} {{ periodYear }}
                </strong>
              </div>

              <div class="calendar-presensi__toolbar">
                <button
                  type="button"
                  class="btn-secondary-outline btn-sm"
                  @click="handlePrevMonth"
                  :disabled="!canGoBack"
                >
                  Back
                </button>

                <button
                  type="button"
                  class="btn-secondary-outline btn-sm"
                  @click="handleNextMonth"
                  :disabled="!canGoNext"
                >
                  Next
                </button>

                <button
                  type="button"
                  class="btn-chip btn-sm"
                  :class="{ 'is-active': periodYear === 2025 }"
                  @click="handleChangeYear(2025)"
                >
                  2025
                </button>

                <button
                  type="button"
                  class="btn-chip btn-sm"
                  :class="{ 'is-active': periodYear === 2026 }"
                  @click="handleChangeYear(2026)"
                >
                  2026
                </button>
              </div>
            </div>

            <div class="calendar">
              <div class="calendar__grid">
                <div
                  v-for="label in dayLabels"
                  :key="label"
                  class="calendar__cell calendar__cell--header"
                >
                  {{ label }}
                </div>

                <div
                  v-for="(day, idx) in calendarCells"
                  :key="idx"
                  :class="calendarCellClass(day)"
                  @click="handleClickDay(day)"
                >
                  <template v-if="day">
                    <div class="calendar__day-number">
                      {{ day }}
                    </div>
                    <span
                      v-if="getStatusForDay(day)"
                      class="calendar__badge"
                      :class="'calendar__badge--' + getStatusForDay(day)"
                    >
                      {{ getStatusForDay(day) }}
                    </span>
                  </template>
                </div>
              </div>

              <p
                v-if="!selectedEmployee"
                class="calendar-presensi__hint"
              >
                Pilih karyawan terlebih dahulu untuk melihat / mengisi presensi.
              </p>
            </div>
          </div>
        </section>

        <!-- Sub-tab GAJI -->
        <section
          v-else
          class="card form-card"
        >
          <h2 class="card-title">Data Gaji Karyawan</h2>
          <p class="card-description">
            Pilih karyawan untuk melihat struk gaji. Tunjangan diambil dari
            master jabatan, dan gaji disesuaikan dengan kehadiran
            (hadir=1, sakit=0,8, cuti=0,4, absen=0).
          </p>

          <div class="form-row">
            <label for="employee-salary-select">Pilih Karyawan</label>
            <select
              id="employee-salary-select"
              v-model="selectedEmployeeId"
            >
              <option value="">-- Pilih karyawan --</option>
              <option
                v-for="emp in employees"
                :key="emp.id"
                :value="String(emp.id)"
              >
                {{ emp.nik }} - {{ emp.name }}
              </option>
            </select>
          </div>

          <div
            v-if="selectedPayslip"
            class="payslip"
          >
            <hr style="margin: 16px 0" />
            <h3
              class="card-title"
              style="font-size: 1rem"
            >
              Struk Gaji Karyawan
            </h3>

            <p>Periode: {{ selectedPayslip.periode }}</p>
            <p>Nama: {{ selectedPayslip.nama }}</p>
            <p>NIK: {{ selectedPayslip.nik }}</p>
            <p>Jabatan: {{ selectedPayslip.jabatan }}</p>
            <p>Departemen: {{ selectedPayslip.departemen }}</p>

            <div style="margin-top: 12px">
              <strong>Komponen Gaji</strong>
              <p>
                Gaji Pokok:
                {{ formatRupiah(selectedPayslip.gajiPokok) }}
              </p>
              <p>
                Tunjangan Gaji:
                {{ formatRupiah(selectedPayslip.tunjanganGaji) }}
              </p>
              <p>
                Total Pendapatan:
                {{ formatRupiah(selectedPayslip.totalPendapatan) }}
              </p>
            </div>

            <div style="margin-top: 12px">
              <strong>Potongan</strong>
              <p>
                Asuransi Kesehatan:
                {{ formatRupiah(selectedPayslip.asuransiKesehatan) }}
              </p>
              <p>
                Total Potongan:
                {{ formatRupiah(selectedPayslip.totalPotongan) }}
              </p>
              <p>
                Gaji setelah potongan asuransi:
                {{ formatRupiah(selectedPayslip.gajiSetelahAsuransi) }}
              </p>
            </div>

            <div style="margin-top: 12px">
              <strong>Rekap Presensi Bulan Ini</strong>
              <p>
                Total kehadiran:
                {{ selectedPayslip.totalHadir }}
              </p>
              <p>
                Total sakit:
                {{ selectedPayslip.totalSakit }}
              </p>
              <p>
                Total cuti:
                {{ selectedPayslip.totalCuti }}
              </p>
              <p>
                Total absen:
                {{ selectedPayslip.totalAbsen }}
              </p>
              <p>
                Total kehadiran efektif:
                {{ selectedPayslip.totalHadirEfektif.toFixed(1) }}
                dari {{ selectedPayslip.hariKerja }} hari kerja
              </p>
              <p>
                Faktor kehadiran:
                {{ (selectedPayslip.faktorKehadiran * 100).toFixed(2) }}%
              </p>
            </div>

            <div style="margin-top: 12px">
              <p>
                <strong>Gaji setelah potongan asuransi: </strong>
                {{ formatRupiah(selectedPayslip.gajiSetelahAsuransi) }}
              </p>
              <p>
                <strong>
                  Gaji Bersih Diterima (berdasarkan kehadiran):
                </strong>
                {{ " " + formatRupiah(selectedPayslip.gajiBersihDiterima) }}
              </p>
            </div>
          </div>

          <div class="periode-toolbar">
            <div class="periode-toolbar__info">
              Periode aktif:
              <strong>
                {{ MONTH_NAMES_ID[periodMonth] }} {{ periodYear }}
              </strong>
            </div>

            <div class="periode-toolbar__buttons">
              <button
                type="button"
                class="btn-secondary-outline btn-sm"
                @click="handlePrevMonth"
                :disabled="!canGoBack"
              >
                Back
              </button>

              <button
                type="button"
                class="btn-secondary-outline btn-sm"
                @click="handleNextMonth"
                :disabled="!canGoNext"
              >
                Next
              </button>

              <button
                type="button"
                class="btn-chip btn-sm"
                :class="{ 'is-active': periodYear === 2025 }"
                @click="handleChangeYear(2025)"
              >
                2025
              </button>

              <button
                type="button"
                class="btn-chip btn-sm"
                :class="{ 'is-active': periodYear === 2026 }"
                @click="handleChangeYear(2026)"
              >
                2026
              </button>
            </div>
          </div>
        </section>
      </main>

      <!-- MODAL PRESENSI -->
      <div
        v-if="attendanceDetail.open"
        class="modal-backdrop"
      >
        <div class="modal">
          <h2 class="modal-title">Detail Presensi</h2>

          <div class="modal-body">
            <p>
              Tanggal:
              <strong>{{ attendanceDetail.date }}</strong>
            </p>

            <p v-if="selectedEmployee">
              Karyawan:
              <strong>
                {{ selectedEmployee.nik }} - {{ selectedEmployee.name }}
              </strong>
            </p>

            <div
              class="form-row"
              style="margin-top: 12px"
            >
              <label>Status Presensi</label>
              <div class="radio-group-vertical">
                <label>
                  <input
                    type="radio"
                    name="status-presensi"
                    value="hadir"
                    v-model="attendanceDetail.status"
                  />
                  Hadir
                </label>
                <label>
                  <input
                    type="radio"
                    name="status-presensi"
                    value="sakit"
                    v-model="attendanceDetail.status"
                  />
                  Sakit
                </label>
                <label>
                  <input
                    type="radio"
                    name="status-presensi"
                    value="cuti"
                    v-model="attendanceDetail.status"
                  />
                  Cuti
                </label>
                <label>
                  <input
                    type="radio"
                    name="status-presensi"
                    value="absen"
                    v-model="attendanceDetail.status"
                  />
                  Absen
                </label>
              </div>
            </div>
          </div>

          <div class="modal-actions">
            <button
              type="button"
              class="btn-secondary-outline"
              @click="handleCloseAttendanceModal"
            >
              Batal
            </button>
            <button
              type="button"
              class="btn-primary"
              @click="handleSaveAttendance"
            >
              Simpan Presensi
            </button>
          </div>
        </div>
      </div>

      <!-- MODAL EDIT KARYAWAN -->
      <div
        v-if="editingEmployee"
        class="modal-backdrop"
      >
        <div class="modal">
          <h2 class="modal-title">Edit Data Karyawan</h2>

          <form
            class="modal-form"
            @submit.prevent="handleUpdateEmployee"
          >
            <div class="form-row">
              <label for="edit-nik">NIK</label>
              <input
                id="edit-nik"
                type="text"
                v-model="editForm.nik"
                required
              />
            </div>

            <div class="form-row">
              <label for="edit-name">Nama</label>
              <input
                id="edit-name"
                type="text"
                v-model="editForm.name"
                required
              />
            </div>

            <div class="form-row">
              <label for="edit-dept">Departemen</label>
              <input
                id="edit-dept"
                type="text"
                v-model="editForm.department"
                required
              />
            </div>

            <div class="form-row">
              <label for="edit-position">Jabatan</label>
              <input
                id="edit-position"
                type="text"
                v-model="editForm.position"
                required
              />
            </div>

            <div class="form-row">
              <label for="edit-salary">Gaji Pokok</label>
              <input
                id="edit-salary"
                type="number"
                min="0"
                v-model="editForm.base_salary"
                required
              />
            </div>

            <div class="modal-actions">
              <button
                type="button"
                class="btn-secondary-outline"
                @click="closeEditModal"
              >
                Batal
              </button>
              <button
                type="submit"
                class="btn-primary"
              >
                Simpan Perubahan
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>
