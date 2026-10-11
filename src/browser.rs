//! Optional bounded browser HTTPS port over maintained Fetch/Streams bindings.
use crate::{
    Error, Result,
    feed::{Action, Request},
};
use futures_util::{
    StreamExt,
    future::{Either, select},
};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AbortController, ReferrerPolicy, RequestCache, RequestCredentials, RequestInit, RequestMode,
    RequestRedirect, Response,
};

/// Origin-pinned transport for a browser window or isolated wallet iframe.
/// Publishing authority stays in the original Charter, never in Fetch.
pub struct Browser {
    origin: String,
    scope: crate::FeedScope,
}

/// Original bounded response bytes, not a parsed or authenticated publication.
pub struct Received {
    bytes: Vec<u8>,
}
impl Received {
    /// Pass these unchanged to the original Follower/Charter parsers.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

struct AbortOnDrop(AbortController);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl Browser {
    /// Validate the same exact configured HTTPS origin as the native owner port.
    pub fn new(configuration: &cchr::Configuration) -> Result<Self> {
        Self::configured(configuration, crate::FeedScope::Community)
    }
    /// Select only the isolated wallet's fixed public paths. The same original
    /// Charter configuration and browser HTTPS/privacy checks remain required.
    pub fn wallet(configuration: &cchr::Configuration) -> Result<Self> {
        Self::configured(configuration, crate::FeedScope::wallet(configuration)?)
    }
    fn configured(configuration: &cchr::Configuration, scope: crate::FeedScope) -> Result<Self> {
        let origin = cchr::Charter::new(configuration.clone())
            .map_err(Error::Trust)?
            .origin()
            .to_owned();
        Ok(Self { origin, scope })
    }

    fn request(&self, request: Request, abort: &AbortController) -> Result<web_sys::Request> {
        let options = RequestInit::new();
        options.set_method("POST");
        options.set_mode(RequestMode::Cors);
        options.set_credentials(RequestCredentials::Omit);
        options.set_redirect(RequestRedirect::Error);
        options.set_cache(RequestCache::NoStore);
        options.set_referrer_policy(ReferrerPolicy::NoReferrer);
        options.set_referrer("");
        options.set_signal(Some(&abort.signal()));
        options.set_body(&JsValue::from_str(&request.body().to_string()));
        let request = web_sys::Request::new_with_str_and_init(
            &format!("{}{}", self.origin, self.scope.path(request)),
            &options,
        )
        .map_err(|_| Error::Network)?;
        for (name, value) in [
            ("Content-Type", "application/json"),
            ("Accept", "application/json"),
            ("Cache-Control", "no-cache, no-store"),
            ("Pragma", "no-cache"),
        ] {
            request
                .headers()
                .set(name, value)
                .map_err(|_| Error::Network)?;
        }
        Ok(request)
    }

    /// Execute only the original public Fetch/Poll request. No cookies, bearer,
    /// member identifiers, redirects, cache or retries. Dropping the future aborts
    /// Fetch and body streaming; Follower separately fences late operation IDs.
    pub async fn execute(&self, request: Request, now: u64) -> Result<Received> {
        let remaining = request
            .deadline()
            .checked_sub(now)
            .filter(|seconds| (1..=300).contains(seconds))
            .ok_or(Error::Timeout)?;
        let limit = match request.action() {
            Action::Fetch => cchr::MAX_FEED_BYTES,
            Action::Poll { .. } => 1024,
        };
        let abort = AbortOnDrop(AbortController::new().map_err(|_| Error::Network)?);
        let request = self.request(request, &abort.0)?;
        let expected_url = request.url();
        let transfer = async {
            let window = web_sys::window().ok_or(Error::Network)?;
            let response: Response = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|_| Error::Network)?
                .dyn_into()
                .map_err(|_| Error::Response)?;
            if response.url() != expected_url || response.redirected() || !response.ok() {
                return Err(Error::Response);
            }
            collect(response, limit).await
        };
        match select(
            Box::pin(transfer),
            Box::pin(TimeoutFuture::new((remaining * 1000) as u32)),
        )
        .await
        {
            Either::Left((result, _)) => result,
            Either::Right(_) => Err(Error::Timeout),
        }
    }
}

async fn collect(response: Response, limit: usize) -> Result<Received> {
    let content_type = response
        .headers()
        .get("Content-Type")
        .map_err(|_| Error::Response)?;
    if !content_type.as_deref().is_some_and(|value| {
        value
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("application/json")
    }) {
        return Err(Error::Response);
    }
    if let Some(length) = response
        .headers()
        .get("Content-Length")
        .map_err(|_| Error::Response)?
    {
        if length.parse::<u64>().map_err(|_| Error::Response)? > limit as u64 {
            return Err(Error::Response);
        }
    }
    let body = response.body().ok_or(Error::Response)?;
    let mut stream = wasm_streams::ReadableStream::from_raw(body)
        .try_into_stream()
        .map_err(|_| Error::Response)?;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk: js_sys::Uint8Array = chunk
            .map_err(|_| Error::Network)?
            .dyn_into()
            .map_err(|_| Error::Response)?;
        let length = chunk.length() as usize;
        if length > limit.saturating_sub(bytes.len()) {
            return Err(Error::Response);
        }
        let start = bytes.len();
        bytes.resize(start + length, 0);
        chunk.copy_to(&mut bytes[start..]);
    }
    Ok(Received { bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Config, Follower};
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    fn configuration() -> cchr::Configuration {
        let signer =
            csgn::Signer::new("alpha", csgn::SecretKey::from_seed(&mut [71; 32]), 0, 1000).unwrap();
        cchr::Configuration {
            origin: "https://alpha.example.test".into(),
            community: "alpha".into(),
            key_ring: signer.key_ring().to_cbor(),
            minimum_revision: 1,
            ring_revision: 0,
        }
    }
    fn feed_request() -> Request {
        Follower::new(Config {
            retry_entropy: 0,
            refresh_seconds: 40,
            poll_seconds: 2,
            request_seconds: 26,
            initial_backoff_seconds: 2,
            maximum_backoff_seconds: 16,
            maximum_staleness_seconds: 60,
        })
        .unwrap()
        .start(100)
        .unwrap()
    }
    fn response(value: &str) -> Response {
        let response = Response::new_with_opt_str(Some(value)).unwrap();
        response
            .headers()
            .set("Content-Type", "application/json; charset=utf-8")
            .unwrap();
        response
    }

    #[wasm_bindgen_test]
    async fn real_browser_request_omits_credentials_and_abort_cancels_signal() {
        let browser = Browser::new(&configuration()).unwrap();
        let abort = AbortOnDrop(AbortController::new().unwrap());
        let signal = abort.0.signal();
        let request = browser.request(feed_request(), &abort.0).unwrap();
        assert_eq!(request.url(), "https://alpha.example.test/v1/trust_feed");
        assert_eq!(request.credentials(), RequestCredentials::Omit);
        assert_eq!(request.redirect(), RequestRedirect::Error);
        assert_eq!(request.cache(), RequestCache::NoStore);
        assert_eq!(request.referrer_policy(), ReferrerPolicy::NoReferrer);
        assert_eq!(request.referrer(), "");
        assert!(request.headers().get("Authorization").unwrap().is_none());
        assert!(request.headers().get("Cookie").unwrap().is_none());
        assert_eq!(
            JsFuture::from(request.text().unwrap())
                .await
                .unwrap()
                .as_string()
                .unwrap(),
            "{}"
        );
        drop(abort);
        assert!(signal.aborted());
        assert!(matches!(
            browser.execute(feed_request(), 126).await,
            Err(Error::Timeout)
        ));
        let mut invalid = configuration();
        invalid.origin = "http://alpha.example.test".into();
        assert!(Browser::new(&invalid).is_err());
    }

    #[wasm_bindgen_test]
    async fn wallet_browser_keeps_the_original_privacy_and_fixed_path_boundary() {
        let signer =
            csgn::Signer::new("wallet", csgn::SecretKey::from_seed(&mut [72; 32]), 0, 1000)
                .unwrap();
        let configuration = cchr::Configuration {
            origin: "https://wallet.example.test".into(),
            community: "wallet".into(),
            key_ring: signer.key_ring().to_cbor(),
            minimum_revision: 1,
            ring_revision: 0,
        };
        let browser = Browser::wallet(&configuration).unwrap();
        let abort = AbortOnDrop(AbortController::new().unwrap());
        let signal = abort.0.signal();
        let request = browser.request(feed_request(), &abort.0).unwrap();
        assert_eq!(
            request.url(),
            "https://wallet.example.test/v1/wallet_trust_feed"
        );
        assert_eq!(request.credentials(), RequestCredentials::Omit);
        assert_eq!(request.redirect(), RequestRedirect::Error);
        assert_eq!(request.cache(), RequestCache::NoStore);
        assert_eq!(request.referrer_policy(), ReferrerPolicy::NoReferrer);
        assert!(request.headers().get("Authorization").unwrap().is_none());
        assert!(request.headers().get("Cookie").unwrap().is_none());
        assert_eq!(
            JsFuture::from(request.text().unwrap())
                .await
                .unwrap()
                .as_string()
                .unwrap(),
            "{}"
        );
        drop(abort);
        assert!(signal.aborted());
        let wrong_scope = cchr::Configuration {
            community: "alpha".into(),
            ..configuration.clone()
        };
        assert!(matches!(
            Browser::wallet(&wrong_scope),
            Err(Error::Configuration)
        ));
        let wrong_origin = cchr::Configuration {
            origin: "http://localhost".into(),
            ..configuration
        };
        assert!(Browser::wallet(&wrong_origin).is_err());
    }

    #[wasm_bindgen_test]
    async fn actual_browser_stream_preserves_bytes_and_refuses_oversized_body() {
        let original = "{\"revision\":10,\"revision\":11}";
        // Preserve duplicate-key bytes for the original strict owner parser.
        assert_eq!(
            collect(response(original), original.len())
                .await
                .unwrap()
                .bytes(),
            original.as_bytes()
        );
        assert!(matches!(
            collect(response(original), original.len() - 1).await,
            Err(Error::Response)
        ));
        let oversized_header = response("{}");
        oversized_header
            .headers()
            .set("Content-Length", "100")
            .unwrap();
        assert!(matches!(
            collect(oversized_header, 2).await,
            Err(Error::Response)
        ));
        let malformed_header = response("{}");
        malformed_header
            .headers()
            .set("Content-Length", "invalid")
            .unwrap();
        assert!(matches!(
            collect(malformed_header, 1024).await,
            Err(Error::Response)
        ));
        let wrong_type = Response::new_with_opt_str(Some("{}")).unwrap();
        assert!(matches!(
            collect(wrong_type, 1024).await,
            Err(Error::Response)
        ));
    }
}
