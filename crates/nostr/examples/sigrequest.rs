// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::SecretKey;
use nostr::{
    ClientMessage, EventBuilder, Keys, Result, Tag,
};
use tungstenite::{connect, Message as WsMessage};

const ALICE_SK: &str = "0e1db7418df1c6453ce42e7f4507b8823fc23e86e1f4f33d7fafc83d366e6e97";
const BOB_SK: &str = "5e79d85b377943fed828365d2a7712a0578272b6c1e0511154f6517e2a13925e";
// const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
const WS_ENDPOINT: &str = "wss://r1.hashed.systems";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) = connect(WS_ENDPOINT).expect("Can't connect");

    let alice_keys = Keys::new(SecretKey::from_str(ALICE_SK)?);
    let bob_keys = Keys::new(SecretKey::from_str(BOB_SK)?);

    let bob_tags_alice = Tag::PubKey(alice_keys.public_key(), Some(WS_ENDPOINT.to_string()));

    const PAYLOAD_REQUEST: &str = "I hereby approve of eating the filet tonight";
    const MEMO: &str = "please approve this because I already started the grill";
    let bob_asks_alice_to_sign = ClientMessage::new_event(
        EventBuilder::new_signature_request(PAYLOAD_REQUEST,Some(MEMO.to_string()),&[bob_tags_alice]).to_event(&bob_keys)?,
    );

    socket.write_message(WsMessage::Text(bob_asks_alice_to_sign.as_json()))?;
    let msg = socket.read_message().expect("Error reading message");
    let msg_text = msg.to_text().expect("Failed to convert message to text");

    println!("REPLY    : {:#?}", msg_text);
    Ok(())
}
