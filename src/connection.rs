use std::collections::VecDeque;
use std::iobuf::AROIobuf;
use mio::tcp::TcpStream;

struct Connection
{
        sock: TcpStream,
        outbuf: VecDeque<<AROIobuf>,
        interest: event::Interest,
}

impl Connection
{
    pub fn new(s: TcpStream) -> Connection {
        Connection {
            sock: s,
            outbuf: VecDeque::new(),
            interest: event::HUP,
        }
    }

    pub fn drain_write_queue_to_socket(&mut self) -> usize {
        let mut writable = true;
        while writable && self.outbuf.len() > 0 {
            let (result, sz) = {
                let buf = self.outbuf.front_mut().unwrap(); //shouldn't panic because of len() check
                let sz = buf.0.len();
                (self.sock.write(buf), sz as usize)
            };
            match result {
                Ok(NonBlock::Ready(n)) =>
                {
                    debug!("Wrote {:?} out of {:?} bytes to socket", n, sz);
                    if n == sz {
                        self.outbuf.pop_front(); // we have written the contents of this buffer so lets get rid of it
                    }
                },
                Ok(NonBlock::WouldBlock) => { // this is also very unlikely, we got a writable message, but failed
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
        outbuf.push_back(buf.clone())
    }
}
