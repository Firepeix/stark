use std::{process::{Command, Stdio}, time::Duration};

use color_eyre::{Result, Report};
use serde::Deserialize;
use tokio::{sync::{broadcast::{Receiver, Sender}}};

use crate::{controller::CommandMessage, google::Manager};

use super::doctor::{Health, check_health};


#[derive(Deserialize, Clone)]
struct Tunnels {
    tunnels: Vec<Tunnel>
}

#[derive(Deserialize, Clone)]
struct Tunnel {
    public_url: String
}

pub(crate) async fn ressurect(pacient: &Manager, dispatcher: Sender<CommandMessage>, listener: Receiver<CommandMessage>) {

    tokio::spawn(async move { start_process(listener).await });
    
    let heart = apply_shock(dispatcher).await;

    patch(heart, pacient).await
}

async fn patch(heart: String, pacient: &Manager) {
    let skeleton = insert_heart(heart, pacient.get_skeleton());
    revive(skeleton).await
}

async fn revive(skeleton: String) {
    let endpoint = "http://localhost:3001/v1/projects/ebisu-mobile/remoteConfig";
    let client = reqwest::Client::new();
    client.put(endpoint)
      .body(skeleton)
      .send()
      .await
      .unwrap();
}

fn insert_heart(heart: String, skeleton: String) -> String {
    let bits = skeleton.split("\"").collect::<Vec<&str>>();
    let size = bits.len();
    let mut body = vec![String::new(); size];
    {
        let mut control = None;
        bits.into_iter().enumerate().for_each(|(index, bit)| {
            if bit.contains("ENDPOINT") {
                control = Some(index);
                body[index + 6] = heart.clone();
                body[index] = bit.to_string();
            }

            if control.is_none()  {
                body[index] = bit.to_string()
            }

            if let Some(previous_index) = control {
                if previous_index + 6 == index {
                    control = None;
                } else {
                    body[index] = bit.to_string()
                }
            }
        });
    }

    body.join("\"")
}

async fn apply_shock(dispatcher: Sender<CommandMessage>) -> String {
    match get_tunnel().await {
        Ok(tunnel) => match take_measure(tunnel).await {
            Ok(firelink_endpoint) => firelink_endpoint,
            Err(report) => handle_error(report, dispatcher).await,
        },
        Err(report) => handle_error(report, dispatcher).await,
    }
}

async fn handle_error(report: Report, dispatcher: Sender<CommandMessage>) -> ! {
    dispatcher.send(CommandMessage::StopNgrok).unwrap();
    // Dormindo para dar tempo da mensagem chegar nos listeners
    tokio::time::sleep(Duration::from_secs(1)).await;
    Err(report).unwrap()
}

async fn start_process(mut listener: Receiver<CommandMessage>) {
    println!("Iniciando Ngrok");
    let mut child = Command::new("./ngrok")
        .arg("http")
        .arg("80")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();


    loop {
        if let Ok(CommandMessage::StopNgrok) = listener.try_recv() {
            println!("Desligando Ngrok");
            child.kill().unwrap();
            break;
        }

        if let Ok(Some(_)) = child.try_wait() {
            println!("Ngrok foi desligado");
            break;
        }
        
        // Dormindo para não gastar ciclos no CPU
        tokio::time::sleep(Duration::from_secs(1)).await;
    }    
}

async fn take_measure(tunnel: Tunnel) -> Result<String> {
    let mut retry = 0;
    let endpoint = format!("{}/health", &tunnel.public_url);
    while retry < 3 {
        let health = check_health(&endpoint).await;
        if let Health::Healthy = health {
            return Ok(tunnel.public_url)
        }

        // Esperando o servidor subir
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