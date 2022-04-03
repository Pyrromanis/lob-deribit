extern crate core;

use crate::lib::{empty_change, empty_snapshot, get_change, get_snapshot, update_book};

mod lib;

use crate::Message::Text;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    //execute everything in a big loop, so in case of packet loss we just use continue and on the
    //next loop cycle we connect again
    loop {
        let address = "wss://www.deribit.com/ws/api/v2";

        let url = url::Url::parse(&address).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        println!("\nWebSocket handshake has been successfully completed!\n");

        //split the stream to write that we use to perform the subscribe action and read which is the stream
        let (mut write, mut read) = ws_stream.split();

        let data = r#"{
        "jsonrpc": "2.0",
         "method": "public/subscribe",
         "id": 42,
         "params": {
        "channels": ["book.BTC-PERPETUAL.100ms"]}
        }"#;
        let v: Value = serde_json::from_str(data).expect("Failed to marshal");

        write
            .send(Message::Text(v.to_string()))
            .await
            .expect("Failed to send message");

        //we skip the first message that tells us everything went fine with subscribing
        read.next().await;
        let first_message = read.next().await;

        //if everything went nicely get the snapshot
        let snapshot;
        match first_message {
            Some(Ok(Text(msg))) => {
                snapshot = get_snapshot(&msg);
            }
            _ => snapshot = empty_snapshot(),
        };
        let mut book = match snapshot {
            Ok(book) => book,
            Err(error) => panic!("Error getting the snapshot: {}!", error),
        };
        //the timer for the print function
        let mut interval = time::interval(Duration::from_secs(1));
        //the book transformation/printing loop. Keep reading (unless id mismatch) and transforming
        //as well as printing
        loop {
            tokio::select! {
               msg=read.next()=>{
               let msg = match msg {
                    Some(msg) => msg,
                    None => break,
                };
                let message_change=msg;
                let change;
                match message_change {
                    Ok(Text(msg))=>{
                        change = get_change(&msg);
                    },
                    _=>change=empty_change(),

                }
                let change=match change{
                    Ok(change)=>change,
                    Err(error)=>panic!("Error getting the change: {}!",error),
                };
                let update=update_book(&mut book, change.params.data);
                match update{
                    Err(error)=>panic!("Error updating: {}!",error),
                    //break if we need to recconect
                    Ok(1)=>break,
                    _=>(),
                };
                },
                _=interval.tick()=>{
                    println!("Highest Bid is: {} with {} quantity, while Lowest Ask is: {} with {} quantity!",book.highest_bid,book.highest_quantity,book.lowest_ask,book.lowest_quantity);
                },
            }
        }
        //we only get here in case we need to reconnect
        continue;
    }
}
