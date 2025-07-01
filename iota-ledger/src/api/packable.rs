// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use std::io::{Read, Write};
use std::str;

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // to have "green" files ;-)
pub enum Error {
    #[error("I/O error happened: {0}.")]
    Io(#[from] std::io::Error),
    #[error("Invalid variant read.")]
    InvalidVariant,
    #[error("Invalid data read.")]
    InvalidData,
    #[error("Invalid Utf8 string read.")]
    InvalidUtf8String,
    #[error("Invalid announced len.")]
    InvalidAnnouncedLen,
    #[error("String too long.")]
    StringTooLong,
}

pub trait Packable {
    fn packed_len(&self) -> usize;

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), Error>;
}

pub trait PackableObject {
    fn pack_as_vec(&self) -> Result<Vec<u8>, Error>;
}

// Blanket implementation for all types that implement Packable
impl<T: Packable> PackableObject for T {
    fn pack_as_vec(&self) -> Result<Vec<u8>, Error> {
        let mut vec = Vec::with_capacity(self.packed_len());
        self.pack(&mut vec)?;
        Ok(vec)
    }
}

pub trait Unpackable {
    fn unpack<R: Read>(buf: &mut R) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Packable for () {
    fn packed_len(&self) -> usize {
        0
    }

    fn pack<W: Write>(&self, _buf: &mut W) -> Result<(), Error> {
        Ok(())
    }
}

impl Unpackable for () {
    fn unpack<R: Read>(_buf: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}

macro_rules! impl_packable_for_num {
    ($ty:ident) => {
        impl Packable for $ty {
            fn packed_len(&self) -> usize {
                std::mem::size_of::<$ty>()
            }

            fn pack<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
                buf.write_all(self.to_le_bytes().as_ref())?;
                Ok(())
            }
        }
        impl Unpackable for $ty {
            fn unpack<R: Read>(buf: &mut R) -> Result<Self, Error> {
                let mut bytes = [0; std::mem::size_of::<$ty>()];
                buf.read_exact(&mut bytes)?;
                Ok($ty::from_le_bytes(bytes))
            }
        }
    };
}

impl Packable for String {
    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.chars().count()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), Error> {
        if self.chars().count() > 255 {
            return Err(Error::StringTooLong);
        }
        let bytes = self.clone().into_bytes();
        (bytes.len() as u8).pack(buf)?;
        buf.write_all(&bytes)?;
        Ok(())
    }
}

impl Unpackable for String {
    fn unpack<R: Read>(buf: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let l = u8::unpack(buf)?;
        let mut v: Vec<u8> = Vec::new();
        for _ in 0..l {
            v.push(u8::unpack(buf)?);
        }
        match str::from_utf8(v.as_ref()) {
            Ok(v) => Ok(String::from(v)),
            Err(_) => Err(Error::InvalidUtf8String),
        }
    }
}

impl_packable_for_num!(i8);
impl_packable_for_num!(u8);
impl_packable_for_num!(i16);
impl_packable_for_num!(u16);
impl_packable_for_num!(i32);
impl_packable_for_num!(u32);
impl_packable_for_num!(i64);
impl_packable_for_num!(u64);
impl_packable_for_num!(i128);
impl_packable_for_num!(u128);
