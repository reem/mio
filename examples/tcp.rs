//! Listens on a TCP socket and prints all incoming data as soon as it is ready.

extern crate mio;

use mio::{EventLoop, MioResult, Handler, Token, NonBlock,
          IoReader, Buf, IoAcceptor, IoWriter};
use mio::net::SockAddr;
use mio::net::tcp::{TcpSocket, TcpAcceptor};
use mio::buf::ByteBuf;
use mio::util::Slab;
use mio::event;

use std::str;

const SERVER: Token = Token(0);

const RESPONSE: &'static str = "200 OK\r
Content-Length: 11\r
\r
Hello World\r
\r";

struct TcpConn {
    sock: TcpSocket,
    buf: ByteBuf,
    token: Token,
    interest: event::Interest
}

impl TcpConn {
    fn new(sock: TcpSocket) -> TcpConn {
        TcpConn {
            sock: sock,
            buf: ByteBuf::new(RESPONSE.len()),
            // Fake token
            token: Token(-1),
            interest: event::HUP
        }
    }
}

struct TcpHandler {
    acceptor: TcpAcceptor,
    conns: Slab<TcpConn>
}

impl TcpHandler {
    fn new(acceptor: TcpAcceptor) -> TcpHandler {
        TcpHandler {
            acceptor: acceptor,
            conns: Slab::new_starting_at(Token(1), 128)
        }
    }

    fn accept(&mut self, event_loop: &mut EventLoop<uint, ()>) -> MioResult<()> {
        let sock = self.acceptor.accept().unwrap().unwrap();
        let conn = TcpConn::new(sock);
        let tok = self.conns.insert(conn).ok().expect("could not add connection");

        // Correctly set the connection's token.
        self.conns[tok].token = tok;

        // Register our interest in this connection.
        event_loop.register_opt(&self.conns[tok].sock, tok,
                                event::READABLE | event::WRITABLE,
                                event::PollOpt::edge()).unwrap();

        Ok(())
    }

    fn connection_readable(&mut self, event_loop: &mut EventLoop<uint, ()>,
                           token: Token) -> MioResult<()> {
        println!("Connection readable.");
        let connection = &mut self.conns[token];

        // Read bytes into our buffer.
        match connection.sock.read(&mut connection.buf) {
            Ok(NonBlock::WouldBlock) => panic!("Received incorrect readable hint."),
            Ok(NonBlock::Ready(r)) => {
                println!("Read {} bytes.", r);
            },
            Err(e) => panic!("Error while reading: {}", e)
        }

        connection.buf.flip();
        println!("{}", str::from_utf8(connection.buf.bytes()).unwrap());

        event_loop.reregister(&connection.sock, connection.token,
                              connection.interest, event::PollOpt::edge())
    }

    fn connection_writable(&mut self, event_loop: &mut EventLoop<uint, ()>,
                           token: Token) -> MioResult<()> {
        println!("Connection writable.");
        let connection = &mut self.conns[token];

        connection.buf.clear();
        connection.buf.write(RESPONSE.as_bytes()).unwrap();

        match connection.sock.write(&mut connection.buf) {
            Ok(NonBlock::WouldBlock) => panic!("Received incorrect writable hint."),

            Ok(NonBlock::Ready(r)) => {
                println!("Wrote {} bytes.", r);
                if connection.buf.remaining() == 0 {
                    connection.interest.remove(event::WRITABLE);
                }
            },

            Err(e) => panic!("Error while reading: {}", e)
        }

        event_loop.reregister(&connection.sock, connection.token,
                              connection.interest, event::PollOpt::edge())
    }
}

impl Handler<uint, ()> for TcpHandler {
    fn readable(&mut self, event_loop: &mut EventLoop<uint, ()>, token: Token, _: event::ReadHint) {
        match token {
            // Got a new connection.
            SERVER => self.accept(event_loop).unwrap(),

            // Can read on an open connection.
            i => self.connection_readable(event_loop, i).unwrap()
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<uint, ()>, token: Token) {
        self.connection_writable(event_loop, token).unwrap()
    }
}

fn main() {
    // Start the event loop.
    let mut event_loop = EventLoop::new().unwrap();

    let addr = SockAddr::parse("127.0.0.1:3000")
        .expect("could not parse InetAddr");

    // Open socket
    let srv = TcpSocket::v4().unwrap();

    // Bind the socket to an address.
    let srv = srv.bind(&addr).unwrap()
        .listen(256u).unwrap();

    // Register our interest in the socket.
    event_loop
        .register_opt(&srv, SERVER, event::READABLE,
                      event::PollOpt::edge()).unwrap();

    // Start the event loop
    event_loop.run(TcpHandler::new(srv))
        .ok().expect("failed to execute event loop");
}

