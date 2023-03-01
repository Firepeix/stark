
mod nurse;
mod doctor;
mod surgeon;

use std::time::Duration;

use tokio::sync::broadcast::{Sender, Receiver};


use crate::{google::Manager, hospital::doctor::{check_health, Health}, controller::CommandMessage};

use self::surgeon::ressurect;


pub(crate) async fn enter(pacient: Manager, dispatcher: Sender<CommandMessage>, listener: Receiver<CommandMessage>) {
    ressurect(&pacient, dispatcher.clone(), listener).await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;
    
    observe(pacient, dispatcher).await;
}


async fn observe(manager: Manager, dispatcher: Sender<CommandMessage>) {
    let mut retries = 0;
    loop {
        println!("Checando saude de firelink");
        match check_health(&format!("{}/health", &manager.get_patient())).await {
            Health::Healthy => tokio::time::sleep(Duration::from_secs(30)).await,
            Health::Dead => {
                println!("Firelink destruida");
                
                if retries >= 30 {
                    println!("Tentativas de ligar falharam");
                    break;
                }

                kill_previous_tunnel(&dispatcher).await;

                start_new_tunnel(&manager, dispatcher.clone(), dispatcher.subscribe()).await;

                retries += 1;

                // Estacionando para dar tempo do servidor subir
                tokio::time::sleep(Duration::from_secs(5)).await
            },
        }
    }

    kill_previous_tunnel(&dispatcher).await;
}

async fn kill_previous_tunnel(dispatcher: &Sender<CommandMessage>) {
    if tunnel_is_running().await {
        println!("Ngrok existente - Matando anterior");
        if dispatcher.send(CommandMessage::StopNgrok).is_err() {
            println!("Ngrok ja desligado")
        }
        // Aguardando o tunnel morrer
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn start_new_tunnel(pacient: &Manager, dispatcher: Sender<CommandMessage>, listener: Receiver<CommandMessage>) {
    if tunnel_is_running().await {
        println!("Ngrok existente - Não iniciando");
        return;
    }

    match ressurect(pacient, dispatcher.clone(), listener).await {
        Ok(_) => {},
        Err(_) => kill_previous_tunnel(&dispatcher).await,
    }
}



async fn tunnel_is_running() -> bool {
    let status = check_health("http://localhost:4040/api/tunnels").await;
    matches!(status, Health::Healthy)
}

