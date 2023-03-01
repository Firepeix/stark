use std::{process::{Command, Stdio}, time::Duration};

use color_eyre::{Result, Report};
use lazy_static::lazy_static;
use serde::Deserialize;
use tokio::{sync::{broadcast::{Receiver, Sender}}};

use crate::{controller::CommandMessage, google::{Manager, self}};

use super::doctor::{Health, check_health};

lazy_static! {
    static ref NGROK_PATH: String = std::env::var("NGROK_PATH").unwrap();
    static ref FIRST_ARGUMENT: String = std::env::args().nth(1).unwrap();
    static ref SECOND_ARGUMENT: String =  std::env::args().nth(2).unwrap();
}


#[derive(Deserialize, Clone)]
struct Tunnels {
    tunnels: Vec<Tunnel>
}

#[derive(Deserialize, Clone)]
struct Tunnel {
    public_url: String
}

pub(crate) async fn ressurect(pacient: &Manager, dispatcher: Sender<CommandMessage>, listener: Receiver<CommandMessage>) -> Result<()> {

    tokio::spawn(async move { start_process(listener).await });
    
    // Espera iniciar o Ngrok
    tokio::time::sleep(Duration::from_secs(3)).await;

    let heart = apply_shock(dispatcher).await;
    

    patch(heart, pacient).await
}

async fn patch(heart: String, pacient: &Manager) -> Result<()> {
    let skeleton = insert_heart(heart, pacient.get_skeleton());
    google::update_manager(skeleton).await
}

fn insert_heart(heart: String, skeleton: String) -> String {
    let bits = skeleton.split('"').collect::<Vec<&str>>();
    let size = bits.len();
    let mut body = vec![String::new(); size];
    {
        let mut control = None;
        bits.into_iter().enumerate().for_each(|(index, bit)| {
            if bit.contains("ENDPOINT") {
                let path = bit.replace("ENDPOINT", "");
                control = Some(index);
                body[index + 6] = if path != "FIRELINK" { format!("{}/{}", heart.clone(), path.to_lowercase()) } else { heart.clone() };
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
    let mut child = Command::new(NGROK_PATH.as_str())
        .arg(FIRST_ARGUMENT.as_str())
        .arg(SECOND_ARGUMENT.as_str())
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
    while retry < 10 {
        let health = check_health(&endpoint).await;
        println!("Verificando inicio do tunel");
        if let Health::Healthy = health {
            return Ok(tunnel.public_url)
        }

        println!("Tunel - Não iniciou corretamente");

        // Esperando o servidor subir
        tokio::time::sleep(Duration::from_secs(3)).await;
        retry += 1;
    }

    Err(color_eyre::eyre::eyre!("Tunnel não subiu em 10 tentativas"))

}

async fn get_tunnel() -> Result<Tunnel> {
    let endpoint = "http://localhost:4040";
    let response = reqwest::get(&format!("{endpoint}/api/tunnels"))
    .await?
    .json::<Tunnels>()
    .await?;

    if response.tunnels.is_empty() {
        return Err(color_eyre::eyre::eyre!("Não possui nem um tunnel ativo"))
    }

    Ok(response.tunnels.first().unwrap().clone())
}