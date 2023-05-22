use super::*;


const COLUMN: usize = 30;


fn print_indent(indent: i32)
{
    for _ in 0..indent {
        print!("  ");
    }
}

fn print_claim(claim: &Claim, indent: i32)
{
    print_indent(indent);

    if claim.present {
        match &claim.data {
            ClaimData::Int64(i) => println!("{:COLUMN$} (#{}) = {}", claim.title, claim.key, i),
            ClaimData::Bool(b) => println!("{:COLUMN$} (#{}) = {}", claim.title, claim.key, b),
            ClaimData::Bstr(v) => println!("{:COLUMN$} (#{}) = [{}]", claim.title, claim.key, hex::encode(v)),
            ClaimData::Text(s) => println!("{:COLUMN$} (#{}) = \"{}\"", claim.title, claim.key, s),
        }
    } else {
        let mandatory = if claim.mandatory { "mandatory " } else { "" };
        println!("* Missing {}claim with key: {} ({})",
                 mandatory, claim.key, claim.title);
    }
}

fn print_cose_sign1(token_type: &str,
                    cose_sign1: &CoseSign1)
{
    println!("== {} Token cose:", token_type);
    println!("{:COLUMN$} = {:?}", "Protected header", cose_sign1.protected.header);
    println!("{:COLUMN$} = {:?}", "Unprotected header", cose_sign1.unprotected);
    //println!("{:COLUMN$} = [{}]", "Token payload", hex::encode(cose_sign1.payload.as_ref().unwrap_or(&Vec::new())));
    println!("{:COLUMN$} = [{}]", "Signature", hex::encode(&cose_sign1.signature));
    println!("== End of {} Token cose\n", token_type);
}

#[allow(dead_code)]
fn print_cose_sign1_wrapper(token_type: &str,
                            cose_sign1_wrapper: &[Claim])
{
    println!("== {} Token cose wrapper:", token_type);
    print_claim(&cose_sign1_wrapper[0], 0);
    //print_claim(&cose_sign1_wrapper[1], 0);  // token payload
    print_claim(&cose_sign1_wrapper[2], 0);
    println!("== End of {} Token cose wrapper\n", token_type);
}

pub fn print_token(claims: &AttestationClaims)
{
    print_cose_sign1("Realm", &claims.realm_cose_sign1);
    //print_cose_sign1_wrapper("Realm", &claims.realm_cose_sign1_wrapper);

    println!("== Realm Token:");
    for token in &claims.realm_token_claims {
        print_claim(token, 0);
    }
    println!("{:COLUMN$} (#{})", "Realm measurements", CCA_REALM_EXTENSIBLE_MEASUREMENTS);
    for claim in &claims.realm_measurement_claims {
        print_claim(claim, 1);
    }
    println!("== End of Realm Token.\n\n");

    print_cose_sign1("Platform", &claims.plat_cose_sign1);
    //print_cose_sign1_wrapper("Platform", &claims.plat_cose_sign1_wrapper);

    println!("== Platform Token:");
    for claim in &claims.plat_token_claims {
        print_claim(claim, 0);
    }

    let mut count = 0;
    println!("{:COLUMN$} (#{})", "Platform SW components", CCA_PLAT_SW_COMPONENTS);
    for component in &claims.sw_component_claims {
        if component.present {
            println!("  SW component #{}:", count);
            for claim in &component.claims {
                print_claim(&claim, 2);
            }
            count += 1;
        }
    }
    println!("== End of Platform Token\n");
}
