use std::net::Ipv4Addr;
use url::{Host, Url};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unparsable,
    UnsupportedScheme,
}

impl Error {
    pub fn message(&self) -> &'static str {
        match self {
            Error::Unparsable => "not a URL",
            Error::UnsupportedScheme => "only http and https pages can be shared",
        }
    }
}

/// Point a loopback URL at `lan` so a phone on the same network can open it.
///
/// Port, path, query and fragment are preserved. Anything that is not a
/// loopback host is already reachable from the phone and is returned verbatim.
pub fn rewrite(raw: &str, lan: Ipv4Addr) -> Result<String, Error> {
    let mut url = Url::parse(raw).map_err(|_| Error::Unparsable)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::UnsupportedScheme);
    }
    if is_loopback(url.host()) {
        url.set_host(Some(&lan.to_string())).map_err(|_| Error::Unparsable)?;
    }
    Ok(url.into())
}

fn is_loopback(host: Option<Host<&str>>) -> bool {
    match host {
        // `.localhost` is reserved for loopback by RFC 6761.
        Some(Host::Domain(name)) => name == "localhost" || name.ends_with(".localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback() || ip.is_unspecified(),
        Some(Host::Ipv6(ip)) => ip.is_loopback() || ip.is_unspecified(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAN: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 42);

    fn rewritten(raw: &str) -> String {
        rewrite(raw, LAN).unwrap()
    }

    #[test]
    fn rewrites_loopback_hosts() {
        assert_eq!(rewritten("http://localhost:5173/"), "http://192.168.1.42:5173/");
        assert_eq!(rewritten("http://127.0.0.1:3000/"), "http://192.168.1.42:3000/");
        assert_eq!(rewritten("http://127.1.2.3:3000/"), "http://192.168.1.42:3000/");
        assert_eq!(rewritten("http://0.0.0.0:8080/"), "http://192.168.1.42:8080/");
        assert_eq!(rewritten("http://[::1]:8080/"), "http://192.168.1.42:8080/");
        assert_eq!(rewritten("http://app.localhost:8080/"), "http://192.168.1.42:8080/");
    }

    #[test]
    fn preserves_everything_but_the_host() {
        assert_eq!(
            rewritten("http://localhost:5173/a/b?q=1&r=2#frag"),
            "http://192.168.1.42:5173/a/b?q=1&r=2#frag"
        );
    }

    #[test]
    fn leaves_routable_hosts_alone() {
        assert_eq!(rewritten("https://example.com/x"), "https://example.com/x");
        assert_eq!(rewritten("http://192.168.1.7:8080/"), "http://192.168.1.7:8080/");
        // Not loopback despite the prefix.
        assert_eq!(rewritten("http://notlocalhost/"), "http://notlocalhost/");
        assert_eq!(rewritten("http://localhost.example.com/"), "http://localhost.example.com/");
    }

    #[test]
    fn rejects_pages_that_cannot_be_shared() {
        assert_eq!(rewrite("chrome://extensions", LAN), Err(Error::UnsupportedScheme));
        assert_eq!(rewrite("file:///etc/hosts", LAN), Err(Error::UnsupportedScheme));
        assert_eq!(rewrite("about:blank", LAN), Err(Error::UnsupportedScheme));
        assert_eq!(rewrite("not a url", LAN), Err(Error::Unparsable));
    }
}
