use serde::de::DeserializeOwned;

pub async fn get_json<T: DeserializeOwned>(
    api_url: &str,
    path: &str,
    api_key: &str,
) -> Result<T, String> {
    let response = reqwest::Client::new()
        .get(format!("{}{}", api_url, path))
        .header("X-Admin-Key", api_key)
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if response.status().is_success() {
        response
            .json::<T>()
            .await
            .map_err(|e| format!("Error procesando JSON: {}", e))
    } else {
        Err(format!("Error del servidor: {}", response.status()))
    }
}
