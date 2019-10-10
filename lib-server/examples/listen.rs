extern crate server;
extern crate serde;

use std::thread;
use std::time::Duration;

#[derive(Debug, serde::Deserialize)]
enum Input {
    Join(String),
    Position {x: i32, y: i32},
}

type Output = ();

fn main() {
    let (mut message_server, listener) = server::new::<Input, Output>(4);

    listener.listen_on_thread("127.0.0.1:3000");

    loop {
        for (id, msg) in message_server.messages() {
            println!("From {}: {:?}", id, msg);
            match msg {
                Input::Join(s) => println!("Join({})", s),
                Input::Position {x, y} => println!("Position({}, {})", x, y),
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
} 
