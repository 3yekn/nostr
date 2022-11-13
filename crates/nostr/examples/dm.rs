// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::error::Error;
use std::str::FromStr;
use std::{thread, time};

use nostr::event::KindBase;
use nostr::util::nips::nip04::decrypt;
use nostr::{ClientMessage, Event, Keys, Kind, RelayMessage, SubscriptionFilter};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
// const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
const WS_ENDPOINT: &str = "wss://relay.damus.io";
// const WS_ENDPOINT: &str = "ws://localhost:3333/ws";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let alice_keys = Keys::from_str(ALICE_SK)?;
    let bob_keys = Keys::from_str(BOB_SK)?;

    let alice_to_bob = "Hey bob this is alice (ping)";
    let bob_to_alice = "Hey alice this is bob (pong)";

    let alice_encrypted_msg =
        Event::new_encrypted_direct_msg(&alice_keys, &bob_keys, alice_to_bob)?;

    let subscribe_to_alice = ClientMessage::new_req(
        "abcdefg",
        vec![SubscriptionFilter::new()
            .authors(vec![alice_keys.public_key()])
            .pubkey(bob_keys.public_key())],
    );

    let subscribe_to_bob = ClientMessage::new_req(
        "123456",
        vec![SubscriptionFilter::new()
            .authors(vec![bob_keys.public_key()])
            .pubkey(alice_keys.public_key())],
    );

    println!("Subscribing to Alice");
    socket.write_message(WsMessage::Text(subscribe_to_alice.to_json()))?;
    println!("Subscribing to Bob");
    socket.write_message(WsMessage::Text(subscribe_to_bob.to_json()))?;

    socket.write_message(WsMessage::Text(
        ClientMessage::new_event(alice_encrypted_msg).to_json(),
    ))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to convert message to text");
        if let Ok(handled_message) = RelayMessage::from_json(msg_text) {
            match handled_message {
                RelayMessage::Empty => {
                    println!("Empty message")
                }
                RelayMessage::Notice { message } => {
                    println!("Got a notice: {}", message);
                }
                RelayMessage::EndOfStoredEvents { subscription_id: _ } => {
                    println!("Relay signalled End of Stored Events");
                }
                RelayMessage::Ok {
                    event_id,
                    status,
                    message,
                } => {
                    println!("Got OK message: {} - {} - {}", event_id, status, message);
                }
                RelayMessage::Event {
                    event,
                    subscription_id: _,
                } => {
                    if event.kind == Kind::Base(KindBase::EncryptedDirectMessage) {
                        if event.tags[0].content() == Some(&alice_keys.public_key_as_str()) {
                            println!("New DM to alice");
                            println!("Encrypted: {}", event.content);
                            println!(
                                "Decrypted: {}",
                                decrypt(
                                    &alice_keys.secret_key()?,
                                    &bob_keys.public_key(),
                                    &event.content
                                )?
                            );
                            thread::sleep(time::Duration::from_millis(5000));
                            let alice_encrypted_msg = Event::new_encrypted_direct_msg(
                                &alice_keys,
                                &bob_keys,
                                alice_to_bob,
                            )?;
                            socket.write_message(WsMessage::Text(
                                ClientMessage::new_event(alice_encrypted_msg).to_json(),
                            ))?;
                        } else if event.tags[0].content() == Some(&bob_keys.public_key_as_str()) {
                            println!("New DM to bob");
                            println!("Encrypted: {}", event.content);
                            println!(
                                "Decrypted: {}",
                                decrypt(
                                    &alice_keys.secret_key()?,
                                    &bob_keys.public_key(),
                                    &event.content
                                )?
                            );
                            thread::sleep(time::Duration::from_millis(5000));
                            let bob_encrypted_msg = Event::new_encrypted_direct_msg(
                                &bob_keys,
                                &alice_keys,
                                bob_to_alice,
                            )?;
                            socket.write_message(WsMessage::Text(
                                ClientMessage::new_event(bob_encrypted_msg).to_json(),
                            ))?;
                        }
                    } else {
                        println!("{:#?}", event);
                    }
                }
            }
        } else {
            println!("Received unexpected message: {}", msg_text);
        }
    }
}
