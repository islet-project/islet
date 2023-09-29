mod comms;

use islet_hes::{
    BootMeasurement, BootMeasurementMetadata, IsletHES, IsletHESError, HWAsymmetricKey,
    HWData, HWHash, HWSymmetricKey,
};
use clap::Parser;
use comms::{CommsChannel, CommsError, Request, Response};
use coset::TaggedCborSerializable;
use daemonize::Daemonize;
use std::fs::{self, File};
use std::io::{Read, Result as IOResult};
use tinyvec::ArrayVec;

use crate::comms::psa_serde::PSA_SUCCESS;

/// Creates a path to a resource file
macro_rules! resource_file {
    ($fname:expr) => {
        // Ugly way to base path on workspace directory
        concat!(env!("CARGO_MANIFEST_DIR"), "/../res/", $fname)
    };
}

/// Islet HES Host App providing measured_boot and attestation functionalities
#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
#[command(about = "Islet HES Host App providing measured_boot and attestation functionalities")]
struct Args {
    /// Path to binary file with BL2 hash
    #[arg(short = 'b', long, value_name = "FILE")]
    #[arg(default_value = resource_file!("bl2_signed_hash.bin"))]
    hash_file: Option<String>,

    /// Path to binary file with GUK
    #[arg(short, long, value_name = "FILE")]
    #[arg(default_value = resource_file!("dummy_guk.bin"))]
    guk_file: Option<String>,

    /// Address of TF-A telnet socket
    #[arg(short, long, value_name = "IP:PORT")]
    #[arg(default_value = "127.0.0.1:5002")]
    addr: String,

    /// Keep reconnecting, when connection is not yet established or is
    /// shut down
    #[arg(short, long)]
    persistent: bool,

    /// Whether to daemonize the app, implies 'persistent'
    #[arg(short, long)]
    daemonize: bool,

    /// Daemonize working directory, pid file and logs will be placed here
    #[arg(short = 'r', long, default_value = "/tmp")]
    daemonize_root: String,
}

fn load_binary_file(filename: &str) -> IOResult<Vec<u8>> {
    let mut f = File::open(filename)?;
    let metadata = fs::metadata(filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

// Fix me - gather all of the numbers of respective psa errors
fn islet_hes_error_to_ret_val(error: IsletHESError) -> i32 {
    match error {
        // PSA_ERROR_BAD_STATE
        IsletHESError::BadState => -137,
        // PSA_ERROR_DOES_NOT_EXIST
        IsletHESError::DoesNotExist => -140,
        // PSA_ERROR_GENERIC_ERROR
        IsletHESError::GenericError => -132,
        // PSA_ERROR_INVALID_ARGUMENT
        IsletHESError::InvalidArgument => -135,
        // PSA_ERROR_NOT_PERMITTED
        IsletHESError::NotPermitted => -133,
        // PSA_ERROR_NOT_SUPPORTED
        IsletHESError::NotSupported => -134,
    }
}

fn comms_error_to_ret_val(error: CommsError) -> i32 {
    match error {
        // PSA_ERROR_BUFFER_TOO_SMALL
        CommsError::BufferTooSmall => -138,
        // PSA_ERROR_GENERIC_ERROR
        CommsError::GenericError => -132,
        // PSA_ERROR_INVALID_ARGUMENT
        CommsError::InvalidArgument => -135,
        // PSA_ERROR_PROGRAMMER_ERROR
        CommsError::ProgrammerError => -129,
        // Internal error codes
        CommsError::CommunicationError => {
            panic!("Communication error cannot be translated as a return value!")
        }
        CommsError::PsaSerdeError => {
            panic!("PSA (de)serialization error cannot be translated as a return value!")
        }
        CommsError::ServiceHandleError => {
            panic!("Service handle error cannot be translated as a return value!")
        }
    }
}

fn process_requests(comms: &mut CommsChannel, islet_hes: &mut IsletHES) -> Result<(), CommsError> {
    println!("Processing connection requests");
    loop {
        let (ret_val, response) = match comms.get_request() {
            Ok(requests) => {
                if requests.is_none() {
                    // Server disconnected gracefully
                    return Ok(());
                }
                println!("Request read: {:?}", requests);
                let request = requests.unwrap();
                match request {
                    Request::GetDAK(ecc_family, key_bits, hash_algo) => {
                        match islet_hes.get_delegated_key(ecc_family, key_bits, hash_algo) {
                            Ok(dak) => (PSA_SUCCESS, Some(Response::GetDAK(dak))),
                            Err(e) => (islet_hes_error_to_ret_val(e), None),
                        }
                    }
                    Request::GetPlatformToken(dak_pub_hash) => {
                        match islet_hes.get_platform_token(&dak_pub_hash) {
                            Ok(token) => (
                                PSA_SUCCESS,
                                Some(Response::GetPlatformToken(token.to_tagged_vec().unwrap())),
                            ),
                            Err(e) => (islet_hes_error_to_ret_val(e), None),
                        }
                    }
                    Request::ExtendMeasurement(slot_id, measurement, lock) => {
                        match islet_hes.extend_measurement(slot_id, measurement, lock) {
                            Ok(()) => (PSA_SUCCESS, None),
                            Err(e) => (islet_hes_error_to_ret_val(e), None),
                        }
                    }
                    Request::ReadMeasurement(index, sw_type_len, version_len) => {
                        match islet_hes.read_measurement(index as usize) {
                            Ok((measurement, is_locked)) => {
                                if sw_type_len < measurement.metadata.sw_type.len()
                                    || version_len < measurement.metadata.sw_version.len()
                                {
                                    (islet_hes_error_to_ret_val(IsletHESError::InvalidArgument), None)
                                } else {
                                    (PSA_SUCCESS, Some(Response::ReadMeasurement(measurement.clone(), is_locked)))
                                }
                            }
                            Err(e) => (islet_hes_error_to_ret_val(e), None),
                        }
                    }
                }
            }
            Err(e) => {
                let ret_val = match e {
                    CommsError::CommunicationError => return Err(e),
                    CommsError::PsaSerdeError => {
                        panic!("Psa (de)serialization failed!");
                    }
                    CommsError::ServiceHandleError => {
                        println!("Got request not intended for us, ignoring...");
                        continue;
                    }
                    CommsError::ProgrammerError => {
                        panic!("Now go fix your code. Tut tut tut!\n");
                    }
                    e => comms_error_to_ret_val(e),
                };

                (ret_val, None)
            }
        };

        println!("Sending response: {:?}", response);
        match comms.send_response(ret_val, response) {
            Ok(()) => (),
            Err(e) => match e {
                CommsError::CommunicationError => return Err(e),
                CommsError::PsaSerdeError => {
                    panic!("Psa (de)serialization failed!");
                }
                _ => panic!("No other error possible"),
            },
        }
    }
}

fn daemonize(daemonize_root: String) -> std::io::Result<()>{
    let root = std::fs::canonicalize(daemonize_root)?;
    let hespid = root.join("hes.pid").to_string_lossy().to_string();
    let hesout = root.join("hes.out").to_string_lossy().to_string();
    let heserr = root.join("hes.err").to_string_lossy().to_string();
    let root = root.to_string_lossy().to_string();

    let stdout = File::create(&hesout).unwrap();
    let stderr = File::create(&heserr).unwrap();

    println!("Pidfile: {}, logs: {}, {}", hespid, hesout, heserr);

    let daemonize = Daemonize::new()
        .pid_file(hespid)
        .working_directory(root)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => println!("Daemonization succeessful"),
        Err(e) => panic!("Daemonization failed: {}", e),
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut args = Args::parse();

    if args.daemonize {
        println!("Daemonization requested");

        // daemon without persistent makes little sense
        args.persistent = true;

        daemonize(args.daemonize_root)?;
    }

    let bl_hash = match &args.hash_file {
        Some(path) => Some(load_binary_file(path)?.iter().cloned().collect()),
        None => None,
    };

    let guk = match &args.guk_file {
        Some(path) => Some(load_binary_file(path)?.iter().cloned().collect()),
        None => None,
    };

    let hw_data = DummyHW::init(guk, bl_hash);

    let mut islet_hes = IsletHES::init(hw_data.clone()).unwrap();
    let mut comms = CommsChannel::new(args.addr);

    let is_persistent = args.persistent;
    comms.connect(is_persistent)?;

    loop {
        match process_requests(&mut comms, &mut islet_hes) {
            // Graceful disconnection
            Ok(()) => {
                if is_persistent {
                    islet_hes.reset(hw_data.clone()).unwrap();
                    println!("Connection was shut down, reconnecting...");
                    comms.connect(true)?;
                    continue;
                } else {
                    return Ok(());
                }
            },
            // Some connection error
            Err(_) => {
                if is_persistent {
                    islet_hes.reset(hw_data.clone()).unwrap();
                    println!("Connection failed, reconnecting...");
                    comms.connect(true)?;
                    continue;
                } else {
                    return Err(std::io::Error::last_os_error());
                }
            }
        }
    }
}

// ================================ DUMMY DATA ===================================

#[derive(Debug, Clone)]
struct DummyHW {
    guk: Option<HWSymmetricKey>,
    bl_hash: Option<HWHash>,
}

type DummyError = ();

impl DummyHW {
    fn init(guk: Option<HWSymmetricKey>, bl_hash: Option<HWHash>) -> Self {
        Self { guk, bl_hash }
    }
}

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
        Ok(self.guk.unwrap_or(
            [
                0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67,
                0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45,
                0x67, 0x89, 0x01, 0x23,
            ]
            .into(),
        ))
    }

    // Equivalent to res/bl2_signed_hash.bin
    fn bl_hash(&self) -> Result<HWHash, Self::Error> {
        Ok(self.bl_hash.unwrap_or(
            [
                0xf1, 0x5f, 0x95, 0x3b, 0xe5, 0x0d, 0xad, 0x92, 0xc3, 0xb2, 0xaa, 0x32, 0x97, 0xe6,
                0xa4, 0xa8, 0xd6, 0x6d, 0x33, 0x63, 0x84, 0x49, 0xec, 0x19, 0x22, 0xb4, 0xa7, 0x92,
                0x4a, 0x7b, 0x30, 0x22,
            ]
            .iter()
            .cloned()
            .collect(),
        ))
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
