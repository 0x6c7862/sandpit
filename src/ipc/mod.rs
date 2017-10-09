//! The IPC is a bi-directional communication channel between the broker and sandbox
//!
//! Permitted calls are executed by the broker and results returned to the sandbox
//!
//! The following commands are defined:
//! * ping - Return "pong"
//! * open - Check whether permitted by the policy and return an fd

// NOTE: This is pretty naïve, but it _is_ supposed to be a toy. Maybe don't roll your own and
//       use a real IPC library instead if you are building something important :) It's also
//       poorly abstracted; things are coupled in places where they shouldn't be, there's heaps of
//       code duplication and you keep having to change multiple places to modify behaviour.

mod dsl {
    // NOTE: If you haven't used parser combinators before this part probably looks crazy. Also,
    //       the Rust ones are built on macros and look kind of crazy regardless. They're really
    //       powerful though, and once you get used to how they work they're actually significantly
    //       simpler than any other sort of parsing construct. I would highly recommend checking
    //       them out if you use a language that has sensible library support for them (I know
    //       Haskell/Scala/Rust/etc. do, but I don't know if Python/Ruby/Java/etc. do) and you need
    //       to parse things.
    pub mod request {
        use nom::{IResult, space};
        use std::str;

        pub enum Command<'a> {
            Ping,
            Open(&'a str),
        }

        named!(cstr<&str>, do_parse!(
            s: map_res!(take_until_s!("\0"), str::from_utf8) >>
            tag!("\0") >>
            (s)
        ));

        named!(ping_parser, terminated!(tag!("ping"), tag!("\0")));
        named!(open_parser<&str>,
            do_parse!(
                tag!("open") >>
                space >>
                filename: cstr >>
                (filename)
            )
       );

        named!(parser<&[u8], Command>, alt!(
            map!(complete!(ping_parser), |_| Command::Ping) |
            map!(complete!(open_parser), |filename| Command::Open(filename))
        ));

        pub fn parse(s: &[u8]) -> Option<Command> {
            match parser(s) {
                IResult::Done(_, parsed) => Some(parsed),
                _ => None,
            }
        }
    }

    pub mod response {
        use nom::IResult;

        named!(ping_parser<&[u8]>, terminated!(tag!("pong"), tag!("\0")));

        named!(parser<&[u8], Option<()>>, map!(ping_parser, |_| Some(())));

        pub fn parse(s: &[u8]) -> Option<()> {
            match parser(s) {
                IResult::Done(unparsed, parsed) => {
                    assert_eq!(unparsed, b"", "expected unparsed to be empty");
                    parsed
                },
                _ => None,
            }
        }
    }
}

// FIXME: The error handling here is _terrible_ :/
pub mod server {
    use unix;
    use futures::{Future, Poll};
    use ipc::dsl::request;
    use std::fs::File;
    use std::io;
    use std::os::unix::io::AsRawFd;
    use std::os::unix::net::SocketAddr;
    use std::str;
    use std::thread;
    use std::time::Duration;
    use tokio_core::reactor::Core;
    use tokio_uds::UnixDatagram;

    enum PolicyAction {
        Allow,
        Deny
    }

    // FIXME: This is supposed to be provided by the _caller_
    fn permitted(filename: &str) -> PolicyAction {
        match filename.starts_with("/tmp/sandpit.sandbox") {
            true => PolicyAction::Allow,
            false => PolicyAction::Deny,
        }
    }

    fn ping(socket: &UnixDatagram) -> Result<(), io::Error> {
        match socket.send(b"pong\0") {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn open(socket: &UnixDatagram, filename: &str) -> Result<(), io::Error> {
        use std::os::unix::io::IntoRawFd;
        let socketfd = socket.as_raw_fd();

        // Check if permitted
        match permitted(filename) {
            PolicyAction::Allow => (),
            PolicyAction::Deny => {
                println!("[ipc server] Request failed policy");
                // FIXME: This isn't reporting errors properly
                return match unix::open_sendmsg_err(socketfd) {
                    Some(_) => Ok(()),
                    None => Err(io::Error::new(io::ErrorKind::Other, "Error in sendmsg")),
                };
            }
        }

        // Create an fd to send
        let file = match File::open(filename) {
            Ok(val) => val,
            Err(e) => {
                println!("[ipc server] open request failed: {}", e);
                // FIXME: This isn't reporting errors properly
                return match unix::open_sendmsg_err(socketfd) {
                    Some(_) => Ok(()),
                    None => Err(io::Error::new(io::ErrorKind::Other, "Error in sendmsg")),
                };
            }
        };

        let fd = file.into_raw_fd();
        match unix::open_sendmsg(socketfd, fd) {
            Some(_) => Ok(()),
            None => Err(io::Error::new(io::ErrorKind::Other, "Error in sendmsg")),
        }
    }

    fn handle(message: &[u8], socket: &UnixDatagram) -> Result<(), io::Error> {
        if let Some(command) = request::parse(message) {
            match command {
                request::Command::Ping => ping(socket),
                request::Command::Open(filename) => open(socket, filename),
            }
        } else {
            println!("[ipc server] Received invalid command");
            Ok(())
        }
    }

    struct Server {
        socket: UnixDatagram,
        buf: Vec<u8>,
        // This represents whether there's more left in the socket to read
        state: Option<(usize, SocketAddr)>,
    }

    impl Future for Server {
        type Item = ();
        type Error = io::Error;

        fn poll(&mut self) -> Poll<(), io::Error> {
            // NOTE: Saying I'm handing a message around is a bit misleading, as everything is
            //       stored in a mutable buffer, but it makes it a bit easier to see what's going
            //       on logically
            loop {
                // Check to see if there's a message and handle it
                self.state = match self.state {
                    Some((size, _)) => {
                        // Get message content and handle
                        let message = &self.buf[..size];
                        match handle(message, &self.socket) { _ => () };

                        // Set state to finished
                        None
                    },
                    None => {
                        // Get message and set state to ready
                        let message = try_nb!(self.socket.recv_from(&mut self.buf));
                        Some(message)
                    }
                }
            }
        }
    }

    pub fn start() -> Result<(), io::Error> {
        // Create the event loop
        // FIXME: The caller is supposed to own the event loop
        // FIXME: Handle errors gracefully
        let mut core = match Core::new() {
            Ok(val) => val,
            Err(e) => panic!("[ipc server] Can't create core: {}", e),
        };
        let handle = core.handle();
        let res = UnixDatagram::bind("server", &handle);
        // FIXME: Handle errors gracefully
        let socket = match res {
            Ok(val) => val,
            Err(e) => panic!("[ipc server] bind(): {}", e),
        };

        // Wait for sandbox
        // TODO: Should be its own future
        loop {
            match socket.connect("client") {
                Ok(_) => {
                    println!("[ipc server] Connected to client");
                    break;
                },
                Err(e) => println!("[ipc server] connect(): {}", e),
            }
            thread::sleep(Duration::from_secs(1));
        }

        // Run the server
        core.run(Server {
            socket: socket,
            buf: vec![0; 64],
            state: None,
        })
    }
}

// FIXME: The client is supposed to also be async
// FIXME: The error handling here is _terrible_ :/
pub mod client {
    use unix;
    use ipc::dsl::response;
    use std::os::unix::net::UnixDatagram;
    use std::thread;
    use std::time::Duration;

    pub struct Client {
        socket: UnixDatagram,
        buf: Vec<u8>,
    }

    impl Client {
        // FIXME: Should be a Result
        pub fn ping(&mut self) -> Option<()> {
            // Send ping
            match self.socket.send(b"ping\0") {
                Ok(_) => (),
                Err(e) => {
                    println!("Something went wrong send()ing ping: {}", e);
                    return None;
                },
            };

            // Recv pong
            let size = match self.socket.recv(&mut self.buf) {
                Ok(val) => val,
                Err(e) => {
                    println!("Something went wrong recv()ing pong: {}", e);
                    return None;
                },
            };

            // Parse and return result
            response::parse(&self.buf[..size])
        }

        // FIXME: Should be a Result
        pub fn open(&mut self, filename: &str) -> Option<i32> {
            let msg = format!("open {}\0", filename);
            match self.socket.send(msg.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    println!("Something went wrong send()ing open: {}", e);
                    return None;
                },
            };

            unix::open_recvmsg(&self.socket)
        }
    }

    // FIXME: Should be a Result
    pub fn connect() -> Option<Client> {
        // FIXME: Handle errors gracefully
        let res = UnixDatagram::bind("client");
        let socket = match res {
            Ok(val) => val,
            Err(e) => panic!("[ipc client] bind(): {}", e),
        };

        // Wait for broker
        // TODO: Should be its own future
        loop {
            match socket.connect("server") {
                Ok(_) => {
                    println!("[ipc client] Connected to server");
                    break;
                },
                Err(e) => println!("[ipc client] connect(): {}", e),
            }
            thread::sleep(Duration::from_secs(1));
        }

        Some(Client {
            socket: socket,
            buf: vec![0; 64],
        })
    }
}
