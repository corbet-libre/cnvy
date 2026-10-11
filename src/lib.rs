//! Envoy follows public trust feeds and supplies authenticated bytes to Charter.
#![forbid(unsafe_code)]
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
pub mod browser;
pub mod feed;
#[cfg(all(feature = "http", not(target_arch = "wasm32")))]
pub mod http;
pub use cchr;
pub type Result<T> = std::result::Result<T, Error>;

// The configured owner selects a fixed public feed pair. It cannot supply a
// request-controlled URL, replace the follower, or install publishing authority.
#[cfg(any(
    all(feature = "http", not(target_arch = "wasm32")),
    all(feature = "browser", target_arch = "wasm32")
))]
#[derive(Clone, Copy)]
enum FeedScope {
    Community,
    Wallet,
}
#[cfg(any(
    all(feature = "http", not(target_arch = "wasm32")),
    all(feature = "browser", target_arch = "wasm32")
))]
impl FeedScope {
    fn wallet(configuration: &cchr::Configuration) -> Result<Self> {
        if configuration.community != "wallet" {
            return Err(Error::Configuration);
        }
        Ok(Self::Wallet)
    }
    fn path(self, request: feed::Request) -> &'static str {
        match (self, request.action()) {
            (Self::Community, _) => request.path(),
            (Self::Wallet, feed::Action::Fetch) => "/v1/wallet_trust_feed",
            (Self::Wallet, feed::Action::Poll { .. }) => "/v1/wallet_trust_changes",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Configuration,
    Busy,
    Stopped,
    Obsolete,
    Timeout,
    Response,
    Network,
    Exhausted,
    ClockRegression,
    Unavailable,
    BehindAnnouncement,
    Trust(cchr::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
