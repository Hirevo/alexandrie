/// The version of the HTTP protocol in use.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Version {
    /// HTTP/0.9
    Http0_9,

    /// HTTP/1.0
    Http1_0,

    /// HTTP/1.1
    Http1_1,

    /// HTTP/2.0
    Http2_0,

    /// HTTP/3.0
    Http3_0,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ord() {
        use Version::*;
        assert!(Http3_0 > Http2_0);
        assert!(Http2_0 > Http1_1);
        assert!(Http1_1 > Http1_0);
        assert!(Http1_0 > Http0_9);
    }
}
