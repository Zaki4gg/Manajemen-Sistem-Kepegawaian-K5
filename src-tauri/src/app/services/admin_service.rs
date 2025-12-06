use crate::app::domain::admin::Admin;
use crate::app::infra::supabase::Supabase;

pub async fn login_admin(email: String, password: String) -> Result<Admin, String> {
    let sb = Supabase::new();

    // SELECT email FROM admin WHERE email = ... AND password = ... LIMIT 1
    let query = format!(
        "admin?select=email&email=eq.{email}&password=eq.{password}&limit=1"
    );

    // helper Supabase: handle request + status + parsing JSON
    let admins: Vec<Admin> = sb
        .get_json(&query, "Supabase login error")
        .await?;

    admins
        .into_iter()
        .next()
        .ok_or_else(|| "Email atau password salah".to_string())
}
