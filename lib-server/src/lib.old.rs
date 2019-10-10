extern crate ws;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use std::cell::RefCell;

use std::thread;
use std::sync::mpsc;

type ClientId = usize;

pub struct MessageServer<I> {
    source: Option<MessageSource<I>>,
    sink: MessageSink<I>,
}

impl <I> MessageServer<I> where I: std::marker::Send {
    pub fn new(capacity: usize) -> MessageServer<I> {
        let (tx, rx) = mpsc::sync_channel(4);
        MessageServer {
            source: Some(MessageSource {
                receiver_tx: tx,
                capacity,
                buckets: HashMap::new(),
                id_counter: 0,
            }),
            sink: MessageSink {
                receiver_rx: rx,
                receivers: HashMap::new(),
            },
        }
    }

    pub fn listen_on_thread<A>(&mut self, addr: A) where
        A: std::net::ToSocketAddrs + std::fmt::Debug + std::marker::Send + 'static {

            if let Some(source) = self.source.take() {
                thread::spawn(move || {
                    let source = RefCell::new(source);

                    ws::listen(addr, |client| {
                        let server = ClientHandler::new(client.clone(), &source);
                        server
                    }).unwrap(); 
                });
            }
        }
    
    pub fn messages<'a>(&'a mut self) -> Box<Iterator<Item=(ClientId, I)> + 'a> {
        let sink = &mut self.sink;

        // Handle new clients
        while let Ok((id, rx)) = sink.receiver_rx.try_recv() {
            sink.receivers.insert(id, rx);
        }

        // Poll clients for messages
        Box::new(
            sink.receivers.iter()
            .filter_map(|(id, rx)| {
                match (id, rx.try_recv()) {
                    (id, Ok(msg)) => Some((*id, msg)),
                    _ => None,
                }
            })
            )
    }
}

struct MessageSource<T> {
    receiver_tx: mpsc::SyncSender<(ClientId, mpsc::Receiver<T>)>,

    capacity: usize,
    buckets: HashMap<ClientId, mpsc::SyncSender<T>>,

    id_counter: ClientId,
}

struct MessageSink<T> {
    receiver_rx: mpsc::Receiver<(ClientId, mpsc::Receiver<T>)>,
    receivers: HashMap<ClientId, mpsc::Receiver<T>>,
}

impl <T> MessageSource<T> where T: serde::de::Deserialize {
    pub fn on_open(&mut self) -> ClientId {
        let id = self.id_counter;
        let (source, sink) = mpsc::sync_channel(self.capacity);

        self.buckets.insert(id, source); 
        self.receiver_tx.send((id, sink));

        self.id_counter += 1;
        id
    }

    pub fn on_message(&mut self, id: ClientId, msg: ws::Message) -> Result<(), ()> {
        if let Some(bucket) = self.buckets.get(&id) {
            if let ws::Message::Text(text) = msg {
                if let Ok(data) = serde_json::from_str::<T>(&text) {
                    return match bucket.try_send(data) {
                        Ok(_) => Ok(()),
                        Err(_) => Err(()),
                    }
                }
            }
        }
        return Err(())
    }
}

struct ClientHandler<'a, I> {
    id: Option<ClientId>,
    out: ws::Sender,
    queue: &'a RefCell<MessageSource<I>>,
}
impl <I> ClientHandler<'_, I> {
    fn new<'a>(out: ws::Sender, queue: &'a RefCell<MessageSource<I>>) -> ClientHandler<'a, I> {
        ClientHandler {
            id: None,
            out,
            queue,
        }
    }
}

impl <I> ws::Handler for ClientHandler<'_, I> {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        self.id = Some(self.queue.borrow_mut().on_open());
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Some(id) = self.id {
            if let Err(_) = self.queue.borrow_mut().on_message(id, msg) {
                // The buffer is full, close the connection
                self.out.close(ws::CloseCode::Protocol);
            }
            return Ok(())
        }
        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        match code {
            ws::CloseCode::Normal => println!("client closed connection."),
            ws::CloseCode::Away   => println!("client left."),
            ws::CloseCode::Abnormal => println!("closing handshake failed"),
            ws::CloseCode::Protocol => println!("server closed connection"),
            _ => println!("client error: {:?}, {}", code, reason),
        }
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

