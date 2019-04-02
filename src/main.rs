
#[macro_use]
extern crate serde_derive;

use serde_json::{json, Value};

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use std::i64;
use regex::Regex;
//use test_client::{runtime::{AccountId, Block, Hash, Index, Extrinsic, Transfer}, AccountKeyring::{self, *}};
use keyring::AccountKeyring;

mod extrinsic;
use crate::extrinsic::{Extrinsic, Transfer};
use hex;
use parity_codec::{Encode, Decode};

#[derive(Serialize, Deserialize, Debug)]
struct JsonBasic {
    jsonrpc: String,
    method: String,
    params: String,
}

// Our Handler struct.
// Here we explicity indicate that the Client needs a Sender,
// whereas a closure captures the Sender for us automatically.
struct Client {
    out: Sender,
    blocknumber: i64,
    id: i64,
}

// We implement the Handler trait for Client so that we can get more
// fine-grained control of the connection.
impl Handler for Client {

    // `on_open` will be called only after the WebSocket handshake is successful
    // so at this point we know that the connection is ready to send/receive messages.
    // We ignore the `Handshake` for now, but you could also use this method to setup
    // Handler state or reject the connection based on the details of the Request
    // or Response, such as by checking cookies or Auth headers.
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // Now we don't need to call unwrap since `on_open` returns a `Result<()>`.
        // If this call fails, it will only result in this connection disconnecting.
        //self.out.send(r#"{"method": "chain_subscribeNewHead", "params": null, "jsonrpc": "2.0", "id": 0}"#);

        // send a transaction
        let nonce = 0;
        let tx = Transfer {
            amount: 42,
            nonce,
            from: AccountKeyring::Alice.into(),
            to: AccountKeyring::Bob.into(),
        };
        let signature = AccountKeyring::from_public(&tx.from).unwrap().sign(&tx.encode()).into();
        let xt = Extrinsic::Transfer(tx, signature).encode();
        let mut xthex = hex::encode(xt);
        xthex.insert_str(0, "0x");
        let jsonreq = json!({
            "method": "author_submitAndWatchExtrinsic",
            "params": [xthex], // params,
            "jsonrpc": "2.0",
            "id": self.id.to_string(),
        });
        println!("sending extrinsic: {}", jsonreq.to_string());
        self.out.send(jsonreq.to_string()).unwrap();

        Ok(())
        
    }

    // `on_message` is roughly equivalent to the Handler closure. It takes a `Message`
    // and returns a `Result<()>`.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Close the connection when we get a response from the server
        println!("Got message: {}", msg);
        let retstr = msg.as_text().unwrap();

        let value: serde_json::Value = serde_json::from_str(retstr).unwrap();
        //let value: serde_json::Value = serde_json::from_str(r#"{"jsonrpc":"2.0","method":"chain_newHead","params":{"result":{"digest":{"logs":["0x046175726121015043281700000000869742ce3fbb1bc9674f1bfc48bf75fed995b5feb994da1f451ac18748877dd6cffcef8bac7a9e532331de1f36f0a4bcd3b18aeb55e00ab21de4f498a51a160e"]},"extrinsicsRoot":"0x5aa69263b009b0b58f1cf0fb7b1cff8b0e63c64eb398d2590da6d646b33543db","number":"0x38f6","parentHash":"0x3e5828e315bf0d69de77ec767df37c53d219463d8c3eb87b0a200cd285dd9d34","stateRoot":"0x2440100547e417cfb47d25390f4ec112642af1421eb4d9ca3dbe320f153f2f03"},"subscription":3}}"#).unwrap();
        //println!("params: {:?}", value);

        // FIXME: graceful error handling!!!

        let _hexstr = value["params"]["result"]["number"].to_string();
        //println!("block number hex: {:?}", _hexstr);
        let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
        let mut _hexclean = String::from(re.replace_all(&_hexstr, ""));
        //println!("block number hex clean: {:?}", _hexclean);
        _hexclean.remove(0);
        _hexclean.remove(0);
        let _unhex = match i64::from_str_radix(&_hexclean, 16) {
            Ok(int) => int,
            _ => 0,
        };
            
        self.blocknumber = _unhex;

        println!("block number : {:?}", _unhex);
        //self.out.close(CloseCode::Normal)
        Ok(())
    }
}

fn main() {
  // Now, instead of a closure, the Factory returns a new instance of our Handler.
  connect("ws://127.0.0.1:9944", |out| Client { out: out, blocknumber: 0, id: 0 } ).unwrap()
}