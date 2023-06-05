
mod google;
mod hospital;
mod controller;


use controller::CommandMessage;
pub use google::generate_request_jwt;
use tokio::sync::broadcast::channel;


pub async fn start() {
    let (dispatcher, listener) = channel::<CommandMessage>(32);
    let pacient = google::get_manager().await;
    hospital::enter(pacient, dispatcher, listener).await
}
