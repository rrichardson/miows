use protocol::Protocol;
use mio::tcp::{TcpStream, TcpListener};
use mio::util::{Slab};
use std::net;

trait ReactorHandler<T>
where T : Protocol, <T as Protocol>::Output : Send
    fn on_message(&mut self, msg: <T as Protocol>::Output, ctrl : ReactorControl<T>);
}

struct Connection<P : Protocol> (P, TcpStream);

struct ReactorControl<T, H>
where T : Protocol, <T as Protocol>::Output : Send,
      H : ReactorHandler<<T as Protocol>::Output>
{
    listeners: Slab<(TcpAcceptor, SyncSender<ProtoMsg<<T as Protocol>::Output>>)>,
    conns: Slab<Connection<T>>,
    config: ReactorConfig,
}

impl<T> ReactorControl<T>
where T : Protocol, <T as Protocol>::Output : Send,
      H : ReactorHandler<<T as Protocol>::Output>
{

    pub fn new(cfg: NetEngineConfig) -> ReactorControl<T> {

        ReactorControl {
            listeners: Slab::new_starting_at(Token(0), 255),
            conns: Slab::new_starting_at(Token(256), cfg.max_connections + 256),
            config: cfg
        }
    }

    pub fn connect<'b>(&mut self,
                   addr: &str,
                   port: usize,
                   event_loop: &mut Reactor) -> Result<Token, String>
    {
        Ok(self).map(|s| net::lookup_host(addr).map(|lh| (s, lh)))
            .and_then(|(s,lh)| lh.nth(0).map(|a| (s, a)))
                .ok_or(format!("Failed to find host for {}", addr))
            .map(move |(s,a)| match a {
                V4(sa4) => (s, SockAddrV4::new(sa4.ip(), port)),
                V6(sa6) => (s, SockAddrV6::new(sa6.ip(), port)) })
            .and_then(|(s,ad)| TcpStream::connect(ad).map(|sock| (s, sock)))
            .and_then(|(s,sock)| s.conns.insert((T::new(), sock)).map(|tok| (s, tok, sock)))
            .and_then(|(s,sock)| s.event_loop.register_opt(sock, tok, event::READABLE, event::PollOpt::edge()).
                .map(|_| tok))
            .map_err(|e| format!("Failed to register connection in Slab: {}", e))
    }

        match TcpSocket::v4() {
            Ok(s) => {
                match self.conns.insert(Connection(T::new(), s)) {
                    Ok(tok) => match event_loop.register_opt(&self.conns.get(tok).unwrap().sock, tok, event::READABLE, event::PollOpt::edge()) {
                        Ok(..) => match self.conns.get(tok).unwrap().sock.connect(&SockAddr::InetAddr(ip, port as u16)) {
                            Ok(..) => Ok(tok)
                            Err(e) => Err(format!("Failed to connect to {:?}:{:?}, error: {:?}", hostname, port, e))
                        },
                        Err(e)      => Err(format!("Failed to register with the event loop, error: {:?}", e))
                    },
                    _ => Err(format!("Failed to insert into connection slab"))
                }
            },
            Err(e) => Err(format!("Failed to create new socket, error:{:?}", e))
        }
    }

    pub fn listen<'b>(&mut self,
                      addr: &'b str,
                      port: usize,
                      event_loop: &mut Reactor) -> Result<Receiver<ProtoMsg< <T as Protocol>::Output>>, String>
    {
        let ip = get_host_addresses(addr).unwrap()[0];
        match TcpSocket::v4() {
            Ok(s) => {
                //s.set_tcp_nodelay(true); TODO: re-add to mio
                match s.bind(&SockAddr::InetAddr(ip, port as u16)) {
                Ok(l) => match l.listen(255) {
                    Ok(a) => {
                        let (tx, rx) = sync_channel(self.config.queue_size);
                        match self.listeners.insert((a, tx)) {
                            Ok(token) => {
                                event_loop.register_opt(&self.listeners.get_mut(token).unwrap().0,
                                                        token,
                                                        event::READABLE,
                                                        event::PollOpt::edge()).
                                                            map_err(|e| format!("event registration failed: {:?}", e)).
                                                            map(move |_| rx)
                            },
                            Err(_) => Err(format!("failed to insert into listener slab"))
                        }
                    },
                    Err(e) => {Err(format!("Failed to listen to socket {:?}:{:?}, error:{:?}", addr, port, e)) }
                },
                Err(e) => Err(format!("Failed to bind to {:?}:{:?}, error:{:?}", addr, port, e))
            }},
            Err(e) => Err(format!("Failed to create TCP socket, error:{:?}", e))
        }
    }

    pub fn accept(&mut self, event_loop: &mut Reactor, token: Token, hint: event::ReadHint) {
        let calloc = &self.config.allocator;
        let buf_sz = self.config.read_buf_sz;
        let (ref mut list, ref tx) = *self.listeners.get_mut(token).unwrap();
            match list.accept() {
                Ok(NonBlock::Ready(sock)) => {
                    let buf = new_buf(buf_sz, calloc.clone());
                    match self.conns.insert(Connection::new(sock, tx.clone(), buf)) {
                        Ok(tok) =>  {
                            event_loop.register_opt(&self.conns.get(tok).unwrap().sock,
                                                    tok, event::READABLE | event::HUP,
                                                    event::PollOpt::edge()).unwrap();
                                      debug!("readable accepted socket for token {:?}", tok); }
                        Err(..)  => error!("Failed to insert into Slab")
                    }; },
                e => { error!("Failed to accept socket: {:?}", e);}
            }
        event_loop.reregister(list, token, event::READABLE, event::PollOpt::edge()).unwrap();
    }

    pub fn on_read(&mut self, event_loop: &mut EventLoop<Token, StreamBuf>, token: Token, hint: event::ReadHint) {
        let mut close = false;
        match self.conns.get_mut(token) {
            None    => error!("Got a readable event for token {:?},
                               but it is not present in MioHandler connections", token),
            Some((p, sock)) => {
                p.on_data(sock);

                if hint.contains(event::HUPHINT) {
                    close = true;
                }
                else {
                    c.interest.insert(event::READABLE);
                    event_loop.reregister(&c.sock, token, c.interest, event::PollOpt::edge()).unwrap();
                }
            }
        }

        if close {
            self.conns.remove(token);
        }
    }
}

impl<T> Handler for ReactorControl<T>
where T : Protocol, <T as Protocol>::Output : Send
{
    type Timeout = u64;
    type Message = <T as Protocol>::Output;

    fn readable(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, token: Token, hint: event::ReadHint) {
        debug!("mio_processor::readable top, token: {:?}", token);
        if self.listeners.contains(token) {
            self.accept(event_loop, token, hint);
        } else {
            self.on_read(event_loop, token, hint);
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, token: Token) {
        debug!("mio_processor::writable, token: {:?}", token);
        if let Some(c) = self.conns.get_mut(token) {
            if c.drain_write_queue_to_socket() > 0 {
                    c.interest.insert(event::WRITABLE);
                    event_loop.reregister(&c.sock, token, c.interest, event::PollOpt::edge()).unwrap();
            }
        }
    }


    fn notify(&mut self, event_loop: &mut EventLoop<ReactorControl<T>> , msg: StreamBuf) {
        let tok = msg.1;
        match self.conns.get_mut(tok) {
            Some(c) => {
                c.outbuf.push_back(msg);
                if c.drain_write_queue_to_socket() > 0 {
                    c.interest.insert(event::WRITABLE);
                    event_loop.reregister(&c.sock, tok, c.interest, event::PollOpt::edge()).unwrap();
                }
            },
            None => {}
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, timeout : u64) {
        let (ref mut cb, ref handle) = *self.timeouts.get_mut(tok).unwrap();
        if !(*cb).call_mut((event_loop,)) {
            event_loop.clear_timeout(handle.unwrap());
        }
    }
}

