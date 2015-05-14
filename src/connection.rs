use std::collections::VecDeque;
use iobuf::{Iobuf, AROIobuf};
use mio::tcp::TcpStream;
use mio::{Buf, TryWrite, Interest};
use std::marker::PhantomData;

pub struct OutBuf (AROIobuf, usize);

impl Buf for OutBuf {
    fn remaining(&self) -> usize {
        self.0.len() as usize
    }

    fn bytes<'b>(&'b self) -> &'b [u8] { unsafe {
        self.0.as_window_slice()
    }}

    fn advance(&mut self, cnt: usize) {
        self.0.advance(cnt as u32).unwrap()
    }
}

enum ConnectionState {
    Ready,
    InProgress
}

pub struct Connection
{
    pub sock: TcpStream,
    pub token: Option<Token>,
    pub outbuf: VecDeque<OutBuf>,
    pub interest: Interest,
    pub state: ConnectionState
}

impl Connection
{
    pub fn new(s: TcpStream) -> Connection {
        Connection {
            sock: s,
            token: None,
            outbuf: VecDeque::new(),
            interest: Interest::hup(),
            state: ConnectionState::InProgress
        }
    }

    pub fn drain_write_queue_to_socket(&mut self) -> usize {
        self.outbuf.len()
    }

    pub fn enqueue(&mut self, buf : &AROIobuf) -> Option<()> {
        Some(self.outbuf.push_back(buf.clone()))
    }
}
