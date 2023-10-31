use islet_sdk::prelude::*;

fn attestation() -> Result<(), Error> {
    let user_data = b"User data";
    let report = attest(user_data)?;
    let claims = verify(&report)?;
    print_claims(&claims);

    if let Some(ClaimData::Bstr(data)) = parse(&claims, config::STR_USER_DATA) {
        assert_eq!(user_data, &data[..user_data.len()]);
    } else {
        assert!(false, "Wrong user data");
    }

    if let Some(ClaimData::Text(data)) = parse(&claims, config::STR_PLAT_PROFILE) {
        assert_eq!(data, "http://arm.com/CCA-SSD/1.0.0");
    } else {
        assert!(false, "Wrong platform profile");
    }

    if let Some(ClaimData::Bstr(data)) = parse(&claims, config::STR_REALM_INITIAL_MEASUREMENT) {
        println!("Realm initial measurement: {:X?}", &data);
    } else {
        assert!(false, "Wrong RIM");
    }

    Ok(())
}

fn sealing() -> Result<(), Error> {
    let plaintext = b"Plaintext";
    let sealed = seal(plaintext)?;
    let unsealed = unseal(&sealed)?;
    assert_eq!(plaintext, &unsealed[..]);
    Ok(())
}

fn main() {
    println!("# ISLET SDK examples: An example app running on aarch64");
    println!("Attestation result {:?}", attestation());
    println!("Sealing result {:?}", sealing());
}
