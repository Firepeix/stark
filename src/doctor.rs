use reqwest::StatusCode;



#[derive(Debug)]
pub enum Health {
    Healthy,
    Dead
}

pub async fn check_health(endpoint: &str) -> Health {
    match probe(endpoint).await {
        StatusCode::OK => Health::Healthy,
        _ => Health::Dead
    }
}

async fn probe(endpoint: &str) -> StatusCode {
    reqwest::get(endpoint)
    .await.unwrap()
    .status()
}