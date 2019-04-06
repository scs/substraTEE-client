
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hex_literal;


use serde_json::{json, Value};

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use std::i64;
use regex::Regex;
use keyring::AccountKeyring;
use node_primitives::AccountId;
mod extrinsic;
use crate::extrinsic::{transfer};

#[macro_use]
use hex;
use parity_codec::{Encode, Decode};
use transaction_pool::txpool::ChainApi as PoolChainApi;
use runtime_primitives::{generic, traits};
use node_primitives::Hash;
//use twox_hash::{two};
use primitives::twox_128;

const REQUEST_GENESIS_HASH: u32     = 1;
const REQUEST_METADATA: u32         = 2;
const REQUEST_TRANSFER: u32         = 3;
const REQUEST_GET_STORAGE: u32      = 4;

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
    chain: Chain,
}

impl Client {
    pub fn new(out: Sender) -> Client {
        Client {
            out: out,
            chain: Default::default(),
        }
    }

    fn author_submitAndWatchExtrinsic(&mut self, nonce: u64) {
        // send a transaction
        let xt= transfer("//Alice", "//Bob", 42, nonce, self.chain.genesis_hash);
        println!("extrinsic: {:?}", xt);

        let mut xthex = hex::encode(xt.encode());

        xthex.insert_str(0, "0x");
        let jsonreq = json!({
            "method": "author_submitAndWatchExtrinsic", 
            "params": [xthex], 
            "jsonrpc": "2.0",
            "id": REQUEST_TRANSFER.to_string(),
        });
        

        println!("sending extrinsic: {}", jsonreq.to_string());
        self.out.send(jsonreq.to_string()).unwrap();
    }

    fn state_getStorage(&mut self, module: &str, storage_key_name: &str, param: Vec<u8>) {
        let mut key = module.as_bytes().to_vec();
        key.append(&mut vec!(' ' as u8));
        key.append(&mut storage_key_name.as_bytes().to_vec());
        //key.append(&mut vec!(' ' as u8));
        key.append(&mut param.clone());
        println!("will query storage for: {:?}", key);
        let mut keyhash = hex::encode(twox_128(&key));
        keyhash.insert_str(0, "0x");
        println!("with storage key: {}", keyhash);
        let jsonreq = json!({
            "method": "state_getStorage",
            "params": [keyhash], 
            //"params": ["0xc99f5446efa57788f39ab529311f4550"],
            "jsonrpc": "2.0",
            "id": REQUEST_GET_STORAGE.to_string(),
        });
        println!("sending extrinsic: {}", jsonreq.to_string());
        self.out.send(jsonreq.to_string()).unwrap();
    }
}

#[derive(Default)]
struct Chain {
    blocknumber: i64,
    genesis_hash: Hash,
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
        
        //get genesis_hash
        //self.out.send(r#"{"method": "chain_getBlockHash", "params": [0], "jsonrpc": "2.0", "id": 0}"#);
        let jsonreq = json!({
            "method": "chain_getBlockHash", 
            "params": [0], 
            "jsonrpc": "2.0",
            "id": REQUEST_GENESIS_HASH.to_string(),
        });
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

        match value["id"].as_str() {
            Some(idstr) => { match idstr.parse::<u32>() {
                Ok(REQUEST_GENESIS_HASH) => {
                    let mut hexstr = value["result"].as_str().unwrap().to_string();
                    if hexstr.starts_with("0x") {
                        hexstr.remove(0);
                        hexstr.remove(0);
                        let mut gh: [u8; 32] = Default::default();
                        gh.copy_from_slice(&hex::decode(&hexstr).unwrap());
                        self.chain.genesis_hash = Hash::from(gh);
                        println!("genesis_hash is 0x{}", hexstr);
                        // FIXME: some state machine logic would be better than directly calling
                        
                        let accountid = AccountId::from(AccountKeyring::Alice);
                        self.state_getStorage("System", "AccountNonce", accountid.encode())  
                    } else {
                        panic!("result should be 0x prefixed hex: {}", hexstr);
                    }

                },
                Ok(REQUEST_GET_STORAGE) => {
                    let mut hexstr = match value["result"].as_str() {
                        Some(res) => res.to_string(),
                        _ => "0x00".to_string(),
                    };

                    if hexstr.starts_with("0x") {
                        hexstr.remove(0);
                        hexstr.remove(0);  
    /*                    let nonce = match u64::from_str_radix(&hexstr, 16) {
                            Ok(int) => int,
                            _ => 0,
                        }; */
                        let nonce = 0;
                        println!("nonce is {}", nonce);
                        self.author_submitAndWatchExtrinsic(nonce);
                    } else {
                        panic!("result should be 0x prefixed hex: {}", hexstr);
                    }

                },
                Ok(REQUEST_TRANSFER) => {
                    match value.get("error") {
                        Some(err) => println!("ERROR: {:?}", err),
                        _ => println!("no error"),
                    }
                },
                Ok(_) => println!("unknown request id"),
                Err(_) => println!("error assigning request id"),
            }},
            _ => {
                // subscriptions
                println!("no id field found in response. must be subscription");
                println!("method: {:?}", value["method"].as_str());
                match value["method"].as_str() {
                    Some("author_extrinsicUpdate") => println!("author_extrinsicUpdate: {:?}", value["params"]["result"]),
                    _ => println!("unsupported method"),
                }
            },
        };

        // FIXME: graceful error handling!!!
/*
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
        */

        Ok(())
    }
}

fn main() {
  // Now, instead of a closure, the Factory returns a new instance of our Handler.
  connect("ws://127.0.0.1:9944", |out| Client::new(out)).unwrap()
}