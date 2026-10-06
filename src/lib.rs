//! Envoy follows public trust feeds and supplies authenticated bytes to Charter.
#![forbid(unsafe_code)]
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
pub mod browser;
pub mod feed;
#[cfg(all(feature = "http", not(target_arch = "wasm32")))]
pub mod http;
pub use cchr;
pub type Result<T> = std::result::Result<T, Error>;
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
