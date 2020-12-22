#[tokio::main]
async fn main() {
    env_logger::init();
    rust_ots::main().await;
}
