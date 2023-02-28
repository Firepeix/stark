use serde::{Deserialize};

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
    let request = reqwest::get("http://localhost:3000/lobby");
    let raw = request
      .await.unwrap()
      .text().await.unwrap();

    Manager {
        config: serde_json::from_str(&raw).unwrap(),
        raw
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
