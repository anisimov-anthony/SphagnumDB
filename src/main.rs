use std::error::Error;
use std::sync::Arc;
use libp2p::Multiaddr;
use clap::{Arg, Command}; // Используем `Command` вместо `App`
use tokio::sync::Mutex;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use sproutdb::core::{
    sprout::Sprout, 
    req_resp_codec::{Command as SproutCommand} // Переименуем, чтобы избежать конфликта имен
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("SproutDB Node")
        .arg(Arg::new("addr") // Используем `Arg::new`
            .long("addr")
            .value_name("ADDRESS") // Указываем имя значения
            .help("Address of another node to connect to"))
        .get_matches();

    let mut sprout = Sprout::new()?;

    // Выводим PeerID текущей ноды
    println!("PeerID: {}", sprout.peer_id()?);

    // Запускаем ноду на случайном порту
    let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
    sprout.listen_on(listen_addr.clone())?;

    // Выводим адрес, на котором нода слушает
    if let Some(addr) = sprout.listeners().next() { // Добавьте метод `listeners` в `Sprout`
        println!("Node is listening on: {}", addr);
    }

    let _ = if let Some(addr) = matches.get_one::<String>("addr") {
        sprout.dial(addr)?;
        println!("Connected to node at: {}", addr);
        
    };

    let sprout_arc = Arc::new(Mutex::new(sprout));

    // Обработка событий
    let handle_events = {
        let sprout_arc = Arc::clone(&sprout_arc);
        tokio::spawn(async move {
            loop {
                let mut sprout = sprout_arc.lock().await;
                if let Err(e) = sprout.handle_event().await {
                    eprintln!("Error handling event: {}", e);
                }
            }
        })
    };

    // Чтение команд из терминала
    let handle_input = {
        let sprout_arc = Arc::clone(&sprout_arc);
        tokio::spawn(async move {
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin).lines();

            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                let mut parts = line.split_whitespace();
                if let Some(command) = parts.next() {
                    match command.to_lowercase().as_str() {
                        "get" => {
                            let mut sprout = sprout_arc.lock().await;
                            if let Some(peer_id) = sprout.connected_peers.iter().next().copied() {
                                
                                match sprout.send_request_to_sprout(peer_id, SproutCommand::Get, "".to_string()) {
                                    Ok(_) => println!("Get request sent"),
                                    Err(e) => eprintln!("Failed to send Get request: {}", e),
                                }
                            } else {
                                eprintln!("Not connected to any node.");
                            }
                        }
                        "set" => {
                            if let Some(value) = parts.next() {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied() {
                                    
                                    match sprout.send_request_to_sprout(peer_id, SproutCommand::Set, value.to_string()) {
                                        Ok(_) => println!("Set request sent with value: {}", value),
                                        Err(e) => eprintln!("Failed to send Set request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            } else {
                                eprintln!("Usage: set <value>");
                            }
                        }
                        _ => {
                            eprintln!("Unknown command: {}", command);
                        }
                    }
                }
            }
        })
    };

    let _ = tokio::join!(handle_events, handle_input);

    Ok(())
}