use dotenvy::dotenv;
use reqwest::Client;
use std::env;

/// Config & helper Supabase
pub struct Supabase {
    pub url: String,
    pub anon_key: String,
}

impl Supabase {
    /// Baca SUPABASE_URL & SUPABASE_ANON_KEY dari environment / .env
    pub fn new() -> Self {
        dotenv().ok(); // load .env kalau ada

        let url = env::var("SUPABASE_URL").expect("SUPABASE_URL not set");
        let anon_key = env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY not set");

        Self { url, anon_key }
    }

    pub fn client(&self) -> Client {
        Client::new()
    }

    /// Buat URL REST Supabase, contoh:
    /// endpoint("employees?select=*")
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}/rest/v1/{}", self.url.trim_end_matches('/'), path)
    }
}
