#[macro_use]
extern crate log;

use jager::database;
use jager::esi_models::ESIKillmail;
use jager::killmail_processing;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time;

use websocket_lite::{ClientBuilder, Message, Opcode};

const WEBSOCKET_URL: &str = "wss://zkillboard.com/websocket/";
const SUBSCRIPTION_MESSAGE: &str = "{\"action\":\"sub\", \"channel\":\"killstream\"}";

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Establishing database connection");
    let db = database::establish_connection().await.unwrap();
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    tokio::spawn(async move {
        loop {
            let message = rx.recv().unwrap();
            let km: ESIKillmail = serde_json::from_str(message.as_text().unwrap()).unwrap();
            info!("Processing new killmail {}", &km.killmail_id);
            match killmail_processing::process_killmail(&db, km).await {
                Ok(_) => {}
                Err(e) => error!("Failed to process killmail: {:?}", e),
            }
        }
    });
    let client_builder = ClientBuilder::new(WEBSOCKET_URL).unwrap();
    info!("Connected to websocket url: {}", WEBSOCKET_URL);
    let mut client = client_builder.connect().unwrap();
    let sub_msg = Message::text(SUBSCRIPTION_MESSAGE);
    let _result = client.send(sub_msg).unwrap();
    info!("Sent subscription request: {}", SUBSCRIPTION_MESSAGE);

    loop {
        let message = client.receive().unwrap();
        if let Some(message) = message {
            let opcode = message.opcode();
            match opcode {
                Opcode::Text => {
                    tx.send(message).unwrap();
                }
                Opcode::Ping => {
                    let pong = Message::pong("");
                    let res = client.send(pong);
                    match res {
                        Ok(_x) => info!("Got ping, Sent pong"),
                        Err(error) => error!("Error replying: {:?}", error),
                    }
                }
                _ => info!("Got other"),
            }
        } else {
            thread::sleep(time::Duration::from_millis(2));
        }
    }
}
