use std::net::{Ipv4Addr, UdpSocket};

/// The IPv4 address the kernel would use as source for traffic leaving the
/// default route.
///
/// `connect` on a UDP socket is connectionless: it only fixes the peer in the
/// socket, picks a route and binds a local address. No packet is sent, no
/// privileges are needed, and the behaviour is identical on Linux and macOS.
/// Reading the routing table directly, or walking the interface list, would
/// force us to maintain a denylist for `docker0`, `virbr0`, `tailscale0` and
/// friends.
pub fn lan_ip() -> Result<Ipv4Addr, &'static str> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|_| "cannot open a UDP socket")?;
    socket
        .connect((Ipv4Addr::new(1, 1, 1, 1), 80))
        .map_err(|_| "no IPv4 route to the network")?;
    match socket.local_addr().map_err(|_| "no IPv4 route to the network")? {
        std::net::SocketAddr::V4(addr) => Ok(*addr.ip()),
        std::net::SocketAddr::V6(_) => Err("no IPv4 route to the network"),
    }
}
