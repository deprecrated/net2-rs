use std::os::wasi::io::{FromRawFd, AsRawFd};

use {TcpBuilder, UdpBuilder, FromInner, AsInner};
use socket::Socket;
use sys;

impl FromRawFd for TcpBuilder {
    unsafe fn from_raw_fd(fd: wasi::Fd) -> TcpBuilder {
        let sock = sys::Socket::from_inner(fd);
        TcpBuilder::from_inner(Socket::from_inner(sock))
    }
}

impl AsRawFd for TcpBuilder {
    fn as_raw_fd(&self) -> wasi::Fd {
        // TODO: this unwrap() is very bad
        self.as_inner().borrow().as_ref().unwrap().as_inner().raw() as wasi::Fd
    }
}

impl FromRawFd for UdpBuilder {
    unsafe fn from_raw_fd(fd: wasi::Fd) -> UdpBuilder {
        let sock = sys::Socket::from_inner(fd);
        UdpBuilder::from_inner(Socket::from_inner(sock))
    }
}

impl AsRawFd for UdpBuilder {
    fn as_raw_fd(&self) -> wasi::Fd {
        // TODO: this unwrap() is very bad
        self.as_inner().borrow().as_ref().unwrap().as_inner().raw() as wasi::Fd
    }
}
