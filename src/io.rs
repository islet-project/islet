/*
 * Copyright (c) 2022 Samsung Electronics Co., Ltd. All Rights Reserved.
 *
 * PROPRIETARY/CONFIDENTIAL
 * This software is the confidential and proprietary information of
 * Samsung Electronics Co., Ltd. ("Confidential Information").
 * You shall not disclose such Confidential Information and
 * shall use it only in accordance with the terms of the license agreement
 * you entered into with Samsung Electronics Co., Ltd. (“SAMSUNG”).
 * SAMSUNG MAKES NO REPRESENTATIONS OR WARRANTIES ABOUT
 * THE SUITABILITY OF THE SOFTWARE, EITHER EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE,
 * OR NON-INFRINGEMENT. SAMSUNG SHALL NOT BE LIABLE
 * FOR ANY DAMAGES SUFFERED BY LICENSEE AS A RESULT OF USING,
 * MODIFYING OR DISTRIBUTING THIS SOFTWARE OR ITS DERIVATIVES.
 */
#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    NotConnected,
    AlreadyExists,
    Unsupported,
    Other,
}

pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Device {
    fn initialize(&mut self) -> Result<()>;
    fn initialized(&self) -> bool;
}

pub trait Write {
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait ConsoleWriter: Device + Write {}

pub struct Stdout<'a> {
    device: Option<&'a mut dyn ConsoleWriter>,
}

impl<'a> Stdout<'a> {
    pub const fn new() -> Self {
        Self { device: None }
    }
    pub fn attach(&mut self, device: &'a mut dyn ConsoleWriter) -> Result<()> {
        if !device.initialized() {
            device.initialize()?;
        }
        self.device.replace(device);
        Ok(())
    }
}

impl<'a> Write for Stdout<'a> {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.device
            .as_mut()
            .map(|dev| dev.write_all(buf))
            .unwrap_or(Err(Error::new(ErrorKind::NotConnected)))
    }
}

//TODO Add lock and remove unsafe
pub unsafe fn stdout() -> &'static mut Stdout<'static> {
    static mut STDOUT: Stdout<'_> = Stdout::new();
    &mut STDOUT
}
