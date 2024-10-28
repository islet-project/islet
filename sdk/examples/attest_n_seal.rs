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
        println!("Realm initial measurement: {:X?}", hex::encode(&data));
    } else {
        assert!(false, "Wrong RIM");
    }

    Ok(())
}

fn sealing() -> Result<(), Error> {
    let plaintext = b"\
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nunc ornare lacinia accumsan. Nam eleifend vel nisl et
    commodo. Quisque in tortor non risus dictum varius. Curabitur pulvinar tellus vitae sapien gravida dapibus. Ut nec
    imperdiet sem, eu ornare turpis. Donec a lectus vitae enim malesuada aliquam sed ac turpis. Mauris turpis massa, mollis
    et ex vitae, tempor gravida odio. Maecenas erat urna, laoreet et ornare auctor, faucibus nec elit. In luctus turpis
    sapien, vel posuere libero pulvinar et. Donec maximus sollicitudin condimentum. Mauris condimentum ex vel purus
    scelerisque faucibus. Donec dapibus viverra massa ut iaculis.

    Maecenas eget sollicitudin lorem. Aenean euismod ultricies dui quis fringilla. Pellentesque sit amet dapibus metus.
    Vivamus tincidunt convallis lectus eget lacinia. Aliquam ac nisl vel erat pulvinar accumsan. Aliquam ut ante id nunc
    molestie rutrum. Pellentesque facilisis venenatis erat, ac ornare elit posuere in. Integer porttitor sit amet tortor at
    lobortis. Morbi imperdiet rutrum metus sed malesuada. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices
    posuere cubilia curae; Vestibulum lacinia eu justo nec auctor. Vivamus suscipit a erat in ultricies. Curabitur sit amet
    egestas turpis. Sed semper nunc at diam varius, in congue nisl pretium. Proin aliquam magna mi.

    Ut imperdiet diam ut nisi consequat tincidunt. Sed imperdiet purus vel fermentum dignissim. Etiam at cursus libero. In
    leo metus, sagittis at dictum vel, mattis quis quam. Pellentesque in erat purus. Suspendisse in pretium urna, sed
    tincidunt felis. Sed dapibus sed ipsum ut mattis. Pellentesque iaculis, dui eget congue hendrerit, velit est
    sollicitudin leo, ac ullamcorper diam ligula quis felis. Etiam fermentum magna quis enim pretium, sed rhoncus metus
    dignissim.

    Integer dignissim hendrerit enim, nec blandit massa. Aliquam porttitor dolor vel congue commodo. Donec maximus dui non
    neque congue, et aliquet odio pharetra. Integer varius magna vitae dolor efficitur aliquam. Aenean suscipit quam et
    lectus tincidunt congue. Aliquam vitae libero dolor. Class aptent taciti sociosqu ad litora torquent per conubia
    nostra, per inceptos himenaeos.

    Donec et ultrices diam, vitae vulputate ligula. Fusce tempor pellentesque commodo. Fusce quam eros, ultrices quis nibh
    in, fermentum laoreet tortor. Nam eget tortor et purus dignissim placerat. Etiam eu tellus et leo imperdiet tempor et
    porttitor diam. Etiam risus enim, viverra non euismod at, eleifend eu elit. Mauris posuere est lacus, at interdum felis
    semper in. Aliquam varius euismod velit, eu iaculis felis malesuada et. Fusce laoreet ac est a euismod. Phasellus vel
    sapien dolor. Maecenas vehicula lorem ac orci luctus, vel sodales libero sollicitudin. Nunc id orci mattis, rutrum nunc
    id, luctus mauris. Sed sem arcu, pretium quis pellentesque sed, bibendum vel tellus. Integer dapibus pretium ligula, at
    rhoncus ante iaculis vel. Proin feugiat enim ut diam mollis, at vestibulum libero vestibulum.";

    let sealed = seal(plaintext)?;
    let unsealed = unseal(&sealed)?;
    assert_eq!(plaintext, &unsealed[..]);
    Ok(())
}

fn main() {
    println!("Attestation result {:?}", attestation());
    println!("Sealing result {:?}", sealing());
}
