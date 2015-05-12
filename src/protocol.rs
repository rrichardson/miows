
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use bytes::{MutBuf, Buf};
use mio::TryRead;

pub type ClientId = usize;
pub type MsgId = usize;

/// This enum contains the high level constructs
/// this protocol will use to speak with the
/// reactor, or to its message handler,
/// or to another layer up the stack.
///
/// Send instructs the reactor to enqueue a message
/// to the connection associated with this protocol
/// instance.
///
/// Timer instructs the reactor to set a timer for usize
/// milliseconds from now, notifying the protocol at
/// that occurence with the supplied msg id
///
/// Kill tells the reactor to send a message to the
/// endpoint, then close the tcp session, it will
/// then supply the msg at the callback on_disconnect
///
/// Out sends a Protocol::Output message to the
/// app's main message handler
///
/// Up sends a message directed towards a higher
/// level protocol handler in the stack
pub enum Message<T : Buf, M : Send> {
    Write(T, MsgId),
    Timer(u64, MsgId),
    Clear(u64),
    Kill(T, MsgId),
    Out(M),
    Up(M)
}

pub trait Protocol {
    type Output : Send;

    fn new() -> Self;

    /// Static fn to create a message from data to be sent to a client
    fn new_message(data: &[u8]) -> Option<Self::Output>;

    /// Invoked when new data has arrived
    fn on_data<T : TryRead>(&mut self, io : &mut T) -> Option<Self::Output>;

    /// Invoked after a message produced by the protocol has been successfully sent
    fn on_send(&mut self, cid : ClientId, mid : MsgId) -> Option<Self::Output>;

    /// Called on the disconnection of a client
    fn on_disconnect(&mut self, cid : ClientId) -> Option<Self::Output>;

    /// Callback to handle new inbound connections
    fn on_accept(&mut self, cid : ClientId, ip : SocketAddr) -> Option<Self::Output>;

    /// Callback to handle new outbound connections
    fn on_connect(&mut self, cid : ClientId) -> Option<Self::Output>;

    /// Callback for the Reactor to handle timer events
    /// return true to re-register the timeout, or false
    /// to cancel
    fn on_timer(&mut self, id : usize) -> Option<Self::Output>;

    /// Called before accepting a connection
    fn on_pre_accept(&mut self, ip : SocketAddr) -> bool {
        return true;
    }
}

