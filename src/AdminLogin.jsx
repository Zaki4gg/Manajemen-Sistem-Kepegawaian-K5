import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function AdminLogin({ onSuccess }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState("");

  const handleSubmit = async (e) => {
    e.preventDefault();
    setErrorMsg("");
    setLoading(true);

    try {
      // Panggil command TAURI -> Rust -> Supabase (tabel admin)
      const admin = await invoke("cmd_admin_login", {
        email,
        password,
      });

      // Jika berhasil, admin = { email: "..." }
      onSuccess(admin);
    } catch (err) {
      console.error(err);
      setErrorMsg(
        typeof err === "string"
          ? err
          : "Login gagal. Periksa kembali email dan password."
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app-root">
      <div className="app-container">
        <header className="app-header">
          <h1 className="app-title">Login Admin</h1>
          <p className="app-subtitle">
            Masuk untuk mengelola data karyawan.
          </p>
        </header>

        <main className="content-layout">
          <section className="card form-card">
            <h2 className="card-title">Masuk sebagai admin</h2>
            <p className="card-description">
              Gunakan akun admin yang sudah terdaftar di tabel <code>admin</code>.
            </p>

            <form onSubmit={handleSubmit} className="employee-form">
              <div className="form-row">
                <label htmlFor="email">Email</label>
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="admin@example.com"
                  required
                />
              </div>

              <div className="form-row">
                <label htmlFor="password">Password</label>
                <input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  required
                />
              </div>

              {errorMsg && (
                <p style={{ color: "#b91c1c", marginTop: "4px" }}>{errorMsg}</p>
              )}

              <div className="form-actions">
                <button
                  type="submit"
                  className="btn-primary"
                  disabled={loading}
                >
                  {loading ? "Memproses..." : "Login"}
                </button>
              </div>
            </form>
          </section>
        </main>
      </div>
    </div>
  );
}

export default AdminLogin;
