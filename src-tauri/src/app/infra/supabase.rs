use dotenvy::dotenv;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;

pub struct Supabase {
    base_url: String,
    pub anon_key: String,
    http: Client,
}

impl Supabase {
    pub fn new() -> Self {
        dotenv().ok();

        let project_url =
            env::var("SUPABASE_URL").expect("SUPABASE_URL not set");
        let anon_key =
            env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY not set");

        let rest_url = format!("{}/rest/v1", project_url.trim_end_matches('/'));

        let http = Client::new();

        Self {
            base_url: rest_url,
            anon_key,
            http,
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn send_request(
        &self,
        req: reqwest::RequestBuilder,
        err_prefix: &str,
    ) -> Result<String, String> {
        let res = req.send().await.map_err(|e| e.to_string())?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            Err(format!("{err_prefix} {status}: {text}"))
        } else {
            Ok(text)
        }
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        err_prefix: &str,
    ) -> Result<T, String> {
        let url = self.endpoint(path);

        let text = self
            .send_request(
                self.http
                    .get(url)
                    .header("apikey", &self.anon_key)
                    .header("Authorization", format!("Bearer {}", self.anon_key)),
                err_prefix,
            )
            .await?;

        serde_json::from_str::<T>(&text).map_err(|e| {
            format!("{err_prefix} parse error: {e} | body: {text}")
        })
    }

    pub async fn insert_json<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        err_prefix: &str,
    ) -> Result<(), String> {
        let url = self.endpoint(path);

        let _ = self
            .send_request(
                self.http
                    .post(url)
                    .header("apikey", &self.anon_key)
                    .header("Authorization", format!("Bearer {}", self.anon_key))
                    .json(body),
                err_prefix,
            )
            .await?;

        Ok(())
    }

    pub async fn patch_json<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        err_prefix: &str,
    ) -> Result<(), String> {
        let url = self.endpoint(path);

        let _ = self
            .send_request(
                self.http
                    .patch(url)
                    .header("apikey", &self.anon_key)
                    .header("Authorization", format!("Bearer {}", self.anon_key))
                    .json(body),
                err_prefix,
            )
            .await?;

        Ok(())
    }

    pub async fn upsert_json<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        err_prefix: &str,
    ) -> Result<(), String> {
        let url = self.endpoint(path);

        let _ = self
            .send_request(
                self.http
                    .post(url)
                    .header("apikey", &self.anon_key)
                    .header("Authorization", format!("Bearer {}", self.anon_key))
                    .header("Content-Type", "application/json")
                    .header("Prefer", "resolution=merge-duplicates")
                    .json(body),
                err_prefix,
            )
            .await?;

        Ok(())
    }

    pub async fn delete(
        &self,
        path: &str,
        err_prefix: &str,
    ) -> Result<(), String> {
        let url = self.endpoint(path);

        let _ = self
            .send_request(
                self.http
                    .delete(url)
                    .header("apikey", &self.anon_key)
                    .header("Authorization", format!("Bearer {}", self.anon_key)),
                err_prefix,
            )
            .await?;

        Ok(())
    }
}
