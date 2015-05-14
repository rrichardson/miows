
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use bytes::{MutBuf, Buf};
use mio::TryRead;

pub type ClientId = usize;
pub type MsgId = usize;
pub type TimerId = usize;

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
/// Clear tells the reactor to de-register the timeout
/// under that ID
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
pub enum Message<M : Send> {
    Write(AROIobuf, MsgId),
    Timer(u64, TimerId),
    Clear(TimerId),
    Kill(AROIobuf),
    Out(M),
    Cons(Message<M>, Message<M>)
}

pub trait Protocol {
    type Output : Send;

    fn new() -> Self;

    /// fn to create a message from data to be sent to a client
    /// This arrives when the Mailbox handler invokes the send command
    fn notify(&mut self, data : Self::Output) -> Option<Message<Self::Output>> {
        None
    }

    /// Invoked when new data has arrived
    fn on_data<T : TryRead>(&mut self, io : &mut T) -> Option<Message<Self::Output>> {
        None
    }


    /// Invoked after a message produced by the protocol has been successfully sent
    fn on_send(&mut self, cid : ClientId, mid : MsgId) -> Option<Message<Self::Output>> {
        None
    }

    /// Called on the disconnection of a client
    fn on_disconnect(&mut self, cid : ClientId) -> Option<Message<Self::Output>> {
        None
    }

    /// Callback to handle new inbound connections
    fn on_accept(&mut self, cid : ClientId, ip : SocketAddr) -> Option<Message<Self::Output>> {
        None
    }

    /// Callback to handle new outbound connections
    fn on_connect(&mut self, cid : ClientId) -> Option<Message<Self::Output>> {
        None
    }

    /// Callback for the Reactor to handle timer events
    /// return true to re-register the timeout, or false
    /// to cancel
    fn on_timer(&mut self, id : usize) -> Option<Message<Self::Output>> {
        None
    }

    /// Called before accepting a connection
    fn on_pre_accept(&mut self, ip : SocketAddr) -> bool {
        return true;
    }
}

