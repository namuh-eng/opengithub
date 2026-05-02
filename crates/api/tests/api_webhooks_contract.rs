use opengithub_api::domain::webhooks::{hmac_sha256_signature, validate_events, SUPPORTED_EVENTS};

#[test]
fn webhook_event_catalog_is_limited_to_supported_events() {
    assert!(SUPPORTED_EVENTS.contains(&"push"));
    assert!(SUPPORTED_EVENTS.contains(&"pull_request_review"));
    assert!(SUPPORTED_EVENTS.contains(&"check_run"));
    assert_eq!(validate_events(&["push".to_owned(), "push".to_owned()]).unwrap(), vec!["push"]);
    assert!(validate_events(&["delete".to_owned()]).is_err());
}

#[test]
fn webhook_hmac_signature_uses_raw_body_bytes() {
    let signature = hmac_sha256_signature("top-secret", br#"{"zen":"Keep it logically awesome."}"#);
    assert_eq!(signature, "sha256=b6e01c8d3eb025ec0a3275f74e78b1c5c256bcec00e48ef382e76b2b8b272e99");
}
