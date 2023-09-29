use islet_hes::{
    self, calculate_public_key_hash, AttestationMgr, BootMeasurement, BootMeasurementMetadata,
    HWAsymmetricKey, HWClaims, HWData, HWHash, HWSymmetricKey, HashAlgo, KeyMaterialData,
    MeasurementMgr, NUM_OF_MEASUREMENT_SLOTS,
};
use coset::TaggedCborSerializable;
use tinyvec::ArrayVec;

use std::fs::{self, File};
use std::io::{Read, Result as IOResult, Write};
struct DummyHW();

type DummyError = ();

impl HWData for DummyHW {
    type Error = DummyError;
    fn boot_measurements(&self) -> Result<Vec<BootMeasurement>, DummyError> {
        Ok(Vec::from([
            BootMeasurement {
                measurement_value: [
                    0x61, 0x97, 0x3b, 0x4f, 0x62, 0x0c, 0x2a, 0xe6, 0xc7, 0x63, 0x51, 0x18, 0xa0,
                    0xb4, 0x37, 0x6d, 0x15, 0x34, 0x4c, 0x1c, 0x53, 0xa2, 0x17, 0x89, 0xb1, 0xaa,
                    0x95, 0xd2, 0x0f, 0x3c, 0x45, 0x06,
                ]
                .iter()
                .cloned()
                .collect(),
                metadata: BootMeasurementMetadata {
                    signer_id: [
                        0xc6, 0xc3, 0x2a, 0x95, 0x7d, 0xf4, 0xc6, 0x69, 0x8c, 0x55, 0x0b, 0x69,
                        0x5d, 0x02, 0x2e, 0xd5, 0x18, 0x0c, 0xae, 0x71, 0xf8, 0xb4, 0x9c, 0xbb,
                        0x75, 0xe6, 0x06, 0x1c, 0x2e, 0xf4, 0x97, 0xe1,
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    measurement_type: 0,
                    sw_type: b"BL1".iter().cloned().collect(),
                    sw_version: b"0.1.0".iter().cloned().collect(),
                },
            },
            BootMeasurement {
                measurement_value: [
                    0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7,
                    0xea, 0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05,
                    0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6,
                    0xFF, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49,
                    0x99, 0x31, 0xcf, 0x59, 0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
                ]
                .into(),
                metadata: BootMeasurementMetadata {
                    signer_id: [
                        0xa0, 0x64, 0xb1, 0xad, 0x60, 0xfa, 0x18, 0x33, 0x94, 0xdd, 0xa5, 0x78,
                        0x91, 0x35, 0x7f, 0x97, 0x2e, 0x4f, 0xe7, 0x22, 0x78, 0x2a, 0xdf, 0xf1,
                        0x85, 0x4c, 0x8b, 0x2a, 0x14, 0x2c, 0x04, 0x10,
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    measurement_type: 2,
                    sw_type: b"BL2".iter().cloned().collect(),
                    sw_version: b"1.9.0+0".iter().cloned().collect(),
                },
            },
        ]))
    }

    fn huk(&self) -> Result<HWSymmetricKey, DummyError> {
        Ok([
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
            0x0c, 0x0d, 0x0e, 0x0f,
        ]
        .into())
    }

    // Equivalent to res/dummy_guk.bin
    fn guk(&self) -> Result<HWSymmetricKey, DummyError> {
        Ok([
            0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67,
            0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45,
            0x67, 0x89, 0x01, 0x23,
        ]
        .into())
    }

    // Equivalent to res/bl2_signed_hash.bin
    fn bl_hash(&self) -> Result<HWHash, Self::Error> {
        Ok([
            0xf1, 0x5f, 0x95, 0x3b, 0xe5, 0x0d, 0xad, 0x92, 0xc3, 0xb2, 0xaa, 0x32, 0x97, 0xe6,
            0xa4, 0xa8, 0xd6, 0x6d, 0x33, 0x63, 0x84, 0x49, 0xec, 0x19, 0x22, 0xb4, 0xa7, 0x92,
            0x4a, 0x7b, 0x30, 0x22,
        ]
        .iter()
        .cloned()
        .collect())
    }

    fn cpak(&self) -> Result<Option<HWAsymmetricKey>, DummyError> {
        Ok(None)
    }

    fn implementation_id(&self) -> Result<[u8; 32], DummyError> {
        Ok([
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB,
            0xBB, 0xBB, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xDD, 0xDD, 0xDD, 0xDD,
            0xDD, 0xDD, 0xDD, 0xDD,
        ])
    }

    fn profile_definition(&self) -> Result<Option<ArrayVec<[u8; 32]>>, DummyError> {
        Ok(Some(
            b"http://arm.com/CCA-SSD/1.0.0".iter().cloned().collect(),
        ))
    }

    fn security_lifecycle(&self) -> Result<u32, DummyError> {
        Ok(0x3000)
    }

    fn verification_service_url(&self) -> Result<Option<ArrayVec<[u8; 32]>>, DummyError> {
        Ok(Some(b"http://whatever.com".iter().cloned().collect()))
    }

    fn platform_config(&self) -> Result<ArrayVec<[u8; 32]>, DummyError> {
        Ok(0xDEADBEEFu32.to_ne_bytes().iter().cloned().collect())
    }
}

// Can be used instead of hardcoded guk and bl hash.
fn _load_binary_file(filename: &str) -> IOResult<Vec<u8>> {
    let mut f = File::open(filename)?;
    let metadata = fs::metadata(filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

fn save_binary_file(filename: &str, data: &[u8]) -> IOResult<()> {
    let mut f = File::create(filename)?;
    f.write_all(data)
}

fn main() {
    println!("Creating test token in out/platform.bin");
    let hw_data = DummyHW();

    // let guk = load_binary_file("res/dummy_guk.bin").unwrap();
    // let hash = load_binary_file("res/bl2_signed_hash.bin").unwrap();

    let key_material = KeyMaterialData {
        guk: hw_data.guk().unwrap(),
        hash: hw_data.bl_hash().unwrap(),
    };

    let hw_claims = HWClaims {
        implementation_id: hw_data.implementation_id().unwrap().into(),
        platform_config: hw_data.platform_config().unwrap(),
        profile_definition: Some(
            String::from_utf8(
                hw_data
                    .profile_definition()
                    .unwrap()
                    .unwrap()
                    .as_slice()
                    .to_vec(),
            )
            .unwrap(),
        ),
        security_lifecycle: hw_data.security_lifecycle().unwrap().try_into().unwrap(),
        verification_service_url: Some(
            String::from_utf8(
                hw_data
                    .verification_service_url()
                    .unwrap()
                    .unwrap()
                    .as_slice()
                    .to_vec(),
            )
            .unwrap(),
        ),
    };

    let measurement_mgr = MeasurementMgr::init(hw_data.boot_measurements().unwrap()).unwrap();
    let mut measurements = Vec::new();
    for i in 0..NUM_OF_MEASUREMENT_SLOTS {
        if let Ok((measurement, _)) = measurement_mgr.read_measurement(i) {
            measurements.push(measurement.clone());
        }
    }

    let mut attestation_mgr = AttestationMgr::init(key_material, hw_claims);

    let dak_scalar_bytes = attestation_mgr
        .get_delegated_key(
            islet_hes::ECCFamily::SecpR1,
            islet_hes::KeyBits::Bits384,
            islet_hes::HashAlgo::Sha256,
            &measurements,
        )
        .unwrap();

    let priv_dak = p384::SecretKey::from_slice(&dak_scalar_bytes).unwrap();

    // let scalar = dak.as_scalar_primitive().to_bytes().to_vec();

    let dak_pub_hash = calculate_public_key_hash(
        priv_dak.public_key().to_sec1_bytes().to_vec(),
        HashAlgo::Sha256,
    );

    let token = attestation_mgr
        .get_platform_token(&dak_pub_hash, &measurements)
        .unwrap()
        .to_tagged_vec()
        .expect("Couldn't export CoseSign1 to tagged cbor");

    save_binary_file(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../out/", "platform_token.bin"),
        &token,
    )
    .unwrap();
    save_binary_file(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../out/", "dak_priv.bin"),
        &dak_scalar_bytes,
    )
    .unwrap();
    save_binary_file(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../out/", "dak_pub.bin"),
        &priv_dak.public_key().to_sec1_bytes(),
    )
    .unwrap();
}
