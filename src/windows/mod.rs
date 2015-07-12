// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(bad_style)]

use std::io;
use std::mem;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::os::windows::io::FromRawSocket;
use std::sync::{Once, ONCE_INIT};
use libc::{self, SOCKET, WORD, c_int};

mod impls;

const WSADESCRIPTION_LEN: usize = 256;
const WSASYS_STATUS_LEN: usize = 128;

#[repr(C)]
#[cfg(target_arch = "x86")]
struct WSADATA {
    wVersion: WORD,
    wHighVersion: WORD,
    szDescription: [u8; WSADESCRIPTION_LEN + 1],
    szSystemStatus: [u8; WSASYS_STATUS_LEN + 1],
    iMaxSockets: u16,
    iMaxUdpDg: u16,
    lpVendorInfo: *mut u8,
}
#[repr(C)]
#[cfg(target_arch = "x86_64")]
struct WSADATA {
    wVersion: WORD,
    wHighVersion: WORD,
    iMaxSockets: u16,
    iMaxUdpDg: u16,
    lpVendorInfo: *mut u8,
    szDescription: [u8; WSADESCRIPTION_LEN + 1],
    szSystemStatus: [u8; WSASYS_STATUS_LEN + 1],
}

#[link(name = "ws2_32")]
extern "system" {
    fn WSAStartup(wVersionRequested: WORD,
                  lpWSAData: *mut WSADATA) -> c_int;
    fn WSACleanup() -> c_int;
}

fn init() {
    static INIT: Once = ONCE_INIT;

    INIT.call_once(|| unsafe {
        let mut data: WSADATA = mem::zeroed();
        let ret = WSAStartup(0x202, &mut data);
        assert_eq!(ret, 0);
        libc::atexit(shutdown);
    });

    extern fn shutdown() {
        unsafe { WSACleanup(); }
    }
}

pub struct Socket {
    socket: SOCKET,
}

impl Socket {
    pub fn new(family: c_int, ty: c_int) -> io::Result<Socket> {
        init();
        unsafe {
            // TODO: Call WSASocket to set other fancy things
            let socket = libc::socket(family, ty, 0);
            if socket != !0 {
                Ok(Socket { socket: socket })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn raw(&self) -> SOCKET { self.socket }

    fn into_socket(self) -> SOCKET {
        let socket = self.socket;
        mem::forget(self);
        socket
    }

    pub fn into_tcp_listener(self) -> TcpListener {
        unsafe { TcpListener::from_raw_socket(self.into_socket()) }
    }

    pub fn into_tcp_stream(self) -> TcpStream {
        unsafe { TcpStream::from_raw_socket(self.into_socket()) }
    }

    pub fn into_udp_socket(self) -> UdpSocket {
        unsafe { UdpSocket::from_raw_socket(self.into_socket()) }
    }
}

impl ::FromInner for Socket {
    type Inner = SOCKET;
    fn from_inner(socket: SOCKET) -> Socket {
        Socket { socket: socket }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::closesocket(self.socket);
        }
    }
}
