
struct Connection<T>
where T : Protocol, <T as Protocol>::Output : Send
{
        sock: TcpSocket,
        outbuf: DList<StreamBuf>,
        interest: event::Interest,
        conn_tx: SyncSender<ProtoMsg<<T as Protocol>::Output>>,
        marker: i32,
        proto: T,
        buf: AppendBuf
}

impl<T> Connection<T>
where T : Protocol, <T as Protocol>::Output : Send
{
    pub fn new(s: TcpSocket, tx: SyncSender<ProtoMsg<<T as Protocol>::Output>>) -> Connection<T> {
        Connection {
            sock: s,
            outbuf: DList::new(),
            interest: event::HUP,
            conn_tx: tx,
            marker: 0,
            proto: <T as Protocol>::new(),
        }
    }

    fn drain_write_queue_to_socket(&mut self) -> usize {
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

    fn read(&mut self) -> MioResult<NonBlock<usize>> {
        self.sock.read(proto)
    }
}
