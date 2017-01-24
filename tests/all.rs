extern crate net2;

use std::net::TcpStream;
use std::io::prelude::*;
use std::thread;
use std::time;

use net2::TcpBuilder;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with: {}", stringify!($e), e),
    })
}

#[test]
fn smoke_build_listener() {
    let b = t!(TcpBuilder::new_v4());
    t!(b.bind("127.0.0.1:0"));
    let listener = t!(b.listen(200));

    let addr = t!(listener.local_addr());

    let t = thread::spawn(move || {
        let mut s = t!(listener.accept()).0;
        let mut b = [0; 4];
        t!(s.read(&mut b));
        assert_eq!(b, [1, 2, 3, 0]);
    });

    let mut stream = t!(TcpStream::connect(&addr));
    t!(stream.write(&[1,2,3]));
    t.join().unwrap();
}

#[test]
fn tcp_connect_w_timeout_fail() {
    let timeout = 100;

    let client = t!(TcpBuilder::new_v4());

    let now = time::Instant::now();
    let result = client.connect_w_timeout("192.192.192.192:1", timeout);
    let elapsed = now.elapsed();

    assert!(result.is_err());
    let result = result.err().unwrap();
    assert_eq!(result.kind(), std::io::ErrorKind::TimedOut);
    let elapsed = elapsed.as_secs() / 1000 + elapsed.subsec_nanos() as u64 / 1000000;
    assert!(elapsed >= (timeout - 1) || elapsed <= (timeout + 1));
}

#[test]
fn tcp_connect_w_timeout_success() {
    let timeout = 0;

    let server = t!(TcpBuilder::new_v4());
    t!(server.bind("127.0.0.1:0"));
    let server = t!(server.listen(500));

    let addr = t!(server.local_addr());

    let t = thread::spawn(move || {
        let mut s = t!(server.accept()).0;
        let mut b = [0; 4];
        t!(s.read(&mut b));
        assert_eq!(b, [1, 2, 3, 0]);
    });

    let client = t!(TcpBuilder::new_v4());

    let mut stream = t!(client.connect_w_timeout(&addr, timeout));
    t!(stream.write(&[1,2,3]));
    t.join().unwrap();

}
