use std::env;

use rustia_proxy::*;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let login_listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7173".to_string());
    let login_server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:7171".to_string());

    // let game_listen_addr = env::args()
    //     .nth(3)
    //     .unwrap_or_else(|| "127.0.0.1:7174".to_string());
    // let game_server_addr = env::args()
    //     .nth(4)
    //     .unwrap_or_else(|| "127.0.0.1:7172".to_string());

    let login = Proxy::builder("127.0.0.1:7173".to_string(), "127.0.0.1:7171".to_string())
        .with_event_handler(debug::DebugEventHandler::new_boxed("Login".to_string()))
        .with_event_handler(login::LoginHandshaker::new_boxed())
        .with_event_handler(login::GameServerInjector::new_boxed("127.0.0.1".to_string(), 7174))
        .build();

    let game = Proxy::builder("127.0.0.1:7174".to_string(), "127.0.0.1:7172".to_string())
        .with_event_handler(debug::DebugEventHandler::new_boxed("Game".to_string()))
        .with_event_handler(game::GameHandshaker::new_boxed())
        .build();

    tokio::join!(
        tokio::spawn(game.run()),
        tokio::spawn(login.run()),
    );
    
    Ok(())
}
