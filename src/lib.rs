#![crate_name = "mio"]
#![feature(globs)]
#![feature(phase)]
#![feature(unsafe_destructor)]
// While in active dev
#![allow(dead_code)]

extern crate alloc;
extern crate nix;

#[phase(plugin, link)]
extern crate log;

pub use buf::{
    Buf,
    MutBuf
};
pub use error::{
    MioResult,
    MioError
};
pub use handler::Handler;
pub use io::{
    pipe,
    NonBlock,
    IoReader,
    IoWriter,
    IoAcceptor,
    PipeReader,
    PipeWriter
};
pub use poll::{
    Poll,
    IoEvent,
    IoEventKind
};
pub use reactor::{
    Reactor,
    ReactorResult
};
pub use slab::Slab;
pub use socket::{
    Socket,
    SockAddr,
    TcpSocket,
    TcpAcceptor,
    UnixSocket,
};

pub mod buf;
mod error;
mod handler;
mod io;
mod os;
mod poll;
mod reactor;
mod slab;
mod socket;
mod timer;
