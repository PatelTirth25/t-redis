use std::sync::Arc;

use config::Config;
use helper_func::{extract_command, unpack_bulk_str};
use resp::RespHandler;
use storage::Storage;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use values::Value;
mod config;
mod helper_func;
mod resp;
mod storage;
mod values;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let storage: Arc<Mutex<Storage>> = Arc::new(Mutex::new(Storage::new()));
    let config: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::new("/tmp/", "rdb.db")));

    println!("Server Running at 127.0.0.1:6379");

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, _)) => {
                let storage = Arc::clone(&storage);
                let config = Arc::clone(&config);
                tokio::spawn(async move { handle_msg(stream, storage, config).await });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_msg(stream: TcpStream, storage: Arc<Mutex<Storage>>, config: Arc<Mutex<Config>>) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await;

        let response = if let Ok(v) = value {
            let (command, args) = extract_command(v).unwrap();
            println!("Args: {:#?}", args);
            let mut storage = Mutex::lock(&storage).await;
            let config = Mutex::lock(&config).await;
            match command.as_str().to_lowercase().as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    if args.len() <= 2 {
                        storage.set(
                            args[0].clone(),
                            args[1].clone(),
                            Value::BulkString("0".to_string()),
                        )
                    } else {
                        match unpack_bulk_str(args[2].clone())
                            .unwrap()
                            .as_str()
                            .to_lowercase()
                            .as_str()
                        {
                            "px" => storage.set(args[0].clone(), args[1].clone(), args[3].clone()),
                            _ => {
                                eprintln!("Unsupported tag for set");
                                Value::Null
                            }
                        }
                    }
                }
                "get" => storage.get(args[0].clone()),
                "config" => {
                    if args.len() < 2 || unpack_bulk_str(args[0].clone()).unwrap() != "get" {
                        Value::Null
                    } else {
                        match unpack_bulk_str(args[1].clone())
                            .unwrap()
                            .as_str()
                            .to_lowercase()
                            .as_str()
                        {
                            "dir" => config.dir(),
                            "dbfilename" => config.dbfilename(),
                            _ => Value::Null,
                        }
                    }
                }
                _ => Value::Null,
            }
        } else {
            break;
        };
        println!("Sending value {:?}", response);
        handler.write_value(response).await.unwrap();
    }
}
