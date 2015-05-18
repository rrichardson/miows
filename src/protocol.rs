
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use bytes::{MutBuf, Buf};
use mio::{TryRead, Interest};

use reactor_control::ReactorControl;

pub type MsgId = usize;
pub type TimerId = usize;

pub enum TimerCtl {
    Replace(u64),
    Restart,
    RestartAndAdd(u64),
    Clear
}


pub trait Protocol {
    type ConnectionState;

    /// Invoked when new data has arrived
    fn on_data<T : TryRead>(&mut self, io : &mut T,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl) -> Option<Interest>
    {
        None
    }

    /// Invoked after a message produced by the protocol has been successfully sent
    fn on_sent(&mut self, mid : MsgId,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl) -> Option<Interest>
    {
        None
    }

    /// Called on the disconnection of a client
    fn on_disconnect(&mut self, cid : Token,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl)
    {
    }

    /// Callback to handle new inbound connections
    fn on_accept(&mut self, cid : Token, ip : SocketAddr,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl) -> Option<Interest>
    {
        None
    }

    /// Callback to handle new outbound connections
    fn on_connect(&mut self, cid : Token,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl) -> Option<Interest>
    {
        None
    }

    /// Callback for the Reactor to handle timer events
    /// return Some value to re-register the timeout at the new delay,
    /// or none to cancel
    fn on_timer(&mut self, tid : Token,
        conn: &mut Box<Connection<Self>>,
        ctrl : &mut ReactorControl) -> Option<usize>
    {
        None
    }

    /// Called before accepting a connection
    fn on_pre_accept(&mut self, ip : SocketAddr) -> bool {
        true
    }
}
