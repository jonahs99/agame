extern crate ws;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

type ClientId = usize;
type ClientPipe<I, O> = (mpsc::Receiver<I>, mpsc::SyncSender<O>);

// Server lives in the main thread, interface for rx, tx
pub struct Server<I, O> {
    new_clients_rx: mpsc::Receiver<(ClientId, ClientPipe<I, O>)>,
    
    clients: HashMap<ClientId, ClientPipe<I, O>>,
}

// Listener will live in another thread
pub struct Listener<I, O> {
    new_clients_tx: mpsc::SyncSender<(ClientId, ClientPipe<I, O>)>,
    capacity: usize, 

    client_count: usize,
}

// Not sure why this has to be public
pub struct Handler<I, O> {
    id: ClientId,
    sender: ws::Sender,

    tx: mpsc::SyncSender<I>,
    rx: Option<mpsc::Receiver<O>>,
}

// Constructs a server and a bound listener
pub fn new<I, O>(capacity: usize) -> (Server<I, O>, Listener<I, O>) {
    let (new_clients_tx, new_clients_rx) = mpsc::sync_channel(4);

    (Server {
        new_clients_rx,
        clients: HashMap::new(),
    }, Listener {
        new_clients_tx,
        capacity,
        client_count: 0,
    })
}

impl <I, O> Server<I, O> {
    pub fn messages<'a>(&'a mut self) -> Box<Iterator<Item=(ClientId, I)> + 'a> {
        while let Ok((id, pipe)) = self.new_clients_rx.try_recv() {
            self.clients.insert(id, pipe);
        }

        Box::new(
            self.clients.iter()
                .map(|(id, (rx, _))| (id, rx.try_recv()))
                .filter_map(|(id, msg)| match msg {
                    Ok(msg) => Some((*id, msg)),
                    _ => None
                })
        )
    }
}

impl <I, O> Listener<I, O>
    where O: std::marker::Send + 'static, I: std::marker::Send + 'static,
          Listener<I, O>: ws::Factory {

    pub fn listen_on_thread(self, addr: &'static str) {
        thread::spawn(move || {
            // let listener = RefCell::new(self);

            ws::WebSocket::new(self).unwrap().listen(addr).unwrap();
        });
    }
}

impl <I, O> ws::Factory for Listener<I, O>
    where Handler<I, O>: ws::Handler {

    type Handler = Handler<I, O>;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        let id = self.client_count;
        self.client_count += 1;

        let (itx, irx) = mpsc::sync_channel(self.capacity);
        let (otx, orx) = mpsc::sync_channel(self.capacity);

        if self.new_clients_tx.try_send((id, (irx, otx))).is_err() {
            sender.close(ws::CloseCode::Again);     
        }

        Handler {
            id,
            sender,

            tx: itx,
            rx: None,
        }
    }
}

impl <I, O> ws::Handler for Handler<I, O>
    where I: serde::de::DeserializeOwned {

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        // For now only allow text input
        if let ws::Message::Text(msg) = msg {
            if let Ok(data) = serde_json::from_str(&msg) {
                if self.tx.try_send(data).is_err() {
                    self.sender.close(ws::CloseCode::Protocol);
                    self.sender.shutdown();
                }
            }
        }
        Ok(())
    }
}
