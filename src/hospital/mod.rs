
mod nurse;
mod doctor;
mod surgeon;

use std::time::Duration;

use tokio::sync::broadcast::{Sender, Receiver};


use crate::{google::{Manager, self}, hospital::doctor::{check_health, Health}, controller::CommandMessage};

use self::surgeon::ressurect;


pub(crate) async fn enter(pacient: Manager, dispatcher: Sender<CommandMessage>, listener: Receiver<CommandMessage>) {
    ressurect(&pacient, listener).await.unwrap();
    observe(dispatcher).await;
}


async fn observe(dispatcher: Sender<CommandMessage>) {
    let mut retries = 0;
    loop {
        let manager = google::get_manager().await;

        let endpoint = &format!("{}/health", &manager.get_patient());
        
        println!("Checando saude de firelink - {endpoint}");
        
        match check_health(endpoint).await {
            Health::Healthy => tokio::time::sleep(Duration::from_secs(30)).await,
            Health::Dead => {
                println!("Firelink destruida");
                
                if retries >= 30 {
                    println!("Tentativas de ligar falharam");
                    break;
                }

                kill_previous_tunnel(&dispatcher).await;

                start_new_tunnel(&manager, dispatcher.subscribe()).await;

                retries += 1;

                println!("Hospital - Aguardando Config Propagar");
                tokio::time::sleep(Duration::from_secs(5)).await;
                println!("Hospital - Tempo de espera realizado");

            },
        }
    }

    kill_previous_tunnel(&dispatcher).await;
}

async fn kill_previous_tunnel(dispatcher: &Sender<CommandMessage>) {
    if dispatcher.send(CommandMessage::StopNgrok).is_err() {
        println!("Ngrok ja desligado")
    }
}

async fn start_new_tunnel(pacient: &Manager, listener: Receiver<CommandMessage>) {
    ressurect(pacient, listener).await.unwrap();
}
