
use bytes::{MutBuf};
use mio::TryRead;

type ClientId = usize;
type MsgId = usize;

pub trait Protocol {
    type Output : Send;

    fn new() -> Self;

    /// Static fn to create a message from data to be sent to a client
    fn new_message(data: &[u8]) -> Option<Output>;

    /// Invoked when new data has arrived
    fn on_data<T : TryRead>(&mut self, io : &mut T) -> Option<Output>;

    /// Invoked after a message produced by the protocol has been successfully sent
    fn on_send(&mut self, cid : ClientId, mid : MsgId) -> Option<Output>;

    /// Called on the disconnection of a client
    fn on_disconnect(&mut self, cid : ClientId) -> Option<Output>;

    /// Callback to handle new inbound connections
    fn on_accept(&mut self, cid : ClientId, ip : SocketAddr) -> Option<Output>;

    /// Callback to handle new outbound connections
    fn on_connect(&mut self, cid : ClientId) -> Option<Output>;

    /// Callback for the Reactor to handle timer events
    /// return true to re-register the timeout, or false
    /// to cancel
    fn on_timer(&mut self, id : usize) -> bool -> Option<Output>;

    /// Called before accepting a connection
    fn on_pre_accept(&mut self, ip : SocketAddr) -> bool {
        return true;
    }
}

