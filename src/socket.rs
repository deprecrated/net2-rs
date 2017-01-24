// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ptr;
use std::fmt;
use std::io;
use std::mem;
use std::net::SocketAddr;
#[cfg(unix)]
use libc::c_int;
#[cfg(windows)]
use winapi::c_int;

use sys;
use sys::c;

pub struct Socket {
    inner: sys::Socket,
}

impl Socket {
    pub fn new(family: c_int, ty: c_int) -> io::Result<Socket> {
        Ok(Socket { inner: try!(sys::Socket::new(family, ty)) })
    }

    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addr, len) = addr2raw(addr);
        unsafe {
            ::cvt(c::bind(self.inner.raw(), addr, len)).map(|_| ())
        }
    }

    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        unsafe {
            ::cvt(c::listen(self.inner.raw(), backlog)).map(|_| ())
        }
    }

    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addr, len) = addr2raw(addr);
        unsafe {
            ::cvt(c::connect(self.inner.raw(), addr, len)).map(|_| ())
        }
    }

    pub fn connect_w_timeout(&self, addr: &SocketAddr, ms: u64) -> io::Result<()> {
        unsafe {
            let mut self_fd_set: c::fd_set = mem::zeroed();
            fd_set(self, &mut self_fd_set);

            #[cfg(unix)]
            let nfds = self.inner.raw() + 1;
            //First argument is ignored in winsock select()
            #[cfg(windows)]
            let nfds = 0;

            #[cfg(windows)]
            let timeout = c::timeval {
                tv_sec: ms as c::c_long / 1000,
                tv_usec: (ms as c::c_long % 1000) * 1000
            };
            #[cfg(windows)]
            let timeout = &timeout as *const c::timeval;
            #[cfg(unix)]
            let mut timeout = c::timeval {
                tv_sec: ms as c::time_t / 1000,
                tv_usec: (ms as c::suseconds_t % 1000) * 1000
            };
            #[cfg(unix)]
            let timeout = &mut timeout as *mut c::timeval;

            try!(::ext::set_nonblocking(self.inner.raw(), true));

            //In most cases connect should return would_block.
            //Unclear if immediate success is actually possible.
            let result = self.connect(addr);
            match result {
                Ok(_) => (),
                Err(error) => {
                    //it is safe as error constructed from last_os_error()
                    if error.raw_os_error().unwrap() != c::CONN_WOULD_BLOCK as i32 {
                        return Err(error);
                    }
                }
            }

            let result = try!(::cvt(c::select(nfds, ptr::null_mut(), &mut self_fd_set, &mut self_fd_set, timeout)));

            try!(::ext::set_nonblocking(self.inner.raw(), false));
            if result == 0 {
                Err(io::Error::new(io::ErrorKind::TimedOut, "Connection timed out"))
            }
            else {
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.raw().fmt(f)
    }
}

impl ::AsInner for Socket {
    type Inner = sys::Socket;
    fn as_inner(&self) -> &sys::Socket { &self.inner }
}

impl ::FromInner for Socket {
    type Inner = sys::Socket;
    fn from_inner(sock: sys::Socket) -> Socket {
        Socket { inner: sock }
    }
}

impl ::IntoInner for Socket {
    type Inner = sys::Socket;
    fn into_inner(self) -> sys::Socket { self.inner }
}

fn addr2raw(addr: &SocketAddr) -> (*const c::sockaddr, c::socklen_t) {
    match *addr {
        SocketAddr::V4(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
        }
        SocketAddr::V6(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
        }
    }
}

#[cfg(unix)]
fn fd_set(socket: &Socket, fd_set: *mut c::fd_set) {
    unsafe { c::FD_SET(socket.inner.raw(), fd_set) };
}

#[cfg(windows)]
fn fd_set(socket: &Socket, fd_set: *mut c::fd_set) {
    let mut fd_set = unsafe { &mut *fd_set };

    for idx in 0..c::FD_SETSIZE {
        if fd_set.fd_array[idx] == 0 {
            fd_set.fd_array[idx] = socket.inner.raw();
            fd_set.fd_count += 1;
            break;
        }
    }
}
