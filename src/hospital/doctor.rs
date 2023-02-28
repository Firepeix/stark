use color_eyre::Result;
use reqwest::StatusCode;



#[derive(Debug)]
pub enum Health {
    Healthy,
    Dead
}

pub async fn check_health(endpoint: &str) -> Health {
    match probe(endpoint).await {
        Ok(StatusCode::OK) => Health::Healthy,
        _ => Health::Dead
    }
}

async fn probe(endpoint: &str) -> Result<StatusCode> {
    Ok(reqwest::get(endpoint)
    .await?
    .status())
}