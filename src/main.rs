extern crate time;
extern crate sha1;
extern crate bincode;
extern crate termion;

extern crate serde;
#[macro_use] extern crate serde_derive;

mod hash_content;
mod block;
mod blocks;
mod peers;
mod help;
mod display;

use std::io::Read;
use std::net::{
    TcpListener,
    SocketAddr,
};
use std::thread::spawn;
use std::sync::{
    Arc,
    Mutex,
};

use bincode::deserialize;

use block::Block;

use blocks::{
    list_blocks,
    broadcast_block,
};

use peers::{
    create_peer,
    list_peers,
};

use help::list_commands;

use display::{
    DEFAULT_STATUS,
    set_status_text,
    clear_screen,
    get_input,
    set_cursor_into_logs,
    set_cursor_into_input,
};

/// Handle incoming TCP connections with other nodes.
fn handle_incoming_connections() {

    let listener = TcpListener::bind("0.0.0.0:10000").unwrap();

    for income in listener.incoming() {

        /* TODO: display message when receive a connection;
           should use mutex as it must modify the content
           of the main text area (so the cursor position
           must not be modified) */

        set_cursor_into_logs();

        let mut stream = income.unwrap();
        println!("Received block from {}.", stream.peer_addr().unwrap());

        let mut buffer: Vec<u8> = Vec::new();

        stream.read_to_end(&mut buffer).unwrap();

        let block: Block = deserialize(&buffer).unwrap();

        /* FIXME: add the block into the local chain */

        set_cursor_into_input();
    }
}

fn main() {

    clear_screen();

    let chain: Arc<Mutex<Vec<Block>>> = Arc::new(Mutex::new(Vec::new()));
    let mut peers: Vec<SocketAddr> = Vec::new();

    spawn(|| { handle_incoming_connections() });

    loop {

        set_status_text(DEFAULT_STATUS);

        let input = get_input();
        let splitted: Vec<&str> = input.split(' ').collect();

        /* get() returns &&str, so we mention result type &str
           and get it from a reference (*) */
        let command: &str = match splitted.get(0) {
            Some(value) => *value,
            None => { continue; }
        };

        const ADD_BLOCK: &str = "add_block";
        const SEE_BLOCKCHAIN: &str = "list";
        const ADD_PEER: &str = "add_peer";
        const LIST_PEERS: &str = "list_peers";
        const HELP: &str = "help";

        let option = match splitted.get(1) {
            Some(option) => option,
            None => {

                if command == ADD_BLOCK ||
                    command == ADD_PEER {
                    continue;
                }

                ""
            }
        };

        if command == ADD_BLOCK {

            let data: i32 = option.parse().unwrap();
            let mut chain = chain.lock().unwrap();

            let mut previous_digest = String::new();

            if !chain.is_empty() {

                previous_digest = chain.last()
                    .unwrap()
                    .get_current()
                    .to_string();
            }

            let block = Block::new(data, previous_digest);
            chain.push(block.clone());

            println!("New block added.");

            broadcast_block(&peers, block);
        }
        else if command == SEE_BLOCKCHAIN {
            list_blocks(&chain);
        }
        else if command == ADD_PEER {
            create_peer(&mut peers, option);
        }
        else if command == LIST_PEERS {
            list_peers(&peers);
        }
        else if command == HELP {
            list_commands();
        }
    }
}
