
#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();
    let health = stark::observe().await;
    dbg!(health);
}
