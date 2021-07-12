use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, ToSocketAddrs},
    prelude::v1::*,
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
};

/// A server for creating a TCP connection to a running GameMaker game.
/// We use this in the Bugger-rs project and within Tango to talk to the GM
/// game. This is an entirely sync, thread based Tcp model, not intended to be
/// used in async contexts.
///
/// It is **not** highly performant **or** portable, so please expect
/// to only use this in debugging and developer contexts.
pub struct TcpServer {
    // we keep this guy around for basically no reason.
    #[allow(dead_code)]
    server_handle: JoinHandle<()>,
    tx: Sender<String>,
    kill_signal: Sender<()>,
}

impl TcpServer {
    /// Creates a new tcp server at the given address.
    pub fn new<A: ToSocketAddrs + Send + Sync + 'static>(address: A) -> Self {
        let (tx, rx) = channel::<String>();
        let (kill_signal, kill_rcvr) = channel();

        // Thread for server
        let server_handle = thread::spawn(move || {
            std::println!("Waiting to connect to Mistria...");
            let (mut stream, _) = TcpListener::bind(address)
                .unwrap()
                .accept()
                .expect("Couldn't connect");
            // Clear any input from the user -- we don't want to fire old stuff (lol)
            while rx.try_recv().is_ok() {}

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
                                std::println!("[FoM]: {}", message);
                            }
                        }
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::WouldBlock => {}
                        ErrorKind::ConnectionReset => {
                            std::println!("Lost connection with Mistria, bailing!");
                            std::process::exit(0);
                        }
                        kind => panic!("Unexpected error: {:?}", kind),
                    },
                }

                // Listen to input from the user
                while let Ok(message) = rx.try_recv() {
                    stream.write_all(&message.into_bytes()).unwrap();
                }

                // and finally, check if we should die
                if kill_rcvr.try_recv().is_ok() {
                    break;
                }
            }
        });

        Self {
            server_handle,
            tx,
            kill_signal,
        }
    }

    /// Sends a message to the TcpServer, crashing if the message fails to send.
    ///
    /// ## Panics
    /// This function will crash on any error from the underlying channel.
    pub fn send_message(&mut self, msg: String) {
        self.tx.send(msg).unwrap();
    }

    /// Shuts the server and the handle down.
    pub fn shutdown(self) {
        self.kill_signal.send(()).unwrap();
        self.server_handle.join().unwrap();
    }
}
