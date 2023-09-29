pub mod psa_serde;

use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    str::from_utf8,
    thread::sleep,
    time::Duration,
};

use islet_hes::{
    ECCFamily, HashAlgo, KeyBits, Measurement, MeasurementMetaData, MeasurementType, ValueHash,
    NUM_OF_MEASUREMENT_SLOTS, SW_TYPE_MAX_SIZE,
};

use self::psa_serde::{
    ExtendRequest, PSAError, PSARequest, PSAResponse, ReadRequest, PSA_MAX_IOVEC,
    RSS_DELEGATED_ATTEST_GET_DELEGATED_KEY, RSS_DELEGATED_ATTEST_GET_PLATFORM_TOKEN,
    RSS_DELEGATED_SERVICE_HANDLE, RSS_MEASURED_BOOT_EXTEND, RSS_MEASURED_BOOT_READ,
    RSS_MEASURED_BOOT_SERVICE_HANDLE, ReadResponse,
};

#[derive(Debug)]
pub enum CommsError {
    BufferTooSmall,
    InvalidArgument,
    GenericError,
    ProgrammerError,
    PsaSerdeError,
    CommunicationError,
    ServiceHandleError,
}

type ResponseParams = [usize; PSA_MAX_IOVEC];

struct MsgMetadata {
    pub protocol_ver: u8,
    pub seq_num: u8,
    pub client_id: u16,
    pub response_params: ResponseParams,
}

#[derive(Debug)]
pub enum Request {
    GetDAK(ECCFamily, KeyBits, HashAlgo),
    GetPlatformToken(ValueHash),
    // (index, sw_type_len, sw_version_len)
    ReadMeasurement(usize, usize, usize),
    ExtendMeasurement(usize, Measurement, bool),
}

#[derive(Debug)]
pub enum Response {
    GetDAK(Vec<u8>),
    GetPlatformToken(Vec<u8>),
    ReadMeasurement(Measurement, bool),
}

struct Channel {
    stream: Option<TcpStream>,
    addr: String,
}

pub struct CommsChannel {
    channel: Channel,
    msg_metadata: Option<MsgMetadata>,
}

impl From<std::io::Error> for CommsError {
    fn from(_value: std::io::Error) -> Self {
        CommsError::CommunicationError
    }
}

impl From<PSAError> for CommsError {
    fn from(_value: PSAError) -> Self {
        CommsError::PsaSerdeError
    }
}

impl CommsChannel {
    pub fn new(addr: String) -> Self {
        Self {
            channel: Channel {
                stream: None,
                addr,
            },
            msg_metadata: None,
        }
    }

    pub fn connect(&mut self, persistent: bool) -> Result<(), std::io::Error> {
        let stream = loop {
            match TcpStream::connect(&self.channel.addr) {
                Ok(stream) => break stream,
                Err(e) => {
                    if e.kind() == ErrorKind::ConnectionRefused && persistent {
                        const RECONNECT_SEC:u64 = 5;
                        println!("Couldn't connect, retrying in {} seconds...",
                                 RECONNECT_SEC);
                        sleep(Duration::from_secs(RECONNECT_SEC));
                        continue;
                    } else {
                        println!("Connection failed");
                        return Err(e);
                    }
                }
            }
        };
        println!("Connection established");
        self.channel.stream = Some(stream);
        Ok(())
    }

    fn convert_attestation_request(
        request_type: i16,
        in_vecs: &[Vec<u8>],
    ) -> Result<Request, CommsError> {
        match request_type {
            RSS_DELEGATED_ATTEST_GET_DELEGATED_KEY => {
                if in_vecs[0].len() != std::mem::size_of::<u8>()
                    || in_vecs[1].len() != std::mem::size_of::<u32>()
                    || in_vecs[2].len() != std::mem::size_of::<u32>()
                {
                    return Err(CommsError::GenericError);
                }

                // FIXME: Currently TF-A returns only zeroes for these parameters.

                // let request_ecc_family: u8 =
                //     u8::from_ne_bytes(psa_request.in_vecs[0].clone().try_into().unwrap());
                // let request_key_bits: u32 =
                //     u32::from_ne_bytes(psa_request.in_vecs[1].clone().try_into().unwrap());
                // let request_hash_algo: u32 =
                //     u32::from_ne_bytes(psa_request.in_vecs[1].clone().try_into().unwrap());

                // let ecc_family = match request_ecc_family {
                //     0x12 => ECCFamily::SecpR1,
                //     _ => return Err(CommsError::PsaInvalidArgument),
                // };

                // let key_bits = match request_key_bits {
                //     256 => KeyBits::Bits256,
                //     384 => KeyBits::Bits384,
                //     521 => return Err(CommsError::PsaInvalidArgument),
                //     _ => return Err(CommsError::PsaInvalidArgument),
                // };

                // let hash_algo = match request_hash_algo {
                //     0x2000009 => HashAlgo::Sha256,
                //     0x200000a => HashAlgo::Sha384,
                //     0x200000b => HashAlgo::Sha512,
                //     _ => return Err(CommsError::PsaInvalidArgument),
                // };

                // Ok(Request::GetDAK(ecc_family, key_bits, hash_algo))

                Ok(Request::GetDAK(
                    ECCFamily::SecpR1,
                    KeyBits::Bits384,
                    HashAlgo::Sha256,
                ))
            }
            RSS_DELEGATED_ATTEST_GET_PLATFORM_TOKEN => {
                let dak_pub_hash_size = in_vecs[0].len();
                if dak_pub_hash_size != HashAlgo::Sha256.len()
                    && dak_pub_hash_size != HashAlgo::Sha384.len()
                    && dak_pub_hash_size != HashAlgo::Sha512.len()
                {
                    println!("dak_pub_hash_size is invalid: {}", dak_pub_hash_size);
                    return Err(CommsError::InvalidArgument);
                }

                Ok(Request::GetPlatformToken(
                    in_vecs[0].iter().cloned().collect(),
                ))
            }
            _ => {
                // This doesn't happen in case of real psa
                Err(CommsError::ProgrammerError)
            }
        }
    }

    fn convert_measured_boot_request(
        request_type: i16,
        in_vecs: &[Vec<u8>],
    ) -> Result<Request, CommsError> {
        match request_type {
            RSS_MEASURED_BOOT_READ => {
                let read_request = ReadRequest::de(&in_vecs[0])?;
                let slot_id = read_request.index as usize;
                if slot_id >= NUM_OF_MEASUREMENT_SLOTS {
                    return Err(CommsError::InvalidArgument);
                }

                Ok(Request::ReadMeasurement(
                    slot_id,
                    read_request.sw_type_size as usize,
                    read_request.version_size as usize,
                ))
            }
            RSS_MEASURED_BOOT_EXTEND => {
                let extend_request = ExtendRequest::de(&in_vecs[0])?;
                let measurement_type = match extend_request.measurement_algo {
                    0x02000009 => MeasurementType::Sha256,
                    0x0200000a => MeasurementType::Sha384,
                    0x0200000b => MeasurementType::Sha512,
                    _ => {
                        println!(
                            "Unknown measurement type: {}",
                            extend_request.measurement_algo
                        );
                        return Err(CommsError::InvalidArgument);
                    }
                };

                if extend_request.sw_type_size as usize > SW_TYPE_MAX_SIZE {
                    return Err(CommsError::InvalidArgument);
                }

                let sw_type =
                    from_utf8(&extend_request.sw_type[..extend_request.sw_type_size as usize])
                        .unwrap()
                        .to_string();

                let signer_id: ValueHash = in_vecs[1].iter().cloned().collect();
                let sw_version = from_utf8(&in_vecs[2]).unwrap().to_string();
                let measurement_value: ValueHash = in_vecs[3].iter().cloned().collect();

                let measurement = Measurement {
                    metadata: MeasurementMetaData {
                        algorithm: measurement_type,
                        signer_id,
                        sw_version,
                        sw_type,
                    },
                    value: measurement_value,
                };

                Ok(Request::ExtendMeasurement(
                    extend_request.index as usize,
                    measurement,
                    extend_request.lock_measurement != 0,
                ))
            }
            _ => Err(CommsError::ProgrammerError),
        }
    }

    fn convert_request(&mut self, psa_request: PSARequest) -> Result<Request, CommsError> {
        self.msg_metadata = Some(MsgMetadata {
            protocol_ver: psa_request.protocol_ver,
            seq_num: psa_request.seq_num,
            client_id: psa_request.client_id,
            response_params: psa_request.out_lens,
        });

        let request = match psa_request.handle {
            RSS_DELEGATED_SERVICE_HANDLE => {
                Self::convert_attestation_request(psa_request.psa_type, &psa_request.in_vecs)?
            }
            RSS_MEASURED_BOOT_SERVICE_HANDLE => {
                Self::convert_measured_boot_request(psa_request.psa_type, &psa_request.in_vecs)?
            }
            _ => {
                println!("Unknown service handle: {}", psa_request.handle);
                return Err(CommsError::ServiceHandleError);
            }
        };

        Ok(request)
    }

    pub(crate) fn read_stream(stream: &mut TcpStream) -> std::io::Result<Option<Vec<u8>>> {
        stream.set_read_timeout(None)?;

        let mut data = [0u8; 0x1000];
        let mut count = stream.read(&mut data)?;
        if count == 0 {
            return Ok(None);
        }

        /* ugly, but should work */
        stream.set_read_timeout(Some(Duration::from_millis(50)))?;
        loop {
            let result = stream.read(&mut data[count..]);
            if let Err(e) = &result {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
            }
            let left = result?;
            if left > 0 {
                count = count + left;
            } else {
                break;
            }
        }

        stream.set_read_timeout(None)?;

        Ok(Some(data[..count].to_vec()))
    }

    pub fn get_request(&mut self) -> Result<Option<Request>, CommsError> {
        let mut stream = self.channel.stream.as_mut().unwrap();
        let data = match Self::read_stream(&mut stream)? {
            Some(data) => data,
            None => return Ok(None),
        };

        let psa_request = PSARequest::de(&data)?;

        Ok(self.convert_request(psa_request.clone()).map(|r| Some(r))?)
    }

    fn convert_response(
        &mut self,
        return_val: i32,
        response: Option<Response>,
    ) -> Result<PSAResponse, CommsError> {
        let msg_metadata = self.msg_metadata.take().unwrap();
        let mut out_vecs = <[Vec<u8>; PSA_MAX_IOVEC]>::default();

        if return_val == 0 {
            match response {
                Some(Response::GetDAK(dak)) => {
                    if dak.len() > msg_metadata.response_params[0] {
                        return Err(CommsError::BufferTooSmall);
                    }
                    out_vecs[0] = dak;
                }
                Some(Response::GetPlatformToken(token)) => {
                    if token.len() > msg_metadata.response_params[0] {
                        return Err(CommsError::BufferTooSmall);
                    }
                    out_vecs[0] = token;
                }
                Some(Response::ReadMeasurement(measurement, is_locked)) => {
                    if measurement.metadata.signer_id.len() > msg_metadata.response_params[1] || measurement.value.len() > msg_metadata.response_params[2] {
                        return Err(CommsError::BufferTooSmall);
                    }
                    let read_response = ReadResponse {
                        is_locked: is_locked as u8,
                        measurement_algo: match measurement.metadata.algorithm {
                            MeasurementType::Sha256 => 0x02000009,
                            MeasurementType::Sha384 => 0x0200000a,
                            MeasurementType::Sha512 => 0x0200000b,
                        },
                        sw_type: measurement.metadata.sw_type.as_bytes().try_into().unwrap(),
                        sw_type_len: measurement.metadata.sw_type.len() as u8,
                        version: measurement.metadata.sw_version.as_bytes().try_into().unwrap(),
                        version_len: measurement.metadata.sw_version.len() as u8,
                    };

                    out_vecs[0] = read_response.ser()?;
                    out_vecs[1] = measurement.metadata.signer_id.to_vec();
                    out_vecs[2] = measurement.value.to_vec();
                }
                None => {
                    if msg_metadata.response_params[0] != 0 {
                        println!("No response but client expected some");
                        return Err(CommsError::ProgrammerError);
                    }
                }
            }
        }

        Ok(PSAResponse {
            out_vecs,
            client_id: msg_metadata.client_id,
            protocol_ver: msg_metadata.protocol_ver,
            return_val,
            seq_num: msg_metadata.seq_num,
        })
    }

    pub fn send_response(
        &mut self,
        ret_val: i32,
        response: Option<Response>,
    ) -> Result<(), CommsError> {
        let psa_response = self.convert_response(ret_val, response)?;

        self.channel
            .stream
            .as_ref()
            .unwrap()
            .write_all(&psa_response.ser()?)?;

        Ok(())
    }
}
