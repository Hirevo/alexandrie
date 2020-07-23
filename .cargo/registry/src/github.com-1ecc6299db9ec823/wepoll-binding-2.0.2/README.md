# wepoll-binding

Safe Rust bindings for [wepoll][wepoll], using [wepoll-sys][wepoll-sys].

## Requirements

* Rust 2018
* Windows
* clang
* A compiler such as gcc, the MSVC compiler (`cl.exe`), etc

## Usage

Add wepoll-binding as a Windows dependency (since it won't build on other
platforms):

```toml
[dependencies.'cfg(windows)'.dependencies]
wepoll-binding = "^2.0"
```

Next you'll need to create an `Epoll` instance, and register some sockets with
it:

```rust
use wepoll_binding::{Epoll, EventFlag};
use std::net::UdpSocket;

let epoll = Epoll::new().unwrap();
let socket = UdpSocket::new("0.0.0.0:0").unwrap();

epoll.register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42).unwrap();
```

You can poll for events using `Epoll::poll()`. For this you'll need to create an
`Events` buffer:

```rust
use wepoll_binding::{Epoll, EventFlag, Events};
use std::net::UdpSocket;

let epoll = Epoll::new().unwrap();
let socket = UdpSocket::new("0.0.0.0:0").unwrap();
let mut events = Events::with_capacity(1);

epoll.register(&socket, EventFlag::OUT | EventFlag::ONESHOT, 42).unwrap();
epoll.poll(&mut events, None);
```

Note that wepoll (and thus this binding) only support sockets, so you can't use
arbitrary file descriptors.

## License

All source code in this repository is licensed under the Mozilla Public License
version 2.0, unless stated otherwise. A copy of this license can be found in the
file "LICENSE".

[wepoll]: https://github.com/piscisaureus/wepoll
[wepoll-sys]: https://gitlab.com/yorickpeterse/wepoll-sys
