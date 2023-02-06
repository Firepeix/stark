
use std::time::Duration;

use crate::{google::{self}, doctor::Health, check_health};

pub async fn observe() {
    let mut retries = 0;
    loop {
        println!("Checando saude de firelink");
        let manager = google::get_manager().await;
        match check_health(&format!("{}/health", &manager.get_patient())).await {
            Health::Healthy => tokio::time::sleep(Duration::from_secs(30)).await,
            Health::Dead => {
                println!("Firelink destruida");
                
                if retries >= 3 {
                    println!("Tentativas de ligar falharam");
                    break;
                }

                kill_previous_tunnel().await;
                retries += 1;
                tokio::time::sleep(Duration::from_secs(4)).await
            },
        }
    }
}

async fn kill_previous_tunnel() {
    dbg!(tunnel_is_running().await);
}

async fn tunnel_is_running() -> bool {
    let status = check_health("http://localhost:4040/api/tunnels").await;
    matches!(status, Health::Healthy)
}

