// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(duration)]

extern crate libc;

use std::io;
use std::ops::Neg;
use std::net::{ToSocketAddrs, SocketAddr};

use utils::{One, NetInt};

mod tcp;
mod udp;
mod socket;
mod ext;
mod utils;

#[cfg(unix)] #[path = "unix/mod.rs"] mod sys;
#[cfg(windows)] #[path = "windows/mod.rs"] mod sys;

pub use tcp::TcpBuilder;
pub use udp::UdpBuilder;
pub use ext::{TcpStreamExt, TcpListenerExt, UdpSocketExt};
pub use ext::{TcpBuilderExt, UdpBuilderExt};

fn one_addr<T: ToSocketAddrs>(tsa: T) -> io::Result<SocketAddr> {
    let mut addrs = try!(tsa.to_socket_addrs());
    let addr = match addrs.next() {
        Some(addr) => addr,
        None => return Err(io::Error::new(io::ErrorKind::Other,
                                          "no socket addresses could be resolved"))
    };
    if addrs.next().is_none() {
        Ok(addr)
    } else {
        Err(io::Error::new(io::ErrorKind::Other,
                           "more than one address resolved"))
    }
}

#[cfg(unix)]
fn cvt<T: One + PartialEq + Neg<Output=T>>(t: T) -> io::Result<T> {
    let one: T = T::one();
    if t == -one {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
#[cfg(windows)]
fn cvt<T: PartialEq + utils::Zero>(t: T) -> io::Result<T> {
    if t == T::zero() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

fn hton<I: NetInt>(i: I) -> I { i.to_be() }

trait AsInner {
    type Inner;
    fn as_inner(&self) -> &Self::Inner;
}

trait FromInner {
    type Inner;
    fn from_inner(inner: Self::Inner) -> Self;
}

trait IntoInner {
    type Inner;
    fn into_inner(self) -> Self::Inner;
}
