import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import "./App.css";

function App() {
  const [employees, setEmployees] = useState([]);
  const [form, setForm] = useState({
    nik: "",
    name: "",
    department: "",
    position: "",
    base_salary: "",
  });

  const loadEmployees = async () => {
    try {
      const res = await invoke("cmd_list_employees");
      setEmployees(res);
    } catch (e) {
      console.error("Error load employees:", e);
    }
  };

  useEffect(() => {
    loadEmployees();
  }, []);

  const handleChange = (e) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await invoke("cmd_add_employee", {
        // HARUS sama dengan nama parameter di Rust (new_emp)
        newEmp: {
          nik: form.nik,
          name: form.name,
          department: form.department,
          position: form.position,
          // HARUS sama dengan field di struct NewEmployee (base_salary)
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

  return (
    <div style={{ padding: 24 }}>
      <h1>Manajemen Sistem Kepegawaian</h1>
      <h2>Data Karyawan</h2>

      <form onSubmit={handleSubmit} style={{ marginBottom: 24 }}>
        <div>
          <label>NIK: </label>
          <input name="nik" value={form.nik} onChange={handleChange} required />
        </div>
        <div>
          <label>Nama: </label>
          <input name="name" value={form.name} onChange={handleChange} required />
        </div>
        <div>
          <label>Departemen: </label>
          <input
            name="department"
            value={form.department}
            onChange={handleChange}
            required
          />
        </div>
        <div>
          <label>Jabatan: </label>
          <input
            name="position"
            value={form.position}   // tadinya form.value (salah)
            onChange={handleChange}
            required
          />
        </div>
        <div>
          <label>Gaji Pokok: </label>
          <input
            name="base_salary"
            type="number"
            value={form.base_salary}
            onChange={handleChange}
            required
          />
        </div>
        <button type="submit">Tambah Karyawan</button>
      </form>

      <table border="1" cellPadding="6">
        <thead>
          <tr>
            <th>NIK</th>
            <th>Nama</th>
            <th>Departemen</th>
            <th>Jabatan</th>
            <th>Gaji Pokok</th>
          </tr>
        </thead>
        <tbody>
          {employees.map((e) => (
            <tr key={e.id}>
              <td>{e.nik}</td>
              <td>{e.name}</td>
              <td>{e.department}</td>
              <td>{e.position}</td>
              <td>{e.base_salary}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default App;
