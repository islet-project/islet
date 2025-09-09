use alloc::{borrow::ToOwned, vec::Vec};
use ciborium::ser;
use coset::{iana, AsCborValue, CoseKeyBuilder};
use ecdsa::elliptic_curve::sec1::ToEncodedPoint;

// Convert SEC1 encoded EC2 public `key` to COSE/CBOR
// Handles only p384 for now, others can be uncommented when needed
pub fn ec_public_key_sec1_to_cose(key: &[u8]) -> Vec<u8> {
    // let p256_sec1_len = 1 + 2 * 32;
    let p384_sec1_len = 1 + 2 * 48;
    // let p521_sec1_len = 1 + 2 * 66;

    let key_cbor_value = match key.len() {
        // n if n == p256_sec1_len => {
        //     let pk = p256::PublicKey::from_sec1_bytes(key).expect("Failed to load p256 sec1 key");
        //     let ep = pk.to_encoded_point(false);
        //     let x = ep.x().unwrap().to_owned().to_vec();
        //     let y = ep.y().unwrap().to_owned().to_vec();
        //     let key = CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_256, x, y).build();
        //     key.to_cbor_value().expect("Failed to encode p256 as CBOR")
        // }
        n if n == p384_sec1_len => {
            let pk = p384::PublicKey::from_sec1_bytes(key).expect("Failed to load p384 sec1 key");
            let ep = pk.to_encoded_point(false);
            let x = ep.x().unwrap().to_owned().to_vec();
            let y = ep.y().unwrap().to_owned().to_vec();
            let key = CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_384, x, y).build();
            key.to_cbor_value().expect("Failed to encode p384 as CBOR")
        }
        // n if n == p521_sec1_len => {
        //     let pk = p521::PublicKey::from_sec1_bytes(key).expect("Failed to load p521 sec1 key");
        //     let ep = pk.to_encoded_point(false);
        //     let x = ep.x().unwrap().to_owned().to_vec();
        //     let y = ep.y().unwrap().to_owned().to_vec();
        //     let key = CoseKeyBuilder::new_ec2_pub_key(iana::EllipticCurve::P_521, x, y).build();
        //     key.to_cbor_value().expect("Failed to encode p521 as CBOR")
        // }
        _ => panic!("Wrong sec1 key length"),
    };

    let mut key_cbor_bytes = Vec::new();
    ser::into_writer(&key_cbor_value, &mut key_cbor_bytes).expect("Failed to serialize CBOR value");
    key_cbor_bytes
}
