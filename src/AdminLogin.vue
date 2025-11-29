<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits(["success"]);

const email = ref("");
const password = ref("");
const loading = ref(false);
const errorMsg = ref("");

async function handleSubmit() {
  errorMsg.value = "";
  loading.value = true;

  try {
    const admin = await invoke("cmd_admin_login", {
      email: email.value,
      password: password.value,
    });

    // kalau berhasil, kirim ke parent
    emit("success", admin);
  } catch (err) {
    console.error(err);
    errorMsg.value =
      typeof err === "string"
        ? err
        : "Login gagal. Periksa kembali email dan password.";
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="app-root">
    <div class="app-container">
      <header class="app-header">
        <h1 class="app-title">Login Admin</h1>
        <p class="app-subtitle">
          Masuk untuk mengelola data karyawan.
        </p>
      </header>

      <main class="content-layout">
        <section class="card form-card">
          <h2 class="card-title">Masuk sebagai admin</h2>
          <p class="card-description">
            Gunakan akun admin yang sudah terdaftar di tabel
            <code>admin</code>.
          </p>

          <form class="employee-form" @submit.prevent="handleSubmit">
            <div class="form-row">
              <label for="email">Email</label>
              <input
                id="email"
                type="email"
                v-model="email"
                placeholder="admin@example.com"
                required
              />
            </div>

            <div class="form-row">
              <label for="password">Password</label>
              <input
                id="password"
                type="password"
                v-model="password"
                placeholder="••••••••"
                required
              />
            </div>

            <p
              v-if="errorMsg"
              style="color: #b91c1c; margin-top: 4px"
            >
              {{ errorMsg }}
            </p>

            <div class="form-actions">
              <button
                type="submit"
                class="btn-primary"
                :disabled="loading"
              >
                {{ loading ? "Memproses..." : "Login" }}
              </button>
            </div>
          </form>
        </section>
      </main>
    </div>
  </div>
</template>
