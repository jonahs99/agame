extern crate server;
extern crate serde;

use std::thread;
use std::time::Duration;

#[derive(Debug, serde::Deserialize)]
enum Input {
    Move {x: i32, y: i32},
}

enum Output {
    Update(Grid),    
}

struct Grid {
    cells: i32,
}

fn main() {
    let (mut message_server, listener) = server::new::<Input, Output>(4);

    listener.listen_on_thread("127.0.0.1:5000");

    loop {
        for (id, msg) in message_server.messages() {
            println!("From {}: {:?}", id, msg);
        }
        thread::sleep(Duration::from_millis(100));
    }
} 
