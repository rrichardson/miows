use std::net;
use iobuf::AROIobuf;
use mio::tcp::{TcpStream, TcpListener};
use mio::util::{Slab};

use protocol::Protocol;
use connection::Connection;

type TaggedBuf = (Token, AROIobuf);

trait ReactorHandler<T>
where T : Send
    fn on_message(&mut self, msg: T, ctrl : &mut ReactorControl<T, Self>);
}

struct ReactorControl<T, H>
where T : Protocol, <T as Protocol>::Output : Send,
      H : ReactorHandler<<T as Protocol>::Output>
{
    listeners: Slab<(TcpAcceptor, SyncSender<ProtoMsg<<T as Protocol>::Output>>)>,
    conns: Slab<Protocol<T>, Connection>,
    config: ReactorConfig,
    handler: H
}

impl<T> ReactorControl<T>
where T : Protocol, <T as Protocol>::Output : Send,
      H : ReactorHandler<<T as Protocol>::Output>
{

    pub fn new(handler: H, cfg: ReactorConfig) -> ReactorControl<T> {
        let num_listeners = 255;
        let conn_slots = cfg.max_connections + num_listeners + 1;
        let timer_slots = (conn_slots * cfg.timers_per_connection)

        ReactorControl {
            listeners: Slab::new_starting_at(Token(0), 255),
            conns: Slab::new_starting_at(Token(num_listeners + 1), conn_slots),
            timers: Slab::new_starting_at(Token(0), timer_slots),
            handler: handler,
            config: cfg
        }
    }

    pub fn connect<'b>(&mut self,
                   addr: &'b str,
                   port: usize,
                   event_loop: &mut EventLoop<ReactorControl<T>>) -> Result<Token, String>
    {
        let saddr = try!(net::lookup_host(addr).and_then(|lh| lh.nth(0))
                        .ok_or(format!("Failed to find host for {}", addr))
                        .map(move |sa| match sa {
                            V4(sa4) => (s, SockAddrV4::new(sa4.ip(), port)),
                            V6(sa6) => (s, SockAddrV6::new(sa6.ip(), port)) }));
        let sock = try!(TcpStream::connect(saddr));
        let tok = try!(self.conns.insert((T::new(), sock)));
        try!(event_loop.register_opt(self.cons.get_mut(tok).unwrap().1,
                                     tok, event::READABLE, event::PollOpt::edge()));
        if let Some(msg) = proto.on_connect(tok) {
            self.handler.on_message(msg, self);
        }
        Ok(tok)
    }

    pub fn listen<'b>(&mut self,
                      addr: &'b str,
                      port: usize,
                      event_loop: &mut EventLoop<ReactorControl<T>>) -> Result<Token, String>
    {
        let saddr : SocketAddr = addr.parse();
        let server = try!(TcpListener::bind(&saddr));
        let accpt = try!(server.listen(255));
        let tok = try!(self.listeners.insert(accpt)));
        try!(event_loop.register_opt(&self.listeners.get_mut(token).unwrap(),
                                     token, event::READABLE, event::PollOpt::edge()));
        Ok(tok)
    }

    pub fn accept(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, token: Token, hint: event::ReadHint) {

        let (ref mut accpt, ref tx) = *self.listeners.get_mut(token).unwrap();
        let buf_sz = self.config.read_buf_sz;

        if let Some(sock) = try!(accpt.accept()) {
            let tok = try!(self.conns.insert((P::new(), sock)));
            try!(event_loop.register_opt(&self.conns.get(tok).unwrap().1,
                                         tok, event::READABLE | event::HUP,
                                          event::PollOpt::edge()).unwrap());
            let (ref mut proto, ref sockref) = self.conns.get_mut(tok).unwrap();
            let peeraddr = try!(sockref.peer_addr());
            if let Some(msg) = proto.on_accept(tok.0, peeraddr) {
                self.handler.on_message(msg, self);
            }
        }

        event_loop.reregister(list, token, event::READABLE, event::PollOpt::edge()).unwrap();
    }

    pub fn on_read(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, token: Token, hint: event::ReadHint) {

        let mut close = false;
        let (ref mut proto, ref sock) = try!(self.conns.get_mut(token));
        if let Some(msg) = proto.on_data(sock) {
            self.handler.on_message(msg, self);
        }

        if hint.contains(event::HUPHINT) {
            close = true;
        }
        else {
            c.interest.insert(event::READABLE);
            event_loop.reregister(&c.sock, token, c.interest, event::PollOpt::edge()).unwrap();
        }

        if close {
            self.conns.remove(token);
            if let Some(msg) = proto.on_disconnect(tok.0) {
                self.handler.on_message(msg, self);
            }
        }
    }
}

impl<T> Handler for ReactorControl<T>
where T : Protocol, <T as Protocol>::Output : Send
{
    type Timeout = u64;
    type Message = TaggedBuf;

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


    fn notify(&mut self, event_loop: &mut EventLoop<ReactorControl<T>>, msg: TaggedBuf) {
        let tok = msg.1;
        match self.conns.get_mut(tok) {
            Some(c) => {
                c.enqueue(msg);
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

