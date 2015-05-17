use std::collections::VecDeque;
use iobuf::{Iobuf, AROIobuf};
use mio::tcp::TcpStream;
use mio::{Buf, Token, TryWrite, Interest};
use std::marker::PhantomData;

use protocol::Protocol;

pub struct OutBuf (Token, AROIobuf);

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

enum ConnectionStatus {
    Ready,
    InProgress
}

pub struct Connection<P : Protocol>
{
    pub sock: TcpStream,
    pub token: Option<Token>,
    pub outbuf: VecDeque<OutBuf>,
    pub interest: Interest,
    pub status: ConnectionStatus,
    pub proto: P,
    pub state: Option<<P as Protocol>::ConnectionState>,
}

impl<P : Protocol> Connection<P>
{
    pub fn new(s: TcpStream) -> Connection {
        Connection {
            sock: s,
            token: None,
            outbuf: VecDeque::new(),
            interest: Interest::hup(),
            status: ConnectionStatus::InProgress,
            proto: P::new(),
            state: None
        }
    }

    pub fn drain_write_queue_to_socket(&mut self) -> usize {
        self.outbuf.len()
    }

    pub fn enqueue(&mut self, buf : &AROIobuf) -> Option<()> {
        Some(self.outbuf.push_back(buf.clone()))
    }
}
