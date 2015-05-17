
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use bytes::{MutBuf, Buf};
use mio::TryRead;

use reactor_control::ReactorControl;

pub type MsgId = usize;
pub type TimerId = usize;


pub trait Protocol {
    type Output : Send;
    type ConnectionState;

    fn new() -> Self;

    fn mailbox(&mut self, Self::Output, Self::ConnectionState, &mut ReactorControl) -> Option<Self::ConnectionState>

    /// Invoked when new data has arrived
    fn on_data<T : TryRead>(&mut self, io : &mut T, Self::ConnectionState) -> Option<Message<Self::Output>> {
        None
    }

    /// Invoked after a message produced by the protocol has been successfully sent
    fn on_send(&mut self, cid : Token, mid : MsgId) -> Option<Message<Self::Output>> {
        None
    }

    /// Called on the disconnection of a client
    fn on_disconnect(&mut self, cid : Token) -> Option<Message<Self::Output>> {
        None
    }

    /// Callback to handle new inbound connections
    fn on_accept(&mut self, cid : Token, ip : SocketAddr) -> (Option<Self::ConnectionState>, Option<Message<Self::Output>>) {
        (None, None)
    }

    /// Callback to handle new outbound connections
    fn on_connect(&mut self, cid : Token) -> (Option<Self::ConnectionState>, Option<Message<Self::Output>>) {
        (None, None)
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

