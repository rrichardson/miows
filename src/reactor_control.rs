
use std::io::Error;
use mio::{Evented, Interest, PollOpt};

trait ReactorControl {

    /// Tells the reactor to connect outbound on TCP to the specified host/port
    fn connect(&mut self, hostname: &str, port: usize) -> Result<Token, Error>;

    /// Tells the reactor to bind to the supplied ip-addr and port and listen for connections
    fn listen(&mut self, hostname: &str, port: usize) -> Result<Token, Error>;

    /// Write the message to be written to the connection for token
    /// Takes a message ID which will be passed to the protocol on_sent callback
    /// on the completion of a success write if the message is enqueued
    /// Attempts to immedately write to the socket, if unsuccessful, it will
    /// enqueue the message for later transmission
    /// on immediate write, it returns the number of bytes written
    fn write(&mut self, tok: Token, buf : OutBuf, mid : usize) -> Result<Option<usize>, Error>;

    /// disconnects and de-registers the connection for the supplied token
    /// if the token is a listener, then it ceases accepting new connections
    /// but does not close the existing connections which it accepted
    fn disconnect(&mut self, tok: Token) -> Result<(), Error>;

    /// registers a socket constructed externally to be tracked by Reactor
    fn register<'a>(&mut self, sock: &Evented, interest: Interest, opt: PollOpt) -> Result<Token, Error>;

    /// changes the interest and event-style for a particular connection for the supplied token
    fn interest(&mut self, tok: Token, interest: Interest, opt: PollOpt) -> Result<(), Error>;
}
