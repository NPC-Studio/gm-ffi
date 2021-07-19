use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, ToSocketAddrs},
    prelude::v1::*,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

/// A server for creating a TCP connection to a running GameMaker game.
/// We use this in the Bugger-rs project and within Tango to talk to the GM
/// game. This is an entirely sync, thread based Tcp model, not intended to be
/// used in async contexts.
///
/// It is **not** highly performant **or** portable, so please expect
/// to only use this in debugging and developer contexts.
#[derive(Debug)]
pub struct TcpServer {
    // we keep this guy around for basically no reason.
    #[allow(dead_code)]
    server_handle: JoinHandle<()>,
    outgoing: Sender<Outgoing>,
    incoming: Receiver<Incoming>,
    connected: bool,
}

enum Outgoing {
    Message(String),
    Kill,
}

enum Incoming {
    Message(String),
    Connected,
    Disconnected,
}

impl TcpServer {
    /// Creates a new tcp server at the given address.
    pub fn new<A: ToSocketAddrs + Send + Sync + Clone + 'static>(address: A) -> Self {
        let (outgoing, rx) = channel::<Outgoing>();
        let (tx, incoming) = channel::<Incoming>();

        // Thread for server
        let server_handle = thread::spawn(move || loop {
            std::println!("Waiting to connect to Mistria...");
            let (mut stream, _) = TcpListener::bind(address.clone())
                .unwrap()
                .accept()
                .expect("Couldn't connect");
            // Clear any input from the user -- we don't want to fire old stuff (lol)
            while rx.try_recv().is_ok() {}
            tx.send(Incoming::Connected).unwrap();

            // Begin connection loop
            std::println!("Connected to Mistria! Entering loop...");
            stream.set_nonblocking(true).unwrap();
            let mut buffer = [0; 1024];
            loop {
                // Listen to input from FoM
                match stream.read(&mut buffer) {
                    Ok(bytes_read) => {
                        let message = String::from_utf8(buffer[..bytes_read].to_vec()).unwrap();
                        let message = message.trim_end_matches('\0');
                        match message {
                            "ping" => {}
                            message => {
                                tx.send(Incoming::Message(message.to_string())).unwrap();
                            }
                        }
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::WouldBlock => {}
                        ErrorKind::ConnectionReset => {
                            std::println!("Lost connection with Mistria, bailing!");
                            tx.send(Incoming::Disconnected).unwrap();

                            break;
                        }
                        kind => panic!("Unexpected error: {:?}", kind),
                    },
                }

                // Listen to input from the user
                while let Ok(message) = rx.try_recv() {
                    match message {
                        Outgoing::Message(message) => {
                            stream.write_all(message.as_bytes()).unwrap();
                            // write the null byte...
                            stream.write_all(&[0]).unwrap();
                        }
                        Outgoing::Kill => break,
                    }
                }
            }
        });

        Self {
            server_handle,
            outgoing,
            incoming,
            connected: false,
        }
    }

    /// Sends a message to the TcpServer, crashing if the message fails to send.
    ///
    /// ## Panics
    /// This function will crash on any error from the underlying channel.
    pub fn send_message(&self, msg: String) {
        self.outgoing.send(Outgoing::Message(msg)).unwrap();
    }

    /// Reads a message from the TcpClient.
    pub fn read_messages(&mut self) -> impl Iterator<Item = String> + '_ {
        let connected = &mut self.connected;
        self.incoming.try_iter().filter_map(move |v| match v {
            Incoming::Message(v) => Some(v),
            Incoming::Connected => {
                *connected = true;
                None
            }
            Incoming::Disconnected => {
                *connected = false;
                None
            }
        })
    }

    /// Shuts the server and the handle down.
    pub fn shutdown(self) {
        self.outgoing.send(Outgoing::Kill).unwrap();
        self.server_handle.join().unwrap();
    }

    /// Get a reference to the tcp server's connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
