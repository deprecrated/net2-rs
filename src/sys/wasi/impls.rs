use std::os::wasi::io::{AsRawFd, FromRawFd};

use socket::Socket;
use sys::{self, c::Fd};
use {AsInner, FromInner, TcpBuilder, UdpBuilder};

impl FromRawFd for TcpBuilder {
    unsafe fn from_raw_fd(fd: Fd) -> TcpBuilder {
        let sock = sys::Socket::from_inner(fd);
        TcpBuilder::from_inner(Socket::from_inner(sock))
    }
}

impl AsRawFd for TcpBuilder {
    fn as_raw_fd(&self) -> Fd {
        // TODO: this unwrap() is very bad
        self.as_inner().borrow().as_ref().unwrap().as_inner().raw() as Fd
    }
}

impl FromRawFd for UdpBuilder {
    unsafe fn from_raw_fd(fd: Fd) -> UdpBuilder {
        let sock = sys::Socket::from_inner(fd);
        UdpBuilder::from_inner(Socket::from_inner(sock))
    }
}

impl AsRawFd for UdpBuilder {
    fn as_raw_fd(&self) -> Fd {
        // TODO: this unwrap() is very bad
        self.as_inner().borrow().as_ref().unwrap().as_inner().raw() as Fd
    }
}
