use std::time::Duration;

use color_eyre::Result;
use lazy_static::lazy_static;
use serde::{Deserialize};
use tokio::time::Sleep;

use super::authentication;

lazy_static! {
    static ref BASE_ENDPOINT: String = std::env::var("REMOTE_CONFIG_URL").unwrap();
}

#[derive(Debug)]
pub(crate) struct Manager {
    config: Config,
    raw: String
}

impl Manager {
    pub(crate) fn get_patient(&self) -> String {
        self.config.firelink_endpoint()
    }

    pub(crate) fn get_skeleton(&self) -> String {
        self.raw.replace('\n', "").replace('\t', "")
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    parameters: RemoteConfigParameters
}

impl Config {
    pub(crate) fn firelink_endpoint(&self) -> String {
        self.parameters.firelink_endpoint.default_value.value.clone()
    }
}


pub(crate) async fn get_manager() -> Manager {
    let mut tries = 0;
    let max = 3;

    while tries < max {
        let result = manager(tries > 0).await;

        if let Ok(manager) = result {
            return manager;
        }

        tries += 1;

        if tries >= max {
            result.unwrap();
        }

        tokio::time::sleep(Duration::from_secs(5)).await
    }

    Err("Chegou ao fim do ciclo sem finalizar").unwrap()
}

async fn manager(force_new_token: bool) -> Result<Manager> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}/v1/projects/ebisu-mobile/remoteConfig", BASE_ENDPOINT.as_str());
    
    
    let request = client.get(endpoint)
        .bearer_auth(authentication::get_auth_token(force_new_token).await?);


    let raw = request.send()
      .await.unwrap()
      .text()
      .await
      .unwrap();


    match serde_json::from_str(&raw) {
        Ok(config) => Ok(Manager { config, raw }),
        Err(_) => {
            println!("Não foi possivel decodificar resposta");
            println!("response = {}", &raw);
            Err(color_eyre::eyre::eyre!("Não foi possivel buscar configs"))
        },
    }
}

pub(crate) async fn update_manager(skeleton: String) -> Result<()> {
    let mut tries = 0;
    let max = 3;

    while tries < max {
        let result = update(&skeleton, tries > 0).await;

        if result.is_ok() {
            return Ok(());
        } else {
            tries += 1;

            if tries >= max {
                result.unwrap();
            }
    
            tokio::time::sleep(Duration::from_secs(5)).await
        }
    }

    Err("Chegou ao fim do ciclo sem finalizar").unwrap()
    
}

async fn update(skeleton: &str, force_authenticate: bool) -> Result<()>  {
    let endpoint = format!("{}/v1/projects/ebisu-mobile/remoteConfig", BASE_ENDPOINT.as_str());
    let client = reqwest::Client::new();
    let token = authentication::get_auth_token(force_authenticate).await?;


    let result = client.put(endpoint)
    .bearer_auth(token)
    .body(String::from(skeleton))
    .send()
    .await;

    match result {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(color_eyre::eyre::eyre!("Não foi possivel enviar para a google"));
            }

            Ok(())
        },
        Err(_) => Err(color_eyre::eyre::eyre!("Não foi possivel enviar para a google")),
    }
}


#[derive(Debug, Deserialize)]
struct RemoteConfigParameters {
    #[serde(rename(deserialize = "FIRELINK_ENDPOINT"))]
    firelink_endpoint: RemoteConfigParameter
}


#[derive(Debug, Deserialize)]
struct RemoteConfigParameter {
    #[serde(rename(deserialize = "defaultValue"))]
    default_value: RemoveConfigValue
}


#[derive(Debug, Deserialize)]
struct RemoveConfigValue {
    value: String
}
