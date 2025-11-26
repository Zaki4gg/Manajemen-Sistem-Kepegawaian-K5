use crate::app::domain::admin::Admin;
use crate::app::infra::supabase::Supabase;

pub async fn login_admin(email: String, password: String) -> Result<Admin, String> {
    let sb = Supabase::new();

    // SELECT email FROM admin WHERE email = ... AND password = ... LIMIT 1
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

    // Supabase REST akan mengembalikan array JSON
    let admins: Vec<Admin> = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    admins
        .into_iter()
        .next()
        .ok_or_else(|| "Email atau password salah".to_string())
}
