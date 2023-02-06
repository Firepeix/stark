use std::{process::{Command, Stdio, Child}, time::Duration};

use color_eyre::{Result, Report};
use serde::Deserialize;
use tokio::{task::JoinHandle, sync::{oneshot::{channel, self, Sender, Receiver, error::TryRecvError}}};

use crate::doctor::Health;

#[derive(Deserialize, Clone)]
struct Tunnels {
    tunnels: Vec<Tunnel>
}

#[derive(Deserialize, Clone)]
struct Tunnel {
    public_url: String
}

pub async fn ressurect() -> String {
    let (tx, rx) = oneshot::channel::<bool>();
    tokio::spawn(async move {start_process(rx).await});
    match get_tunnel().await {
        Ok(tunnel) => match take_measure(tunnel).await {
            Ok(firelink_endpoint) => firelink_endpoint,
            Err(report) => handle_error(report, tx).await,
        },
        Err(report) => handle_error(report, tx).await,
    }

}

async fn handle_error(report: Report, commander: Sender<bool>) -> ! {
    commander.send(true).unwrap();
    Err(report).unwrap()
}

async fn start_process(mut commander: Receiver<bool>) {
    let mut child = Command::new("./ngrok")
        .arg("http")
        .arg("80")
        .spawn()
        .unwrap();

    loop {
        if commander.try_recv().is_ok() {
            println!("Desligando Ngrok");
            child.kill().unwrap();
            break;
        }

        if let Ok(Some(_)) = child.try_wait() {
            println!("Ngrok foi desligado");
            break;
        }
    }    
}

async fn take_measure(tunnel: Tunnel) -> Result<String> {
    let mut retry = 0;
    while retry < 3 {
        let health = crate::check_health(&tunnel.public_url).await;
        if let Health::Healthy = health {
            return Ok(tunnel.public_url)
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
        retry += 1;
    }

    Err(color_eyre::eyre::eyre!("Tunnel não subiu em 3 tentativas"))

}

async fn get_tunnel() -> Result<Tunnel> {
    let endpoint = "http://localhost:3000";
    let response = reqwest::get(&format!("{endpoint}/api/tunnels"))
    .await?
    .json::<Tunnels>()
    .await?;

    if response.tunnels.is_empty() {
        return Err(color_eyre::eyre::eyre!("Não possui nem um tunnel ativo"))
    }

    Ok(response.tunnels.first().unwrap().clone())
}