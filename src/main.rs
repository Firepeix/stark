#[tokio::main]
async fn main() {
    operate().await
}

async fn operate() {
    color_eyre::install().unwrap();
    stark::start().await;
}
