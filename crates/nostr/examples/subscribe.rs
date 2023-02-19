// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::SecretKey;
use nostr::{
    ClientMessage, Filter, Keys, Kind, RelayMessage, Result, SubscriptionId,
};
use tungstenite::{connect, Message as WsMessage};

const _ALICE_SK: &str = "0e1db7418df1c6453ce42e7f4507b8823fc23e86e1f4f33d7fafc83d366e6e97";
const BOB_SK: &str = "5e79d85b377943fed828365d2a7712a0578272b6c1e0511154f6517e2a13925e";
// const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
const WS_ENDPOINT: &str = "wss://r1.hashed.systems";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) = connect(WS_ENDPOINT).expect("Can't connect");

    let bob_keys = Keys::new(SecretKey::from_str(BOB_SK)?);

    let subscribe_to_bob = ClientMessage::new_req(
        SubscriptionId::new("abcdefgh"),
        vec![Filter::new()
            .authors(vec![bob_keys.public_key()])
            // subscribe to Signature Requests, NIP-70, Kind=9999 (WIP)
            .kind(Kind::SignatureRequest)],
    );

    socket.write_message(WsMessage::Text(subscribe_to_bob.as_json()))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        println!("MESSAGE : {:#?}", msg_text);
        if let Ok(handled_message) = RelayMessage::from_json(msg_text) {
            match handled_message {
                RelayMessage::Notice { message } => {
                    println!("Got a notice: {}", message);
                }
                RelayMessage::Event {
                    event: e,
                    subscription_id: _,
                } => {
                    println!("Got an event  : {:#?} ", e);
                }
                RelayMessage::EndOfStoredEvents(_subscription_id) => {
                    println!("Relay signalled End of Stored Events");
                }
                RelayMessage::Ok {
                    event_id,
                    status,
                    message,
                } => {
                    println!("Got OK message: {} - {} - {}", event_id, status, message);
                }
                RelayMessage::Auth { challenge } => {
                    println!("Got a auth challenge: {}", challenge);
                }
                RelayMessage::Empty => {
                    println!("Empty message");
                }
            }
        } else {
            println!("Got unexpected message: {}", msg_text);
        }
    }
}
