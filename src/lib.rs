
mod google;
mod hospital;
mod controller;


use color_eyre::Result;
use controller::CommandMessage;
pub use google::generate_request_jwt;
use ngrok::{prelude::TunnelBuilder, tunnel::UrlTunnel, Tunnel};
use tokio::sync::broadcast::channel;


pub async fn start() {
    let (dispatcher, listener) = channel::<CommandMessage>(32);
    let pacient = google::get_manager().await;
    hospital::enter(pacient, dispatcher, listener).await
}
