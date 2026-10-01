mod support;
use cnvy::{
    Error,
    feed::{Action, Config, State},
};
use support::Driver as Assurance;
use support::*;
fn config() -> Config {
    Config {
        refresh_seconds: 40,
        poll_seconds: 2,
        request_seconds: 26,
        initial_backoff_seconds: 2,
        maximum_backoff_seconds: 16,
        retry_entropy: 0,
        maximum_staleness_seconds: 60,
    }
}
fn ready() -> (Assurance, csgn::Signer) {
    let mut signer = signer();
    let feed = feed(&mut signer, 10, 1);
    let authority = authority(&signer);
    let mut a = Assurance::new(ORIGIN, "alpha", config()).unwrap();
    let request = a.start(NOW).unwrap();
    a.on_feed(request.id(), &bytes(&feed), &authority, NOW)
        .unwrap();
    (a, signer)
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn startup_announcements_and_atomic_install() {
    let (mut a, mut s) = ready();
    assert_eq!(a.feed_state(), State::Following);
    assert_eq!(a.current(NOW).unwrap().revision(), 10);
    assert!(a.next(NOW + 1).unwrap().is_none());
    let poll = a.next(NOW + 2).unwrap().unwrap();
    assert_eq!(poll.path(), "/v1/trust_changes");
    assert_eq!(poll.body(), serde_json::json!({"revision":10}));
    a.on_announcement(
        poll.id(),
        br#"{"revision":11,"policy_epoch":2,"changed":true}"#,
        NOW + 2,
    )
    .unwrap();
    assert!(a.current(NOW + 2).is_err());
    let request = a.next(NOW + 2).unwrap().unwrap();
    assert_eq!(request.action(), Action::Fetch);
    assert_eq!(request.body(), serde_json::json!({}));
    let current = feed(&mut s, 11, 2);
    a.on_feed(request.id(), &bytes(&current), &authority(&s), NOW + 2)
        .unwrap();
    assert_eq!(a.current(NOW + 2).unwrap().revision(), 11);
    assert_eq!(
        a.on_announcement(poll.id(), br#"{}"#, NOW + 2),
        Err(Error::Obsolete)
    );
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn lost_announcements_refresh_and_failure_never_extends_freshness() {
    let (mut a, _) = ready();
    // No change delivery: the bounded timer still fetches public material.
    let refresh = a.next(NOW + 40).unwrap().unwrap();
    assert_eq!(refresh.action(), Action::Fetch);
    a.on_error(refresh.id(), NOW + 40, u64::MAX).unwrap();
    assert_eq!(a.feed_state(), State::Backoff);
    assert!(a.next(NOW + 41).unwrap().is_none());
    assert!(a.current(NOW + 59).is_ok());
    assert_eq!(a.current(NOW + 60).unwrap_err(), Error::Unavailable);
    let retry = a.next(NOW + 60).unwrap().unwrap();
    assert_eq!(retry.action(), Action::Fetch);
    assert!(a.next(NOW + 60).unwrap().is_none());
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn cancellations_duplicates_clock_regression_and_restart_are_fenced() {
    let (mut a, mut s) = ready();
    let poll = a.next(NOW + 2).unwrap().unwrap();
    a.stop();
    assert_eq!(a.feed_state(), State::Stopped);
    assert!(a.current(NOW + 2).is_err());
    assert_eq!(
        a.on_announcement(poll.id(), br#"{}"#, NOW + 2),
        Err(Error::Obsolete)
    );
    let fresh = a.start(NOW + 2).unwrap();
    assert!(fresh.id() > poll.id());
    assert!(a.current(NOW + 2).is_err());
    let v = feed(&mut s, 9, 1);
    assert_eq!(
        a.on_feed(fresh.id(), &bytes(&v), &authority(&s), NOW + 2),
        Err(Error::Trust(cnvy::cchr::Error::Rollback))
    );
    let next = a.next(NOW + 5).unwrap().unwrap();
    assert_eq!(a.next(NOW + 4), Err(Error::ClockRegression));
    assert_eq!(a.on_error(next.id(), NOW + 5, 0), Err(Error::Obsolete));
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn timeout_and_retry_storms_have_one_bounded_operation() {
    let mut a = Assurance::new(ORIGIN, "alpha", config()).unwrap();
    let initial = a.start(NOW).unwrap();
    assert!(a.next(NOW + 25).unwrap().is_none());
    assert!(a.next(NOW + 26).unwrap().is_none());
    assert_eq!(a.on_error(initial.id(), NOW + 26, 0), Err(Error::Obsolete));
    let mut count = 0;
    for offset in 27..200 {
        if let Some(r) = a.next(NOW + offset).unwrap() {
            count += 1;
            a.on_error(r.id(), NOW + offset, 0).unwrap();
            for _ in 0..20 {
                a.on_change(u64::MAX, NOW + offset).unwrap();
                assert!(a.next(NOW + offset).unwrap().is_none());
            }
        }
    }
    assert!((1..=15).contains(&count));
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_announcements_and_corrupt_feed_cannot_become_trust() {
    let (mut a, s) = ready();
    let poll = a.next(NOW + 2).unwrap().unwrap();
    assert_eq!(
        a.on_announcement(
            poll.id(),
            br#"{"revision":11,"policy_epoch":2,"changed":false}"#,
            NOW + 2
        ),
        Err(Error::Response)
    );
    assert_eq!(a.current(NOW + 2).unwrap().revision(), 10);
    let fetch = a.next(NOW + 4).unwrap().unwrap();
    assert!(
        a.on_feed(fetch.id(), b"{}", &authority(&s), NOW + 4)
            .is_err()
    );
    assert_eq!(a.current(NOW + 4).unwrap().revision(), 10);
    a.on_change(11, NOW + 4).unwrap();
    assert!(a.current(NOW + 4).is_err());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn arbitrary_epoch_hint_is_bounded_by_the_next_verified_epoch() {
    let (mut a, mut s) = ready();
    let poll = a.next(NOW + 2).unwrap().unwrap();
    a.on_policy_epoch(u64::MAX, NOW + 2).unwrap();
    assert!(a.current(NOW + 2).is_err());
    assert_eq!(
        a.on_announcement(poll.id(), b"{}", NOW + 2),
        Err(Error::Obsolete)
    );
    let refresh = a.next(NOW + 2).unwrap().unwrap();
    let old = feed(&mut s, 11, 1);
    assert_eq!(
        a.on_feed(refresh.id(), &bytes(&old), &authority(&s), NOW + 2),
        Err(Error::BehindAnnouncement)
    );
    assert!(a.current(NOW + 2).is_err());
    let fresh = a.next(NOW + 4).unwrap().unwrap();
    let new = feed(&mut s, 12, 2);
    a.on_feed(fresh.id(), &bytes(&new), &authority(&s), NOW + 4)
        .unwrap();
    assert_eq!(a.current(NOW + 4).unwrap().policy_epoch(), 2);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn configuration_and_stopped_operations_are_explicit() {
    for case in 0..8 {
        let mut invalid = config();
        match case {
            0 => invalid.refresh_seconds = 0,
            1 => invalid.poll_seconds = 0,
            2 => invalid.request_seconds = 25,
            3 => invalid.initial_backoff_seconds = 0,
            4 => invalid.maximum_backoff_seconds = 1,
            5 => invalid.maximum_backoff_seconds = 86401,
            6 => invalid.maximum_staleness_seconds = 1,
            _ => invalid.maximum_staleness_seconds = 86401,
        }
        assert!(matches!(
            cnvy::feed::Follower::new(invalid),
            Err(Error::Configuration)
        ));
    }
    let mut follower = cnvy::feed::Follower::new(config()).unwrap();
    assert_eq!(follower.next(0, NOW), Ok(None));
    assert_eq!(follower.on_change(1, 0, NOW), Err(Error::Stopped));
    assert_eq!(follower.refresh(NOW), Err(Error::Stopped));
    assert_eq!(follower.on_policy_epoch(1, 0, NOW), Err(Error::Stopped));
    assert_eq!(follower.permits_current(0, 0, NOW), Err(Error::Unavailable));
    let r = follower.start(NOW).unwrap();
    assert_eq!(follower.permits_current(0, 0, NOW), Err(Error::Unavailable));
    assert_eq!(follower.start(NOW), Err(Error::Busy));
    assert_eq!(follower.failed(r.id() + 1, NOW, 0), Err(Error::Obsolete));
    assert_eq!(
        follower.failed(r.id(), r.deadline(), 0),
        Err(Error::Timeout)
    );
    assert_eq!(Error::Timeout.to_string(), "Timeout");
    assert!(matches!(
        cnvy::feed::Follower::new(config()).unwrap().start(u64::MAX),
        Err(Error::Exhausted)
    ));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn response_types_and_every_announcement_field_are_checked() {
    let mut follower = cnvy::feed::Follower::new(config()).unwrap();
    let r = follower.start(NOW).unwrap();
    assert_eq!(
        follower.announcement(r.id(), b"{}", NOW),
        Err(Error::Response)
    );
    let candidates = [
        vec![b' '; 1025],
        b"not JSON".to_vec(),
        br#"{"revision":10,"policy_epoch":0,"changed":false}"#.to_vec(),
        br#"{"revision":9,"policy_epoch":1,"changed":false}"#.to_vec(),
        br#"{"revision":10,"policy_epoch":1,"changed":true}"#.to_vec(),
    ];
    for bytes in candidates {
        let (mut a, _) = ready();
        let poll = a.next(NOW + 2).unwrap().unwrap();
        assert_eq!(
            a.on_announcement(poll.id(), &bytes, NOW + 2),
            Err(Error::Response)
        );
    }
    let (mut a, mut signer) = ready();
    let poll = a.next(NOW + 2).unwrap().unwrap();
    let value = feed(&mut signer, 10, 1);
    assert_eq!(
        a.on_feed(poll.id(), &bytes(&value), &authority(&signer), NOW + 2),
        Err(Error::Response)
    );
    let (mut a, _) = ready();
    let poll = a.next(NOW + 2).unwrap().unwrap();
    a.on_announcement(
        poll.id(),
        br#"{"revision":10,"policy_epoch":1,"changed":false}"#,
        NOW + 2,
    )
    .unwrap();
    a.on_policy_epoch(1, NOW + 2).unwrap();
    assert!(a.current(NOW + 2).is_ok());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn direct_refresh_fences_polls_and_preserves_backoff() {
    let mut signer = signer();
    let value = feed(&mut signer, 10, 1);
    let mut charter = cnvy::cchr::Charter::new(ORIGIN, "alpha").unwrap();
    let mut follower = cnvy::feed::Follower::new(config()).unwrap();
    let r = follower.start(NOW).unwrap();
    follower
        .feed(
            r.id(),
            &bytes(&value),
            &authority(&signer),
            &mut charter,
            NOW,
        )
        .unwrap();
    let poll = follower.next(10, NOW + 2).unwrap().unwrap();
    follower.refresh(NOW + 2).unwrap();
    assert_eq!(
        follower.announcement(poll.id(), b"{}", NOW + 2),
        Err(Error::Obsolete)
    );
    let fetch = follower.next(10, NOW + 2).unwrap().unwrap();
    follower.failed(fetch.id(), NOW + 2, 0).unwrap();
    follower.refresh(NOW + 2).unwrap();
    assert_eq!(follower.next(10, NOW + 2), Ok(None));
    // The old publication remains valid, but cannot satisfy a newer revision hint.
    follower.on_change(11, 10, NOW + 2).unwrap();
    let fetch = follower.next(10, NOW + 4).unwrap().unwrap();
    assert_eq!(
        follower.feed(
            fetch.id(),
            &bytes(&value),
            &authority(&signer),
            &mut charter,
            NOW + 4
        ),
        Err(Error::BehindAnnouncement)
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn newer_revision_fences_an_in_flight_poll() {
    let (mut assurance, _) = ready();
    let poll = assurance.next(NOW + 2).unwrap().unwrap();
    assurance.on_change(11, NOW + 2).unwrap();
    assert_eq!(
        assurance.on_announcement(poll.id(), b"{}", NOW + 2),
        Err(Error::Obsolete)
    );
    assert_eq!(
        assurance.next(NOW + 2).unwrap().unwrap().action(),
        Action::Fetch
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn automatic_failures_use_injected_entropy_for_bounded_retry() {
    for case in 0..4 {
        let mut config = config();
        config.retry_entropy = 1;
        let mut follower = cnvy::feed::Follower::new(config).unwrap();
        let request = follower.start(NOW).unwrap();
        let failed_at = if case < 2 { request.deadline() } else { NOW };
        match case {
            0 => assert_eq!(follower.next(0, failed_at), Ok(None)),
            1 => assert_eq!(
                follower.failed(request.id(), failed_at, 1),
                Err(Error::Timeout)
            ),
            2 => {
                let mut charter = cnvy::cchr::Charter::new(ORIGIN, "alpha").unwrap();
                assert!(
                    follower
                        .feed(
                            request.id(),
                            b"{}",
                            &authority(&signer()),
                            &mut charter,
                            failed_at
                        )
                        .is_err()
                );
            }
            _ => assert_eq!(
                follower.announcement(request.id(), b"{}", failed_at),
                Err(Error::Response)
            ),
        }
        assert_eq!(follower.next(0, failed_at + 2), Ok(None));
        assert!(follower.next(0, failed_at + 3).unwrap().is_some());
    }
}
