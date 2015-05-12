use std::collections::VecDeque;
use iobuf::{Iobuf, AROIobuf};
use mio::tcp::TcpStream;
use mio::{Buf, TryWrite, Interest};
use std::marker::PhantomData;

pub struct OutBuf (AROIobuf);

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

pub struct Connection
{
    pub sock: TcpStream,
    pub outbuf: VecDeque<AROIobuf>,
    pub interest: Interest,
}

impl Connection
{
    pub fn new(s: TcpStream) -> Connection {
        Connection {
            sock: s,
            outbuf: VecDeque::new(),
            interest: Interest::hup(),
        }
    }

    pub fn drain_write_queue_to_socket(&mut self) -> usize {
        let mut writable = true;
        while writable && self.outbuf.len() > 0 {
            let (result, sz) = {
                let buf = self.outbuf.front_mut().unwrap(); //shouldn't panic because of len() check
                let sz = buf.len();
                (self.sock.write(&mut OutBuf(*buf)), sz as usize)
            };
            match result {
                Ok(Some(n)) =>
                {
                    debug!("Wrote {:?} out of {:?} bytes to socket", n, sz);
                    if n == sz {
                        self.outbuf.pop_front(); // we have written the contents of this buffer so lets get rid of it
                    }
                },
                Ok(None) => { // this is also very unlikely, we got a writable message, but failed
                    // to write anything at all.
                    debug!("Got Writable event for socket, but failed to write any bytes");
                    writable = false;
                },
                Err(e) => { error!("error writing to socket: {:?}", e); writable = false }
            }
        }
        self.outbuf.len()
    }

    pub fn enqueue(&mut self, buf : &AROIobuf) -> Option<()> {
        Some(self.outbuf.push_back(buf.clone()))
    }
}
