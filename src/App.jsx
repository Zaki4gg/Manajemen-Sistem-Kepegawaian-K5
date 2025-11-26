import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import "./App.css";
import AdminLogin from "./AdminLogin";

// Nama bulan untuk periode
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

function buildPeriodeText(monthIndex, year) {
  const monthName = MONTH_NAMES_ID[monthIndex] ?? "";
  const firstDay = 1;
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();

  const firstStr = String(firstDay).padStart(2, "0");
  const lastStr = String(lastDay).padStart(2, "0");

  return `${firstStr}–${lastStr} ${monthName} ${year}`;
}

// Hitung struk gaji berdasarkan data employees + jabatan dari database
function hitungStrukGaji(employee, jabatanList, periode) {
  const gajiPokok = Number(employee.base_salary ?? 0);

  const jabatanRow =
    jabatanList?.find((j) => j.nama === employee.position) || null;

  const tunjanganGaji = jabatanRow ? Number(jabatanRow.tunjangan ?? 0) : 0;

  const asuransiKesehatan = 450_000;

  const totalPendapatan = gajiPokok + tunjanganGaji;
  const totalPotongan = asuransiKesehatan;
  const gajiBersih = totalPendapatan - totalPotongan;

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
    gajiBersih,
  };
}

function formatRupiah(n) {
  return (
    "Rp " + Number(n).toLocaleString("id-ID", { minimumFractionDigits: 2 })
  );
}

function CalendarPresensi({
  year,
  month,
  selectedEmployee,
  presensiByDate = {},
  onSelectDay,
}) {
  const dayLabels = ["Sen", "Sel", "Rab", "Kam", "Jum", "Sab", "Min"];

  const firstDay = new Date(year, month, 1).getDay();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const startOffset = (firstDay + 6) % 7;

  const cells = [];
  for (let i = 0; i < startOffset; i += 1) {
    cells.push(null);
  }
  for (let day = 1; day <= daysInMonth; day += 1) {
    cells.push(day);
  }

  return (
    <div className="calendar">
      <div className="calendar__grid">
        {dayLabels.map((label) => (
          <div key={label} className="calendar__cell calendar__cell--header">
            {label}
          </div>
        ))}

        {cells.map((day, idx) => {
          if (!day) {
            return <div key={idx} className="calendar__cell" />;
          }

          const yyyy = year;
          const mm = String(month + 1).padStart(2, "0");
          const dd = String(day).padStart(2, "0");
          const key = `${yyyy}-${mm}-${dd}`;
          const status = presensiByDate[key];

          const cellClass = [
            "calendar__cell",
            status ? `calendar__cell--status-${status}` : "",
            selectedEmployee ? "calendar__cell--clickable" : "",
          ]
            .filter(Boolean)
            .join(" ");

          const handleClick = () => {
            if (!selectedEmployee || !onSelectDay) return;
            const dateObj = new Date(year, month, day);
            onSelectDay(dateObj);
          };

          return (
            <div key={idx} className={cellClass} onClick={handleClick}>
              <div className="calendar__day-number">{day}</div>
              {status && (
                <span className={`calendar__badge calendar__badge--${status}`}>
                  {status.charAt(0).toUpperCase() + status.slice(1)}
                </span>
              )}
            </div>
          );
        })}
      </div>

      {!selectedEmployee && (
        <p className="calendar-presensi__hint">
          Pilih karyawan terlebih dahulu untuk melihat / mengisi presensi.
        </p>
      )}
    </div>
  );
}

/* ---------- SMALL REUSABLE COMPONENTS ---------- */

function EmployeeSelect({ id, label, employees, value, onChange }) {
  return (
    <div className="form-row">
      <label htmlFor={id}>{label}</label>
      <select
        id={id}
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="">-- Pilih karyawan --</option>
        {employees.map((emp) => (
          <option key={emp.id} value={String(emp.id)}>
            {emp.nik} - {emp.name}
          </option>
        ))}
      </select>
    </div>
  );
}

/* ---------- SECTION COMPONENTS ---------- */

function EmployeeFormSection({ form, onChange, onSubmit }) {
  return (
    <section className="card form-card">
      <h2 className="card-title">Data Karyawan</h2>
      <p className="card-description">
        Isi data karyawan lalu tekan tombol tambah untuk menyimpan.
      </p>

      <form onSubmit={onSubmit} className="employee-form">
        <div className="form-row">
          <label htmlFor="nik">NIK</label>
          <input
            id="nik"
            name="nik"
            value={form.nik}
            onChange={onChange}
            required
            placeholder="Contoh: 11231067"
          />
        </div>

        <div className="form-row">
          <label htmlFor="name">Nama</label>
          <input
            id="name"
            name="name"
            value={form.name}
            onChange={onChange}
            required
            placeholder="Nama lengkap"
          />
        </div>

        <div className="form-row">
          <label htmlFor="department">Departemen</label>
          <input
            id="department"
            name="department"
            value={form.department}
            onChange={onChange}
            required
            placeholder="Contoh: Marketing"
          />
        </div>

        <div className="form-row">
          <label htmlFor="position">Jabatan</label>
          <input
            id="position"
            name="position"
            value={form.position}
            onChange={onChange}
            required
            placeholder="Contoh: Staff, Analis, Manager"
          />
        </div>

        <div className="form-row">
          <label htmlFor="base_salary">Gaji Pokok</label>
          <input
            id="base_salary"
            name="base_salary"
            type="number"
            min="0"
            value={form.base_salary}
            onChange={onChange}
            required
            placeholder="Contoh: 3000000"
          />
        </div>

        <div className="form-actions">
          <button type="submit" className="btn-primary">
            Tambah Karyawan
          </button>
        </div>
      </form>
    </section>
  );
}

function JabatanSection({
  jabatanForm,
  editingJabatanNama,
  onChange,
  onSubmit,
  jabatanList,
  onEdit,
  onDelete,
}) {
  return (
    <section className="card table-card">
      <h2 className="card-title">Master Jabatan &amp; Tunjangan Gaji</h2>
      <p className="card-description">
        Kelola daftar jabatan dan besaran tunjangannya. Data ini dipakai untuk
        perhitungan struk gaji.
      </p>

      <form onSubmit={onSubmit} className="employee-form">
        <div className="form-row">
          <label htmlFor="jabatan-nama">Nama Jabatan</label>
          <input
            id="jabatan-nama"
            name="nama"
            value={jabatanForm.nama}
            onChange={onChange}
            required
            placeholder="Contoh: Staff, Analis, Manager"
          />
        </div>

        <div className="form-row">
          <label htmlFor="jabatan-tunjangan">Tunjangan Gaji</label>
          <input
            id="jabatan-tunjangan"
            name="tunjangan"
            type="number"
            min="0"
            value={jabatanForm.tunjangan}
            onChange={onChange}
            required
            placeholder="Contoh: 300000"
          />
        </div>

        <div className="form-actions">
          <button type="submit" className="btn-primary">
            {editingJabatanNama ? "Simpan Perubahan" : "Tambah Jabatan"}
          </button>
          {editingJabatanNama && (
            <button
              type="button"
              className="btn-secondary-outline"
              onClick={onEdit.bind(null, null)}
            >
              Batal
            </button>
          )}
        </div>
      </form>

      <div className="table-wrapper" style={{ marginTop: "16px" }}>
        <table className="employee-table">
          <thead>
            <tr>
              <th>Nama Jabatan</th>
              <th>Tunjangan</th>
              <th>Aksi</th>
            </tr>
          </thead>
          <tbody>
            {jabatanList.map((j) => (
              <tr key={j.nama}>
                <td>{j.nama}</td>
                <td>{formatRupiah(j.tunjangan)}</td>
                <td className="action-cell action-cell--jabatan">
                  <button
                    type="button"
                    className="btn-secondary"
                    onClick={() => onEdit(j)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className="btn-danger"
                    onClick={() => onDelete(j)}
                  >
                    Hapus
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function EmployeeTableSection({ employees, onEdit, onDelete }) {
  return (
    <section className="card table-card">
      <div className="table-card-header">
        <h2 className="card-title">Daftar Karyawan</h2>
        <span className="badge-count">{employees.length} karyawan</span>
      </div>

      <div className="table-wrapper">
        <table className="employee-table">
          <thead>
            <tr>
              <th>NIK</th>
              <th>Nama</th>
              <th>Departemen</th>
              <th>Jabatan</th>
              <th>Gaji Pokok</th>
              <th className="aksi-header--karyawan">Aksi</th>
            </tr>
          </thead>
          <tbody>
            {employees.map((emp) => (
              <tr key={emp.id}>
                <td>{emp.nik}</td>
                <td>{emp.name}</td>
                <td>{emp.department}</td>
                <td>{emp.position}</td>
                <td>{formatRupiah(emp.base_salary)}</td>
                <td className="action-cell action-cell--karyawan">
                  <button
                    type="button"
                    className="btn-secondary"
                    onClick={() => onEdit(emp)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className="btn-danger"
                    onClick={() => onDelete(emp)}
                  >
                    Hapus
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function PresensiSection({
  employees,
  selectedemployee_id,
  onChangeEmployee,
  periodYear,
  periodMonth,
  canGoBack,
  canGoNext,
  onPrevMonth,
  onNextMonth,
  onChangeYear,
  selectedEmployee,
  attendanceByDate,
  onSelectDay,
}) {
  return (
    <section className="card form-card">
      <h2 className="card-title">Data Presensi Karyawan</h2>
      <p className="card-description">
        Pilih karyawan lalu lihat presensinya dalam bentuk kalender.
      </p>

      <EmployeeSelect
        id="employee-schedule-select"
        label="Pilih Karyawan"
        employees={employees}
        value={selectedemployee_id}
        onChange={onChangeEmployee}
      />

      <div className="calendar-presensi">
        <div className="calendar-presensi__header">
          <div className="calendar-presensi__periode">
            Periode:{" "}
            <strong>
              {MONTH_NAMES_ID[periodMonth]} {periodYear}
            </strong>
          </div>

          <div className="calendar-presensi__toolbar">
            <button
              type="button"
              className="btn-secondary-outline btn-sm"
              onClick={onPrevMonth}
              disabled={!canGoBack}
            >
              Back
            </button>

            <button
              type="button"
              className="btn-secondary-outline btn-sm"
              onClick={onNextMonth}
              disabled={!canGoNext}
            >
              Next
            </button>

            <button
              type="button"
              className={`btn-chip btn-sm ${
                periodYear === 2025 ? "is-active" : ""
              }`}
              onClick={() => onChangeYear(2025)}
            >
              2025
            </button>

            <button
              type="button"
              className={`btn-chip btn-sm ${
                periodYear === 2026 ? "is-active" : ""
              }`}
              onClick={() => onChangeYear(2026)}
            >
              2026
            </button>
          </div>
        </div>

        <CalendarPresensi
          year={periodYear}
          month={periodMonth}
          selectedEmployee={selectedEmployee}
          presensiByDate={attendanceByDate}
          onSelectDay={onSelectDay}
        />
      </div>
    </section>
  );
}

function GajiSection({
  employees,
  selectedemployee_id,
  onChangeEmployee,
  selectedPayslip,
  periodYear,
  periodMonth,
  canGoBack,
  canGoNext,
  onPrevMonth,
  onNextMonth,
  onChangeYear,
}) {
  return (
    <section className="card form-card">
      <h2 className="card-title">Data Gaji Karyawan</h2>
      <p className="card-description">
        Pilih karyawan untuk melihat struk gaji. Tunjangan diambil dari master
        jabatan.
      </p>

      <EmployeeSelect
        id="employee-salary-select"
        label="Pilih Karyawan"
        employees={employees}
        value={selectedemployee_id}
        onChange={onChangeEmployee}
      />

      {selectedPayslip && (
        <div className="payslip">
          <hr style={{ margin: "16px 0" }} />
          <h3 className="card-title" style={{ fontSize: "1rem" }}>
            Struk Gaji Karyawan
          </h3>

          <p>Periode: {selectedPayslip.periode}</p>
          <p>Nama: {selectedPayslip.nama}</p>
          <p>NIK: {selectedPayslip.nik}</p>
          <p>Jabatan: {selectedPayslip.jabatan}</p>
          <p>Departemen: {selectedPayslip.departemen}</p>

          <div style={{ marginTop: "12px" }}>
            <strong>Komponen Gaji</strong>
            <p>Gaji Pokok: {formatRupiah(selectedPayslip.gajiPokok)}</p>
            <p>
              Tunjangan Gaji: {formatRupiah(selectedPayslip.tunjanganGaji)}
            </p>
            <p>
              Total Pendapatan: {formatRupiah(selectedPayslip.totalPendapatan)}
            </p>
          </div>

          <div style={{ marginTop: "12px" }}>
            <strong>Potongan</strong>
            <p>
              Asuransi Kesehatan:{" "}
              {formatRupiah(selectedPayslip.asuransiKesehatan)}
            </p>
            <p>
              Total Potongan: {formatRupiah(selectedPayslip.totalPotongan)}
            </p>
          </div>

          <div style={{ marginTop: "12px" }}>
            <strong>Gaji Bersih Diterima: </strong>
            {formatRupiah(selectedPayslip.gajiBersih)}
          </div>
        </div>
      )}

      <div className="periode-toolbar">
        <div className="periode-toolbar__info">
          Periode aktif:{" "}
          <strong>
            {MONTH_NAMES_ID[periodMonth]} {periodYear}
          </strong>
        </div>

        <div className="periode-toolbar__buttons">
          <button
            type="button"
            className="btn-secondary-outline btn-sm"
            onClick={onPrevMonth}
            disabled={!canGoBack}
          >
            Back
          </button>

          <button
            type="button"
            className={`btn-chip btn-sm ${
              periodYear === 2025 ? "is-active" : ""
            }`}
            onClick={() => onChangeYear(2025)}
          >
            2025
          </button>

          <button
            type="button"
            className={`btn-chip btn-sm ${
              periodYear === 2026 ? "is-active" : ""
            }`}
            onClick={() => onChangeYear(2026)}
          >
            2026
          </button>

          <button
            type="button"
            className="btn-secondary-outline btn-sm"
            onClick={onNextMonth}
            disabled={!canGoNext}
          >
            Next
          </button>
        </div>
      </div>
    </section>
  );
}

function AttendanceModal({
  detail,
  selectedEmployee,
  onChangeStatus,
  onClose,
  onSave,
}) {
  return (
    <div className="modal-backdrop">
      <div className="modal">
        <h2 className="modal-title">Detail Presensi</h2>

        <div className="modal-body">
          <p>
            Tanggal: <strong>{detail.date}</strong>
          </p>
          {selectedEmployee && (
            <p>
              Karyawan:{" "}
              <strong>
                {selectedEmployee.nik} - {selectedEmployee.name}
              </strong>
            </p>
          )}

          <div className="form-row" style={{ marginTop: "12px" }}>
            <label>Status Presensi</label>
            <div className="radio-group-vertical">
              <label>
                <input
                  type="radio"
                  name="status-presensi"
                  value="hadir"
                  checked={detail.status === "hadir"}
                  onChange={onChangeStatus}
                />
                Hadir
              </label>
              <label>
                <input
                  type="radio"
                  name="status-presensi"
                  value="sakit"
                  checked={detail.status === "sakit"}
                  onChange={onChangeStatus}
                />
                Sakit
              </label>
              <label>
                <input
                  type="radio"
                  name="status-presensi"
                  value="cuti"
                  checked={detail.status === "cuti"}
                  onChange={onChangeStatus}
                />
                Cuti
              </label>
              <label>
                <input
                  type="radio"
                  name="status-presensi"
                  value="absen"
                  checked={detail.status === "absen"}
                  onChange={onChangeStatus}
                />
                Absen
              </label>
            </div>
          </div>
        </div>

        <div className="modal-actions">
          <button
            type="button"
            className="btn-secondary-outline"
            onClick={onClose}
          >
            Batal
          </button>
          <button type="button" className="btn-primary" onClick={onSave}>
            Simpan Presensi
          </button>
        </div>
      </div>
    </div>
  );
}

function EditEmployeeModal({
  editForm,
  onChangeField,
  onClose,
  onSubmit,
}) {
  return (
    <div className="modal-backdrop">
      <div className="modal">
        <h2 className="modal-title">Edit Data Karyawan</h2>

        <form onSubmit={onSubmit} className="modal-form">
          <div className="form-row">
            <label htmlFor="edit-nik">NIK</label>
            <input
              id="edit-nik"
              type="text"
              value={editForm.nik}
              onChange={(e) => onChangeField("nik", e.target.value)}
              required
            />
          </div>

          <div className="form-row">
            <label htmlFor="edit-name">Nama Lengkap</label>
            <input
              id="edit-name"
              type="text"
              value={editForm.name}
              onChange={(e) => onChangeField("name", e.target.value)}
              required
            />
          </div>

          <div className="form-row">
            <label htmlFor="edit-dept">Departemen</label>
            <input
              id="edit-dept"
              type="text"
              value={editForm.department}
              onChange={(e) => onChangeField("department", e.target.value)}
              required
            />
          </div>

          <div className="form-row">
            <label htmlFor="edit-position">Jabatan</label>
            <input
              id="edit-position"
              type="text"
              value={editForm.position}
              onChange={(e) => onChangeField("position", e.target.value)}
              required
            />
          </div>

          <div className="form-row">
            <label htmlFor="edit-salary">Gaji Pokok</label>
            <input
              id="edit-salary"
              type="number"
              min="0"
              value={editForm.base_salary}
              onChange={(e) => onChangeField("base_salary", e.target.value)}
              required
            />
          </div>

          <div className="modal-actions">
            <button
              type="button"
              className="btn-secondary-outline"
              onClick={onClose}
            >
              Batal
            </button>
            <button type="submit" className="btn-primary">
              Simpan Perubahan
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

/* ---------- MAIN APP ---------- */

function App() {
  const [employees, setEmployees] = useState([]);
  const [form, setForm] = useState({
    nik: "",
    name: "",
    department: "",
    position: "",
    base_salary: "",
  });

  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [admin, setAdmin] = useState(null);

  const [activeTab, setActiveTab] = useState("karyawan");
  const [activeReportTab, setActiveReportTab] = useState("jadwal");

  const [selectedemployee_id, setSelectedemployee_id] = useState(null);

  const [periodYear, setPeriodYear] = useState(2025);
  const [periodMonth, setPeriodMonth] = useState(0);

  const [attendanceByDate, setAttendanceByDate] = useState({});
  const [attendanceDetail, setAttendanceDetail] = useState({
    open: false,
    date: null,
    status: "hadir",
  });

  const canGoBack = !(periodYear === MIN_YEAR && periodMonth === 0);
  const canGoNext = !(periodYear === MAX_YEAR && periodMonth === 11);

  const [jabatanList, setJabatanList] = useState([]);
  const [jabatanForm, setJabatanForm] = useState({
    nama: "",
    tunjangan: "",
  });
  const [editingJabatanNama, setEditingJabatanNama] = useState(null);

  const [editingEmployee, setEditingEmployee] = useState(null);
  const [editForm, setEditForm] = useState({
    id: "",
    nik: "",
    name: "",
    department: "",
    position: "",
    base_salary: "",
  });

  /* ---- NAVIGASI PERIODE ---- */
  const handlePrevMonth = () => {
    if (!canGoBack) return;

    if (periodMonth > 0) {
      setPeriodMonth(periodMonth - 1);
    } else if (periodYear > MIN_YEAR) {
      setPeriodYear(periodYear - 1);
      setPeriodMonth(11);
    }
  };

  const handleNextMonth = () => {
    if (!canGoNext) return;

    if (periodMonth < 11) {
      setPeriodMonth(periodMonth + 1);
    } else if (periodYear < MAX_YEAR) {
      setPeriodYear(periodYear + 1);
      setPeriodMonth(0);
    }
  };

  const handleChangeYear = (year) => {
    if (year < MIN_YEAR || year > MAX_YEAR) return;
    setPeriodYear(year);
  };

  /* ---- LOAD DATA ---- */
  const loadEmployees = async () => {
    try {
      const res = await invoke("cmd_list_employees");
      setEmployees(res);
    } catch (e) {
      console.error("Error load employees:", e);
    }
  };

  const loadJabatan = async () => {
    try {
      const res = await invoke("cmd_list_jabatan");
      setJabatanList(res);
    } catch (e) {
      console.error("Error load jabatan:", e);
    }
  };

  const loadPresensiForCurrentMonth = async () => {
  try {
    const employee_id = Number(selectedemployee_id);
    console.log("LOAD PRESENSI", { employee_id, year: periodYear, month: periodMonth + 1 });

    if (!employee_id) {
      setAttendanceByDate({});
      return;
    }

    const res = await invoke("cmd_list_presensi", {
      employee_id: employee_id,
      year: periodYear,
      month: periodMonth + 1,
    });

    console.log("HASIL PRESENSI DARI BACKEND", res);

    const map = {};
    res.forEach((row) => {
      const key = row.tanggal.slice(0, 10);
      map[key] = row.status;
    });
    setAttendanceByDate(map);
  } catch (e) {
    console.error("Error load presensi:", e);
  }
};

  useEffect(() => {
    if (!isLoggedIn) return;
    loadEmployees();
    loadJabatan();
  }, [isLoggedIn]);

  useEffect(() => {
    if (!isLoggedIn) return;

    if (!selectedemployee_id) {
      setAttendanceByDate({});
      return;
    }

    loadPresensiForCurrentMonth();
  }, [isLoggedIn, selectedemployee_id, periodYear, periodMonth]);

  /* ---- PRESENSI ---- */
  const handleSelectDay = (dateObj) => {
    if (!selectedemployee_id) return;

    const yyyy = dateObj.getFullYear();
    const mm = String(dateObj.getMonth() + 1).padStart(2, "0");
    const dd = String(dateObj.getDate()).padStart(2, "0");
    const key = `${yyyy}-${mm}-${dd}`;

    const existingStatus = attendanceByDate[key] || "hadir";

    setAttendanceDetail({
      open: true,
      date: key,
      status: existingStatus,
    });
  };

  const handleChangeAttendanceStatus = (e) => {
    setAttendanceDetail((prev) => ({
      ...prev,
      status: e.target.value,
    }));
  };

  const handleSaveAttendance = async () => {
    if (!selectedemployee_id || !attendanceDetail.date) return;

    try {
      await invoke("cmd_upsert_presensi", {
        presensi: {
          employee_id: Number(selectedemployee_id),
          tanggal: attendanceDetail.date,   // "YYYY-MM-DD"
          status: attendanceDetail.status,  // "hadir" | "sakit" | "cuti" | "absen"
        },
      });

      setAttendanceByDate((prev) => ({
        ...prev,
        [attendanceDetail.date]: attendanceDetail.status,
      }));

      setAttendanceDetail({
        open: false,
        date: null,
        status: "hadir",
      });
    } catch (e) {
      console.error("Error upsert presensi:", e);
      alert("Gagal menyimpan presensi");
    }
  };

  const handleCloseAttendanceModal = () => {
    setAttendanceDetail({
      open: false,
      date: null,
      status: "hadir",
    });
  };

  /* ---- FORM KARYAWAN ---- */
  const handleChange = (e) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
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

      setForm({
        nik: "",
        name: "",
        department: "",
        position: "",
        base_salary: "",
      });
      await loadEmployees();
    } catch (err) {
      console.error("Error add employee:", err);
      alert("Gagal menambah karyawan");
    }
  };

  const openEditModal = (emp) => {
    setEditingEmployee(emp);
    setEditForm({
      id: emp.id,
      nik: emp.nik,
      name: emp.name,
      department: emp.department,
      position: emp.position,
      base_salary: String(emp.base_salary),
    });
  };

  const closeEditModal = () => {
    setEditingEmployee(null);
  };

  const handleChangeEditField = (field, value) => {
    setEditForm((prev) => ({ ...prev, [field]: value }));
  };

  const handleUpdateEmployee = async (e) => {
    e.preventDefault();

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
      alert("Data karyawan berhasil diperbarui");
    } catch (err) {
      console.error("Error update employee:", err);
      alert("Gagal memperbarui data karyawan");
    }
  };

  const handleDeleteEmployee = async (emp) => {
    const ok = window.confirm(
      `Yakin ingin menghapus karyawan dengan NIK ${emp.nik}?`
    );
    if (!ok) return;

    try {
      await invoke("cmd_delete_employee", { id: emp.id });
      await loadEmployees();
      alert("Data karyawan berhasil dihapus");
    } catch (err) {
      console.error("Error delete employee:", err);
      alert("Gagal menghapus data karyawan");
    }
  };

  /* ---- MASTER JABATAN ---- */
  const handleChangeJabatan = (e) => {
    setJabatanForm({ ...jabatanForm, [e.target.name]: e.target.value });
  };

  const handleSubmitJabatan = async (e) => {
    e.preventDefault();

    try {
      const payload = {
        nama: jabatanForm.nama,
        tunjangan: Number(jabatanForm.tunjangan),
      };

      if (editingJabatanNama) {
        await invoke("cmd_update_jabatan", {
          nama: editingJabatanNama,
          jabatan: payload,
        });
      } else {
        await invoke("cmd_add_jabatan", { jabatan: payload });
      }

      setJabatanForm({ nama: "", tunjangan: "" });
      setEditingJabatanNama(null);
      await loadJabatan();
    } catch (err) {
      console.error("Error simpan jabatan:", err);
      alert("Gagal menyimpan data jabatan");
    }
  };

  const startEditJabatan = (jabatanOrNull) => {
    if (!jabatanOrNull) {
      setEditingJabatanNama(null);
      setJabatanForm({ nama: "", tunjangan: "" });
      return;
    }

    setEditingJabatanNama(jabatanOrNull.nama);
    setJabatanForm({
      nama: jabatanOrNull.nama,
      tunjangan: String(jabatanOrNull.tunjangan),
    });
  };

  const handleDeleteJabatan = async (jabatan) => {
    const ok = window.confirm(
      `Yakin menghapus jabatan "${jabatan.nama}"?`
    );
    if (!ok) return;

    try {
      await invoke("cmd_delete_jabatan", { nama: jabatan.nama });
      await loadJabatan();
    } catch (err) {
      console.error("Error delete jabatan:", err);
      const msg = String(err || "");
      if (msg.includes("409")) {
        alert(
          `Tidak bisa menghapus jabatan "${jabatan.nama}" karena masih digunakan oleh data karyawan. ` +
            `Ubah atau hapus dulu karyawan yang memakai jabatan ini.`
        );
      } else {
        alert("Gagal menghapus jabatan");
      }
    }
  };

  const selectedEmployee = employees.find(
    (e) => String(e.id) === String(selectedemployee_id)
  );

  const periodeText = buildPeriodeText(periodMonth, periodYear);

  const selectedPayslip =
    selectedEmployee && jabatanList.length > 0
      ? hitungStrukGaji(selectedEmployee, jabatanList, periodeText)
      : null;

  if (!isLoggedIn) {
    return (
      <AdminLogin
        onSuccess={(adminData) => {
          setAdmin(adminData);
          setIsLoggedIn(true);
        }}
      />
    );
  }

  return (
    <div className="app-root">
      <div className="app-container">
        <header className="app-header">
          <h1 className="app-title">Manajemen Sistem Kepegawaian</h1>
          <p className="app-subtitle">
            Kelola data karyawan dengan tampilan yang lebih nyaman.
          </p>
          {admin && (
            <p className="app-subtitle">
              Login sebagai: <strong>{admin.email}</strong>
            </p>
          )}
        </header>

        <nav className="tab-nav">
          <button
            type="button"
            className={`tab-nav__item ${
              activeTab === "karyawan" ? "is-active" : ""
            }`}
            onClick={() => setActiveTab("karyawan")}
          >
            Data Karyawan
          </button>
          <button
            type="button"
            className={`tab-nav__item ${
              activeTab === "laporan" ? "is-active" : ""
            }`}
            onClick={() => setActiveTab("laporan")}
          >
            Laporan (Gaji & Jadwal)
          </button>
        </nav>

        {activeTab === "karyawan" && (
          <main className="content-layout">
            <EmployeeFormSection
              form={form}
              onChange={handleChange}
              onSubmit={handleSubmit}
            />

            <JabatanSection
              jabatanForm={jabatanForm}
              editingJabatanNama={editingJabatanNama}
              onChange={handleChangeJabatan}
              onSubmit={handleSubmitJabatan}
              jabatanList={jabatanList}
              onEdit={startEditJabatan}
              onDelete={handleDeleteJabatan}
            />

            <EmployeeTableSection
              employees={employees}
              onEdit={openEditModal}
              onDelete={handleDeleteEmployee}
            />
          </main>
        )}

        {activeTab === "laporan" && (
          <main className="content-layout">
            <nav className="tab-nav tab-nav--inner">
              <button
                type="button"
                className={`tab-nav__item ${
                  activeReportTab === "jadwal" ? "is-active" : ""
                }`}
                onClick={() => setActiveReportTab("jadwal")}
              >
                Data Presensi Karyawan
              </button>
              <button
                type="button"
                className={`tab-nav__item ${
                  activeReportTab === "gaji" ? "is-active" : ""
                }`}
                onClick={() => setActiveReportTab("gaji")}
              >
                Data Gaji Karyawan
              </button>
            </nav>

            {activeReportTab === "jadwal" && (
              <PresensiSection
                employees={employees}
                selectedemployee_id={selectedemployee_id}
                onChangeEmployee={setSelectedemployee_id}
                periodYear={periodYear}
                periodMonth={periodMonth}
                canGoBack={canGoBack}
                canGoNext={canGoNext}
                onPrevMonth={handlePrevMonth}
                onNextMonth={handleNextMonth}
                onChangeYear={handleChangeYear}
                selectedEmployee={selectedEmployee}
                attendanceByDate={attendanceByDate}
                onSelectDay={handleSelectDay}
              />
            )}

            {activeReportTab === "gaji" && (
              <GajiSection
                employees={employees}
                selectedemployee_id={selectedemployee_id}
                onChangeEmployee={setSelectedemployee_id}
                selectedPayslip={selectedPayslip}
                periodYear={periodYear}
                periodMonth={periodMonth}
                canGoBack={canGoBack}
                canGoNext={canGoNext}
                onPrevMonth={handlePrevMonth}
                onNextMonth={handleNextMonth}
                onChangeYear={handleChangeYear}
              />
            )}
          </main>
        )}

        {attendanceDetail.open && (
          <AttendanceModal
            detail={attendanceDetail}
            selectedEmployee={selectedEmployee}
            onChangeStatus={handleChangeAttendanceStatus}
            onClose={handleCloseAttendanceModal}
            onSave={handleSaveAttendance}
          />
        )}

        {editingEmployee && (
          <EditEmployeeModal
            editForm={editForm}
            onChangeField={handleChangeEditField}
            onClose={closeEditModal}
            onSubmit={handleUpdateEmployee}
          />
        )}
      </div>
    </div>
  );
}

export default App;
