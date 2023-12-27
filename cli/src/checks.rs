use crate::{tools, GenericResult};
use colored::Colorize;

// HELPER FUNCTIONS

fn start(positive: bool, name: &str)
{
    print!("TEST CASE: ");
    if positive {
        print!("{}", "POS: ".blue());
    } else {
        print!("{}", "NEG: ".magenta());
    }
    print!("\"{}\": ", name.yellow());
}

fn success() -> GenericResult
{
    println!("{}", "PASSED".green());
    Ok(())
}

fn check_len<T>(v: &Vec<T>, len: usize) -> GenericResult
{
    if v.len() != len {
        let msg = format!("Wrong length, got: {}, expected: {}", v.len(), len);
        return Err(msg.into());
    }
    Ok(())
}

fn expect_err<V, E: std::error::Error>(res: Result<V, E>, err: &str) -> GenericResult
{
    match res {
        Ok(_) => Err("Expected error, OK returned".into()),
        Err(e) => {
            if e.to_string() != err {
                let msg = format!(
                    "Wrong error, got: \"{}\", expected: \"{}\"",
                    e.to_string(),
                    err
                );
                Err(msg.into())
            } else {
                Ok(())
            }
        }
    }
}

// TEST CASES

fn test_rsi_api() -> GenericResult
{
    start(true, "RSI API basic test");
    rsi_el0::abi_version()?;
    success()
}

fn test_positive_read_measurement_rim() -> GenericResult
{
    start(true, "read_measurement RIM, check length");
    let data = rsi_el0::measurement_read(0)?;
    check_len(&data, rsi_el0::MAX_MEASUR_LEN as usize)?;
    success()
}

fn test_positive_read_measurement_rems() -> GenericResult
{
    start(true, "read_measurement REMs, check length");
    let data = rsi_el0::measurement_read(1)?;
    check_len(&data, rsi_el0::MAX_MEASUR_LEN as usize)?;
    let data = rsi_el0::measurement_read(4)?;
    check_len(&data, rsi_el0::MAX_MEASUR_LEN as usize)?;
    success()
}

fn test_negative_read_measurement_index() -> GenericResult
{
    start(false, "read_measurement, wrong index");
    expect_err(rsi_el0::measurement_read(5), "EFAULT: Bad address")?;
    success()
}

fn test_positive_extend_measurement_basic() -> GenericResult
{
    start(true, "extend_measurement, basic");
    let extend = tools::random_data(rsi_el0::MAX_MEASUR_LEN as usize);
    rsi_el0::measurement_extend(1, &extend)?;
    success()
}

fn test_positive_extend_measurement_check() -> GenericResult
{
    start(true, "extend_measurement, verify change");
    for index in [1, 4] {
        let data = rsi_el0::measurement_read(index)?;
        let extend = tools::random_data(rsi_el0::MAX_MEASUR_LEN as usize);
        rsi_el0::measurement_extend(index, &extend)?;
        let new_data = rsi_el0::measurement_read(index)?;
        if data == new_data {
            return Err("Measurement did not change with extend".into());
        }
    }
    success()
}

fn test_negative_extend_measurement_index() -> GenericResult
{
    start(false, "extend_measurement, wrong index");
    let extend = tools::random_data(rsi_el0::MAX_MEASUR_LEN as usize);
    expect_err(
        rsi_el0::measurement_extend(5, &extend),
        "EFAULT: Bad address",
    )?;
    success()
}

fn test_positive_attestation_get_token() -> GenericResult
{
    start(true, "attestation, get token");
    let challenge = tools::random_data(rsi_el0::CHALLENGE_LEN as usize);
    let _token = rsi_el0::attestation_token(&challenge.try_into().unwrap())?;
    success()
}

fn test_positive_attestation_verify_token() -> GenericResult
{
    start(true, "attestation, verify token");
    let challenge = tools::random_data(rsi_el0::CHALLENGE_LEN as usize);
    let token = rsi_el0::attestation_token(&challenge.try_into().unwrap())?;
    let _claims = cca_token::verifier::verify_token(&token)?;
    success()
}

pub fn run_tests(_verbose: bool) -> GenericResult
{
    test_rsi_api()?;

    test_positive_read_measurement_rim()?;
    test_positive_read_measurement_rems()?;
    test_negative_read_measurement_index()?;

    test_positive_extend_measurement_basic()?;
    test_positive_extend_measurement_check()?;
    test_negative_extend_measurement_index()?;

    test_positive_attestation_get_token()?;
    test_positive_attestation_verify_token()?;

    Ok(())
}
