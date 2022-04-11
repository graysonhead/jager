#[macro_use]
extern crate log;

use backend::database;
use backend::killmail_processing;
use backend::logging;
use datamodels::esi_models::ESIKillmail;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time;

use websocket_lite::{ClientBuilder, Message, Opcode};

const WEBSOCKET_URL: &str = "wss://zkillboard.com/websocket/";
const SUBSCRIPTION_MESSAGE: &str = "{\"action\":\"sub\", \"channel\":\"killstream\"}";

#[tokio::main]
async fn main() {
    info!("Establishing database connection");
    let db = database::establish_connection().await.unwrap();
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    logging::setup_logging();
    tokio::spawn(async move {
        loop {
            let message = rx.recv().unwrap();
            let km: ESIKillmail = serde_json::from_str(message.as_text().unwrap()).unwrap();
            let km_id = km.killmail_id;
            info!("Processing new killmail {}", &km.killmail_id);
            match killmail_processing::process_killmail(&db, km).await {
                Ok(_) => info!("Finished processing killmail: {}", km_id),
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
                    info!("Got ping from websocket");
                    let pong = Message::pong("");
                    let res = client.send(pong);
                    match res {
                        Ok(_x) => info!("Sent pong to websocket"),
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
