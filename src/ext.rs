// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::mem;
use std::net::{TcpStream, TcpListener, UdpSocket, Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use libc::{self, c_int, socklen_t, c_void, c_uint};

use {TcpBuilder, UdpBuilder};

#[cfg(unix)] type Socket = c_int;
#[cfg(unix)] use std::os::unix::prelude::*;
#[cfg(windows)] type Socket = libc::SOCKET;
#[cfg(windows)] use std::os::windows::prelude::*;

#[cfg(target_os = "linux")] const IPV6_MULTICAST_LOOP: c_int = 19;
#[cfg(target_os = "macos")] const IPV6_MULTICAST_LOOP: c_int = 11;
#[cfg(target_os = "windows")] const IPV6_MULTICAST_LOOP: c_int = 11;
#[cfg(target_os = "linux")] const IPV6_V6ONLY: c_int = 26;
#[cfg(target_os = "macos")] const IPV6_V6ONLY: c_int = 27;
#[cfg(target_os = "windows")] const IPV6_V6ONLY: c_int = 27;

#[cfg(windows)]
extern "system" {
    fn WSAIoctl(s: libc::SOCKET,
                dwIoControlCode: libc::DWORD,
                lpvInBuffer: libc::LPVOID,
                cbInBuffer: libc::DWORD,
                lpvOutBuffer: libc::LPVOID,
                cbOutBuffer: libc::DWORD,
                lpcbBytesReturned: libc::LPDWORD,
                lpOverlapped: *mut c_void,
                lpCompletionRoutine: *mut c_void) -> c_int;
}

#[cfg(windows)] const SIO_KEEPALIVE_VALS: libc::DWORD = 0x98000004;
#[cfg(windows)]
struct tcp_keepalive {
    onoff: libc::c_ulong,
    keepalivetime: libc::c_ulong,
    keepaliveinterval: libc::c_ulong,
}

extern "system" {
    fn getsockopt(sockfd: Socket,
                  level: c_int,
                  optname: c_int,
                  optval: *mut c_void,
                  optlen: *mut socklen_t) -> c_int;
}

fn setopt<T: Copy>(sock: Socket, opt: c_int, val: c_int,
                   payload: T) -> io::Result<()> {
    unsafe {
        let payload = &payload as *const T as *const c_void;
        try!(::cvt(libc::setsockopt(sock, opt, val, payload,
                                    mem::size_of::<T>() as socklen_t)));
        Ok(())
    }
}

fn getopt<T: Copy>(sock: Socket, opt: c_int, val: c_int) -> io::Result<T> {
    unsafe {
        let mut slot: T = mem::zeroed();
        let mut len = mem::size_of::<T>() as socklen_t;
        try!(::cvt(getsockopt(sock, opt, val, &mut slot as *mut _ as *mut _,
                              &mut len)));
        assert_eq!(len as usize, mem::size_of::<T>());
        Ok(slot)
    }
}

pub trait TcpStreamExt {
    fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;
    fn nodelay(&self) -> io::Result<bool>;
    fn set_keepalive(&self, keepalive: Option<Duration>) -> io::Result<()>;
    fn keepalive(&self) -> io::Result<Option<Duration>>;
    fn set_read_timeout(&self, val: Option<Duration>) -> io::Result<()>;
    fn read_timeout(&self) -> io::Result<Option<Duration>>;
    fn set_write_timeout(&self, val: Option<Duration>) -> io::Result<()>;
    fn write_timeout(&self) -> io::Result<Option<Duration>>;
    fn set_ttl(&self, ttl: u32) -> io::Result<()>;
    fn ttl(&self) -> io::Result<u32>;
    fn set_only_v6(&self, only_v6: bool) -> io::Result<()>;
    fn only_v6(&self) -> io::Result<bool>;
}

pub trait TcpListenerExt {
    fn set_ttl(&self, ttl: u32) -> io::Result<()>;
    fn ttl(&self) -> io::Result<u32>;
    fn set_only_v6(&self, only_v6: bool) -> io::Result<()>;
    fn only_v6(&self) -> io::Result<bool>;
}

pub trait UdpSocketExt {
    fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;
    fn broadcast(&self) -> io::Result<bool>;
    fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> io::Result<()>;
    fn multicast_loop_v4(&self) -> io::Result<bool>;
    fn set_multicast_ttl_v4(&self, multicast_ttl_v4: u32) -> io::Result<()>;
    fn multicast_ttl_v4(&self) -> io::Result<u32>;
    fn set_multicast_loop_v6(&self, multicast_loop_v6: bool) -> io::Result<()>;
    fn multicast_loop_v6(&self) -> io::Result<bool>;
    fn set_ttl(&self, ttl: u32) -> io::Result<()>;
    fn ttl(&self) -> io::Result<u32>;
    fn set_only_v6(&self, only_v6: bool) -> io::Result<()>;
    fn only_v6(&self) -> io::Result<bool>;
    fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                         -> io::Result<()>;
    fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                         -> io::Result<()>;
    fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                          -> io::Result<()>;
    fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                          -> io::Result<()>;
}

pub trait TcpBuilderExt {
    fn ttl(&self, ttl: u32) -> io::Result<&Self>;
    fn only_v6(&self, only_v6: bool) -> io::Result<&Self>;
}

pub trait UdpBuilderExt {
    fn ttl(&self, ttl: u32) -> io::Result<&Self>;
    fn only_v6(&self, only_v6: bool) -> io::Result<&Self>;
}

trait AsSock {
    fn as_sock(&self) -> Socket;
}

#[cfg(unix)]
impl<T: AsRawFd> AsSock for T {
    fn as_sock(&self) -> Socket { self.as_raw_fd() }
}
#[cfg(windows)]
impl<T: AsRawSocket> AsSock for T {
    fn as_sock(&self) -> Socket { self.as_raw_socket() }
}

impl TcpStreamExt for TcpStream {
    fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_TCP, libc::TCP_NODELAY,
               nodelay as c_int)
    }
    fn nodelay(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_TCP, libc::TCP_NODELAY)
            .map(int2bool)
    }

    #[cfg(unix)]
    fn set_keepalive(&self, keepalive: Option<Duration>) -> io::Result<()> {
        try!(setopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_KEEPALIVE,
                    keepalive.is_some() as c_int));
        if let Some(dur) = keepalive {
            try!(setopt(self.as_sock(), libc::IPPROTO_TCP, libc::TCP_KEEPIDLE,
                        dur.secs() as c_int));
        }
        Ok(())
    }

    #[cfg(unix)]
    fn keepalive(&self) -> io::Result<Option<Duration>> {
        let keepalive = try!(getopt::<c_int>(self.as_sock(), libc::SOL_SOCKET,
                                             libc::SO_KEEPALIVE));
        if keepalive == 0 {
            return Ok(None)
        }
        let secs = try!(getopt::<c_int>(self.as_sock(), libc::IPPROTO_TCP,
                                        libc::TCP_KEEPIDLE));
        Ok(Some(Duration::new(secs as u64, 0)))
    }

    #[cfg(windows)]
    fn set_keepalive(&self, keepalive: Option<Duration>) -> io::Result<()> {
        let ms = dur2timeout(keepalive);
        let ka = tcp_keepalive {
            onoff: keepalive.is_some() as libc::c_ulong,
            keepalivetime: ms as libc::c_ulong,
            keepaliveinterval: ms as libc::c_ulong,
        };
        unsafe {
            ::cvt(WSAIoctl(self.as_sock(),
                          SIO_KEEPALIVE_VALS,
                          &ka as *const _ as *mut _,
                          mem::size_of_val(&ka) as libc::DWORD,
                          0 as *mut _,
                          0,
                          0 as *mut _,
                          0 as *mut _,
                          0 as *mut _)).map(|_| ())
        }
    }

    #[cfg(windows)]
    fn keepalive(&self) -> io::Result<Option<Duration>> {
        let mut ka = tcp_keepalive {
            onoff: 0,
            keepalivetime: 0,
            keepaliveinterval: 0,
        };
        unsafe {
            try!(::cvt(WSAIoctl(self.as_sock(),
                                SIO_KEEPALIVE_VALS,
                                0 as *mut _,
                                0,
                                &mut ka as *mut _ as *mut _,
                                mem::size_of_val(&ka) as libc::DWORD,
                                0 as *mut _,
                                0 as *mut _,
                                0 as *mut _)));
        }
        Ok({
            if ka.onoff == 0 {
                None
            } else {
                timeout2dur(ka.keepaliveinterval as libc::DWORD)
            }
        })
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        setopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_RCVTIMEO,
               dur2timeout(dur))
    }

    fn read_timeout(&self) -> io::Result<Option<Duration>> {
        getopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_RCVTIMEO)
            .map(timeout2dur)
    }

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        setopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_SNDTIMEO,
               dur2timeout(dur))
    }

    fn write_timeout(&self) -> io::Result<Option<Duration>> {
        getopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_SNDTIMEO)
            .map(timeout2dur)
    }

    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL, ttl as c_int)
    }

    fn ttl(&self) -> io::Result<u32> {
        getopt::<c_int>(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL)
            .map(|b| b as u32)
    }

    fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY, only_v6 as c_int)
    }

    fn only_v6(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY).map(int2bool)
    }
}

#[cfg(unix)]
fn dur2timeout(dur: Option<Duration>) -> libc::timeval {
    // TODO: be more rigorous
    match dur {
        Some(d) => libc::timeval { tv_sec: d.secs() as libc::time_t, tv_usec: 0 },
        None => libc::timeval { tv_sec: 0, tv_usec: 0 },
    }
}

#[cfg(unix)]
fn timeout2dur(dur: libc::timeval) -> Option<Duration> {
    if dur.tv_sec == 0 && dur.tv_usec == 0 {
        None
    } else {
        Some(Duration::new(dur.tv_sec as u64, 0))
    }
}

#[cfg(windows)]
fn dur2timeout(dur: Option<Duration>) -> libc::DWORD {
    // TODO: be more rigorous
    match dur {
        Some(d) => (d.secs() * 1000) as libc::DWORD,
        None => 0,
    }
}

#[cfg(windows)]
fn timeout2dur(dur: libc::DWORD) -> Option<Duration> {
    if dur == 0 {
        None
    } else {
        Some(Duration::new((dur / 1000) as u64, 0))
    }
}

fn int2bool(n: c_int) -> bool {
    if n == 0 {false} else {true}
}

impl UdpSocketExt for UdpSocket {
    fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_BROADCAST,
               broadcast as c_int)
    }
    fn broadcast(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::SOL_SOCKET, libc::SO_BROADCAST)
            .map(int2bool)
    }
    fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_MULTICAST_LOOP,
               multicast_loop_v4 as c_int)
    }
    fn multicast_loop_v4(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_MULTICAST_LOOP)
            .map(int2bool)
    }
    fn set_multicast_ttl_v4(&self, multicast_ttl_v4: u32) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_MULTICAST_TTL,
               multicast_ttl_v4 as c_int)
    }
    fn multicast_ttl_v4(&self) -> io::Result<u32> {
        getopt::<c_int>(self.as_sock(), libc::IPPROTO_IP, libc::IP_MULTICAST_TTL)
            .map(|b| b as u32)
    }
    fn set_multicast_loop_v6(&self, multicast_loop_v6: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_MULTICAST_LOOP,
               multicast_loop_v6 as c_int)
    }
    fn multicast_loop_v6(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_MULTICAST_LOOP)
            .map(int2bool)
    }

    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL, ttl as c_int)
    }

    fn ttl(&self) -> io::Result<u32> {
        getopt::<c_int>(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL)
            .map(|b| b as u32)
    }

    fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY, only_v6 as c_int)
    }

    fn only_v6(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY).map(int2bool)
    }

    fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                         -> io::Result<()> {
        let mreq = libc::ip_mreq {
            imr_multiaddr: ip2in_addr(multiaddr),
            imr_interface: ip2in_addr(interface),
        };
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_ADD_MEMBERSHIP, mreq)
    }

    fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                         -> io::Result<()> {
        let mreq = libc::ip6_mreq {
            ipv6mr_multiaddr: ip2in6_addr(multiaddr),
            ipv6mr_interface: interface as c_uint,
        };
        setopt(self.as_sock(), libc::IPPROTO_IPV6, libc::IPV6_ADD_MEMBERSHIP,
               mreq)
    }

    fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr)
                          -> io::Result<()> {
        let mreq = libc::ip_mreq {
            imr_multiaddr: ip2in_addr(multiaddr),
            imr_interface: ip2in_addr(interface),
        };
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_DROP_MEMBERSHIP, mreq)
    }

    fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32)
                          -> io::Result<()> {
        let mreq = libc::ip6_mreq {
            ipv6mr_multiaddr: ip2in6_addr(multiaddr),
            ipv6mr_interface: interface as c_uint,
        };
        setopt(self.as_sock(), libc::IPPROTO_IPV6, libc::IPV6_DROP_MEMBERSHIP,
               mreq)
    }
}

fn ip2in_addr(ip: &Ipv4Addr) -> libc::in_addr {
    let oct = ip.octets();
    libc::in_addr {
        s_addr: ::hton(((oct[0] as u32) << 24) |
                       ((oct[1] as u32) << 16) |
                       ((oct[2] as u32) <<  8) |
                       ((oct[3] as u32) <<  0)),
    }
}

fn ip2in6_addr(ip: &Ipv6Addr) -> libc::in6_addr {
    let seg = ip.segments();
    libc::in6_addr {
        s6_addr: [
            ::hton(seg[0]),
            ::hton(seg[1]),
            ::hton(seg[2]),
            ::hton(seg[3]),
            ::hton(seg[4]),
            ::hton(seg[5]),
            ::hton(seg[6]),
            ::hton(seg[7]),
        ],
    }
}

impl TcpListenerExt for TcpListener {
    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL, ttl as c_int)
    }

    fn ttl(&self) -> io::Result<u32> {
        getopt::<c_int>(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL)
            .map(|b| b as u32)
    }

    fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY, only_v6 as c_int)
    }

    fn only_v6(&self) -> io::Result<bool> {
        getopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY).map(int2bool)
    }
}

impl TcpBuilderExt for TcpBuilder {
    fn ttl(&self, ttl: u32) -> io::Result<&Self> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL, ttl as c_int)
            .map(|()| self)
    }

    fn only_v6(&self, only_v6: bool) -> io::Result<&Self> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY, only_v6 as c_int)
            .map(|()| self)
    }
}

impl UdpBuilderExt for UdpBuilder {
    fn ttl(&self, ttl: u32) -> io::Result<&Self> {
        setopt(self.as_sock(), libc::IPPROTO_IP, libc::IP_TTL, ttl as c_int)
            .map(|()| self)
    }

    fn only_v6(&self, only_v6: bool) -> io::Result<&Self> {
        setopt(self.as_sock(), libc::IPPROTO_IPV6, IPV6_V6ONLY, only_v6 as c_int)
            .map(|()| self)
    }
}
