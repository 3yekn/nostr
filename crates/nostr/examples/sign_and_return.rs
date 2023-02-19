// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use std::str::FromStr;

use nostr::secp256k1::SecretKey;
use nostr::Timestamp;
use nostr::SECP256K1;
use nostr::{
    event::EventBuilder, event::Marker, ClientMessage, Filter, Keys, Kind, RelayMessage, Result, SubscriptionId,
    Tag,
};
use tungstenite::{connect, Message as WsMessage};
// use secp256k1;

const ALICE_SK: &str = "0e1db7418df1c6453ce42e7f4507b8823fc23e86e1f4f33d7fafc83d366e6e97";
const BOB_SK: &str = "5e79d85b377943fed828365d2a7712a0578272b6c1e0511154f6517e2a13925e";
// const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
const WS_ENDPOINT: &str = "wss://r1.hashed.systems";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) = connect(WS_ENDPOINT).expect("Can't connect");

    let bob_keys = Keys::new(SecretKey::from_str(BOB_SK)?);
    let alice_keys = Keys::new(SecretKey::from_str(ALICE_SK)?);

    let subscribe_to_bob = ClientMessage::new_req(
        SubscriptionId::new("wait-for-signature-request"),
        vec![Filter::new()
            .authors(vec![bob_keys.public_key()])
            .since(Timestamp::now())
            // subscribe to Signature Requests, NIP-70, Kind=9999 (WIP)
            .kind(Kind::SignatureRequest),
        ],
            // .kind(Kind::TextNote)],
    );

    socket.write_message(WsMessage::Text(subscribe_to_bob.as_json()))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
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

                    if e.kind == Kind::SignatureRequest {
                        println!("Payload to sign   : {}", e.content);
                        let raw_payload_to_sign = e.content.as_bytes();
                        println!("Raw payload to sign   : {:?}", raw_payload_to_sign);
    
                        // hash the message
                        let hashed_message = Sha256Hash::hash(raw_payload_to_sign);
                        println!("Hashed message   : {:#?}", hashed_message);
    
                        let payload_to_sign = secp256k1::Message::from_slice(&hashed_message)?;
    
                        const MEMO: &str = "sgtm";
    
                        let tag_original_request = Tag::Event(
                            e.id,
                            Some("wss://r1.hashed.systems".to_string()),
                            Some(Marker::Reply),
                        );
    
                        let tag_bob_in_reply =
                            Tag::PubKey(e.pubkey, Some("wss://r1.hashed.systems".to_string()));
    
                        let signature =
                            SECP256K1.sign_schnorr(&payload_to_sign, &alice_keys.key_pair()?);
                        let alice_signs_the_message_and_responds = ClientMessage::new_event(
                            EventBuilder::new_signature_response(
                                signature,
                                Some(MEMO.to_string()),
                                &[tag_original_request, tag_bob_in_reply],
                            )
                            .to_event(&alice_keys)?,
                        );
    
                        println!("Signature : {}", signature.to_string());
    
                        socket.write_message(WsMessage::Text(alice_signs_the_message_and_responds.as_json()))?;
                        let msg = socket.read_message().expect("Error reading message");
                        let msg_text = msg.to_text().expect("Failed to convert message to text");
                    
                        println!("REPLY    : {:#?}", msg_text);
                    }                    
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
