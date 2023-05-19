use islet_sdk::prelude::*;

fn attestation() -> Result<(), Error> {
    let user_data = b"User data";
    let report = attest(user_data)?;
    let claims = verify(&report)?;
    println!("Debug: {:?}", claims);

    if let claim::Value::Bytes(value) = &claims.value(config::STR_USER_DATA).ok_or(Error::Claims)? {
        assert_eq!(user_data, &value[..user_data.len()]);
    } else {
        assert!(false, "Wrong user data");
    }

    if let claim::Value::String(value) = &claims
        .value(config::STR_PLAT_PROFILE)
        .ok_or(Error::Claims)?
    {
        assert_eq!(value.as_str(), "http://arm.com/CCA-SSD/1.0.0");
    } else {
        assert!(false, "Wrong platform profile");
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
