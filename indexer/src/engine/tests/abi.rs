use super::*;

#[test]
fn decodes_indexed_address_topic() {
    let topic = "0x000000000000000000000000ab5801a7d398351b8be11c439e05c5b3259aec9b";
    assert_eq!(
        topic_to_address(topic).unwrap(),
        "0xab5801a7d398351b8be11c439e05c5b3259aec9b"
    );
}

#[test]
fn decodes_uint_topic() {
    let topic = "0x00000000000000000000000000000000000000000000000000000000000004d2";
    assert_eq!(topic_to_decimal(topic).unwrap(), "1234");
}

#[test]
fn decodes_dynamic_string() {
    let ret = "0x0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000003\
                 4741530000000000000000000000000000000000000000000000000000000000";
    assert_eq!(decode_string(ret).unwrap(), "GAS");
}

#[test]
fn decodes_bytes32_string() {
    let ret = "0x4d4b520000000000000000000000000000000000000000000000000000000000";
    assert_eq!(decode_string(ret).unwrap(), "MKR");
}

#[test]
fn decodes_decimals() {
    let ret = "0x0000000000000000000000000000000000000000000000000000000000000012";
    assert_eq!(decode_small_uint(ret).unwrap(), 18);
}

#[test]
fn rejects_ragged_data() {
    assert!(words("0x1234").is_none());
    assert!(decode_string("0x").is_none());
}
