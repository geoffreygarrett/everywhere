
//! Basic invariants that must hold on ALL targets.

use everywhere_net::WsMessage;

#[test]
fn debug_and_eq() {
    let a = WsMessage::Text("hello".into());
    let b = format!("{a:?}");
    assert!(b.contains("Text"));
    assert_eq!(a.clone(), a);
}

#[test]
fn binary_debug_len() {
    let msg = WsMessage::Binary(vec![1, 2, 3].into());
    assert!(format!("{msg:?}").contains("3")); // length printed
}
