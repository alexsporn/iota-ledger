use crate::api::packable::{Error as PackableError, Read, Unpackable};

use crate::Transport;

use crate::api::constants::APDUInstructions;
use crate::api::{errors, helpers};
use crate::packable_vec;

use fastcrypto::{
    ed25519::{ED25519_PUBLIC_KEY_LENGTH, Ed25519PublicKey},
    hash::Digest,
    traits::ToFromBytes,
};

pub struct PublicKeyResult {
    pub public_key: Ed25519PublicKey,
    pub address: Digest<32>,
}

impl Unpackable for PublicKeyResult {
    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        if u8::unpack(buf)? != ED25519_PUBLIC_KEY_LENGTH as u8 {
            return Err(PackableError::InvalidAnnouncedLen);
        }
        let mut key = [0_u8; ED25519_PUBLIC_KEY_LENGTH];
        buf.read_exact(&mut key)?;
        let public_key =
            Ed25519PublicKey::from_bytes(&key).map_err(|_| PackableError::InvalidData)?;

        if u8::unpack(buf)? != 32 {
            return Err(PackableError::InvalidAnnouncedLen);
        }
        let mut address_buffer = [0_u8; 32];
        buf.read_exact(&mut address_buffer)?;
        let address = Digest::<32>::new(address_buffer);

        Ok(Self {
            public_key,
            address,
        })
    }
}

pub fn exec(
    transport: &Transport,
    bip32: &bip32::DerivationPath,
    show: bool,
) -> Result<PublicKeyResult, errors::APIError> {
    let payload: helpers::PackedBIP32Path = bip32.into();
    let ins = if show {
        APDUInstructions::VerifyAddress
    } else {
        APDUInstructions::GetPublicKey
    };
    helpers::send_with_blocks(transport, ins, packable_vec![payload], None)
}
