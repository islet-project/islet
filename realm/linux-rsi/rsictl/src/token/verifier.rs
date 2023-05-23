use super::*;
use ciborium::{de, value::Value};
use coset::{AsCborValue, CoseSign1};
use p384::ecdsa::signature::Verifier;
use std::str::FromStr;

fn unpack_i64(val: &Value) -> Result<i64, TokenError>
{
    if let Value::Integer(i) = val {
        if let Ok(i) = (*i).try_into() {
            return Ok(i);
        }
    }

    Err(TokenError::InvalidTokenFormat("unpack i64 failed"))
}

fn unpack_array(val: Value, err: &'static str) -> Result<Vec<Value>, TokenError>
{
    if let Value::Array(vec) = val {
        Ok(vec)
    } else {
        Err(TokenError::InvalidTokenFormat(err))
    }
}

fn unpack_map(val: Value, err: &'static str) -> Result<Vec<(Value, Value)>, TokenError>
{
    if let Value::Map(v) = val {
        Ok(v)
    } else {
        Err(TokenError::InvalidTokenFormat(err))
    }
}

fn unpack_tag(val: Value, id: u64, err: &'static str) -> Result<Value, TokenError>
{
    if let Value::Tag(tag, data) = val {
        if tag != id {
            return Err(TokenError::InvalidTag(err));
        }
        let unboxed = *data;
        Ok(unboxed)
    } else {
        Err(TokenError::InvalidTokenFormat(err))
    }
}

fn unpack_keyed_array(
    tupple: (Value, Value),
    id: u32,
    err: &'static str,
) -> Result<Vec<Value>, TokenError>
{
    if let (Value::Integer(key), Value::Array(vec)) = tupple {
        if key != id.into() {
            return Err(TokenError::InvalidKey(err));
        }
        Ok(vec)
    } else {
        Err(TokenError::InvalidTokenFormat("unpack vec elem failed"))
    }
}

fn unpack_keyed_bytes(
    tupple: (Value, Value),
    id: u32,
    err: &'static str,
) -> Result<Vec<u8>, TokenError>
{
    if let (Value::Integer(key), Value::Bytes(vec)) = tupple {
        if key != id.into() {
            return Err(TokenError::InvalidKey(err));
        }
        Ok(vec)
    } else {
        Err(TokenError::InvalidTokenFormat(err))
    }
}

fn get_claim(val: Value, claim: &mut Claim) -> Result<(), TokenError>
{
    match (val, &claim.data) {
        (Value::Bool(b), ClaimData::Bool(_)) => claim.data = ClaimData::Bool(b),
        (i @ Value::Integer(_), ClaimData::Int64(_)) => {
            claim.data = ClaimData::Int64(unpack_i64(&i)?)
        }
        (Value::Bytes(v), ClaimData::Bstr(_)) => claim.data = ClaimData::Bstr(v),
        (Value::Text(s), ClaimData::Text(_)) => claim.data = ClaimData::Text(s),
        _ => {
            return Err(TokenError::InvalidTokenFormat(
                "incompatible claim data type",
            ))
        }
    }

    claim.present = true;

    Ok(())
}

fn find_claim(claims: &mut [Claim], key: i64) -> Option<&mut Claim>
{
    for elem in claims {
        if elem.key == key {
            return Some(elem);
        }
    }

    None
}

fn get_claims_from_map(
    map: Vec<(Value, Value)>,
    claims: &mut [Claim],
) -> Result<Vec<(Value, Value)>, TokenError>
{
    let mut not_found = Vec::<(Value, Value)>::new();

    for (orig_key, val) in map {
        let key = unpack_i64(&orig_key)?;
        let claim = find_claim(claims, key);
        if let Some(claim) = claim {
            claim.key = key;
            get_claim(val, claim)?;
        } else {
            not_found.push((orig_key, val));
        }
    }

    // return the rest if any
    Ok(not_found)
}

fn verify_realm_token(attest_claims: &mut AttestationClaims) -> Result<(), TokenError>
{
    let realm_payload = attest_claims.realm_cose_sign1_wrapper[1].data.get_bstr();
    let val = de::from_reader(&realm_payload[..])?;
    let map = unpack_map(val, "realm token not a map")?;

    // main parsing
    let rest = get_claims_from_map(map, &mut attest_claims.realm_token_claims)?;

    // there should be one element left, rems array
    if rest.len() != 1 {
        return Err(TokenError::InvalidTokenFormat("no rems"));
    }

    let rems = rest.into_iter().next().unwrap();
    let rems = unpack_keyed_array(rems, CCA_REALM_EXTENSIBLE_MEASUREMENTS, "rems array")?;

    if rems.len() != CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS {
        return Err(TokenError::InvalidTokenFormat("wrong rems count"));
    }

    // zip rems (Value) and claims (Claim) to easily iterate together
    let rem_map = rems
        .into_iter()
        .zip(&mut attest_claims.realm_measurement_claims);

    for (rem, claim) in rem_map {
        get_claim(rem, claim)?;
    }

    Ok(())
}

fn verify_platform_token(attest_claims: &mut AttestationClaims) -> Result<(), TokenError>
{
    let platform_payload = attest_claims.plat_cose_sign1_wrapper[1].data.get_bstr();
    let val = de::from_reader(&platform_payload[..])?;
    let map = unpack_map(val, "platform token not a map")?;

    // main parsing
    let rest = get_claims_from_map(map, &mut attest_claims.plat_token_claims)?;

    // there should be one element left, sw components array
    if rest.len() != 1 {
        return Err(TokenError::InvalidTokenFormat("no sw components"));
    }

    let sw_components = rest.into_iter().next().unwrap();
    let sw_components =
        unpack_keyed_array(sw_components, CCA_PLAT_SW_COMPONENTS, "sw components array")?;

    if sw_components.len() > attest_claims.sw_component_claims.len() {
        return Err(TokenError::InvalidTokenFormat("too much sw components"));
    }

    // zip components (Value) and claims (SwComponent) to easily iterate together
    let sw_components_zipped = sw_components
        .into_iter()
        .zip(&mut attest_claims.sw_component_claims);

    for (sw_comp, sw_comp_claim) in sw_components_zipped {
        let map = unpack_map(sw_comp, "sw component not a map")?;
        let rest = get_claims_from_map(map, &mut sw_comp_claim.claims)?;
        if rest.len() != 0 {
            return Err(TokenError::InvalidTokenFormat(
                "sw component contains unrecognized claims",
            ));
        }
        sw_comp_claim.present = true;
    }

    Ok(())
}

fn verify_token_sign1(
    buf: &[u8],
    cose_sign1: &mut CoseSign1,
    cose_sign1_wrapper: &mut [Claim; CLAIM_COUNT_COSE_SIGN1_WRAPPER],
) -> Result<(), TokenError>
{
    let val = de::from_reader(buf)?;
    let data = unpack_tag(val, TAG_COSE_SIGN1, "cose sign1 tag")?;

    // unpack with CoseSign1 for the purpose of coset verification
    *cose_sign1 = CoseSign1::from_cbor_value(data.clone())?;

    // unpack manually using ciborium
    let vec = unpack_array(data, "cose sign1 not an array")?;

    if vec.len() != CLAIM_COUNT_COSE_SIGN1_WRAPPER + 1 {
        return Err(TokenError::InvalidTokenFormat(
            "wrong cose sign1 claim count",
        ));
    }

    let mut iter = vec.into_iter();

    // Protected header
    get_claim(iter.next().unwrap(), &mut cose_sign1_wrapper[0])?;
    // Unprotected header, map, may me empty (ignored)
    iter.next().unwrap();
    // Payload
    get_claim(iter.next().unwrap(), &mut cose_sign1_wrapper[1])?;
    // Signature
    get_claim(iter.next().unwrap(), &mut cose_sign1_wrapper[2])?;

    Ok(())
}

fn verify_cca_token(buf: &[u8]) -> Result<(Vec<u8>, Vec<u8>), TokenError>
{
    let val = de::from_reader(buf)?;
    let data = unpack_tag(val, TAG_CCA_TOKEN, "cca token tag")?;
    let map = unpack_map(data, "cca token not a map")?;

    if map.len() != 2 {
        return Err(TokenError::InvalidTokenFormat(
            "wrong realm/plat token count",
        ));
    }

    let mut iter = map.into_iter();
    let platform =
        unpack_keyed_bytes(iter.next().unwrap(), CCA_PLAT_TOKEN, "platform token bytes")?;
    let realm = unpack_keyed_bytes(
        iter.next().unwrap(),
        CCA_REALM_DELEGATED_TOKEN,
        "realm token bytes",
    )?;

    Ok((platform, realm))
}

pub fn verify_token(buf: &[u8]) -> Result<AttestationClaims, TokenError>
{
    let mut attest_claims = AttestationClaims::new();

    let (platform_token, realm_token) = verify_cca_token(&buf)?;

    verify_token_sign1(
        &realm_token,
        &mut attest_claims.realm_cose_sign1,
        &mut attest_claims.realm_cose_sign1_wrapper,
    )?;
    verify_token_sign1(
        &platform_token,
        &mut attest_claims.plat_cose_sign1,
        &mut attest_claims.plat_cose_sign1_wrapper,
    )?;

    verify_realm_token(&mut attest_claims)?;
    verify_platform_token(&mut attest_claims)?;

    let realm_key = attest_claims.realm_token_claims[4].data.get_bstr();
    verify_coset_signature(&attest_claims.realm_cose_sign1, realm_key, b"")?;

    //let platform_key = external_source();
    //verify_coset_signature(&attest_claims.plat_cose_sign1, platform_key, b"")?;

    Ok(attest_claims)
}

// RustCrypto verifier

fn verify_coset_signature(cose: &CoseSign1, key: &[u8], aad: &[u8]) -> Result<(), TokenError>
{
    if cose.protected.header.alg.is_none() {
        return Err(TokenError::InvalidAlgorithm(None));
    }
    let alg = cose
        .protected
        .header
        .alg
        .as_ref()
        .unwrap()
        .clone()
        .try_into()?;
    let verifier = RustCryptoVerifier::new(alg, &key);
    cose.verify_signature(aad, |sig, data| verifier.verify(sig, data))
}

enum SigningAlgorithm
{
    // sha256 + secp256r1/prime256v1/P-256
    ES256,
    // sha384 + secp384r1/P-384
    ES384,
    // sha512 + secp521r1/P-521
    ES512,
}

impl TryFrom<coset::Algorithm> for SigningAlgorithm
{
    type Error = TokenError;

    fn try_from(alg: coset::Algorithm) -> Result<Self, Self::Error>
    {
        match alg {
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES256) => {
                Ok(SigningAlgorithm::ES256)
            }
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES384) => {
                Ok(SigningAlgorithm::ES384)
            }
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES512) => {
                Ok(SigningAlgorithm::ES512)
            }
            unknown => Err(TokenError::InvalidAlgorithm(Some(unknown))),
        }
    }
}

struct RustCryptoVerifier
{
    algorithm: SigningAlgorithm,
    key_public_raw: Vec<u8>,
}

impl RustCryptoVerifier
{
    fn new(algorithm: SigningAlgorithm, key_public: &[u8]) -> Self
    {
        Self {
            algorithm,
            key_public_raw: key_public.to_vec(),
        }
    }

    fn verify(&self, sig: &[u8], data: &[u8]) -> Result<(), TokenError>
    {
        match self.algorithm {
            SigningAlgorithm::ES256 => {
                let key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&self.key_public_raw)?;
                let sig_hex = hex::encode(sig);
                let sig = p256::ecdsa::Signature::from_str(&sig_hex)?;
                key.verify(data, &sig)?;
            }
            SigningAlgorithm::ES384 => {
                let key = p384::ecdsa::VerifyingKey::from_sec1_bytes(&self.key_public_raw)?;
                let sig_hex = hex::encode(sig);
                let sig = p384::ecdsa::Signature::from_str(&sig_hex)?;
                key.verify(data, &sig)?;
            }
            SigningAlgorithm::ES512 => {
                // p521 from RustCrypto cannot do ecdsa
                return Err(TokenError::NotImplemented("p521 ecdsa"));
            }
        }
        Ok(())
    }
}
