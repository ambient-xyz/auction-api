use crate::constant::PUBKEY_BYTES;
use crate::Pubkey;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::Serialize;

pub const MAX_TEE_VERIFIERS: usize = 10;

#[derive(Pod, Clone, Copy, Zeroable, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(C)]
pub struct TeeVerifierList {
    pub len: u64,
    pub tee_verifiers: [Pubkey; MAX_TEE_VERIFIERS],
}

impl TeeVerifierList {
    pub const LEN: usize = std::mem::size_of::<TeeVerifierList>();

    pub fn from_bytes_mut(data: &mut [u8]) -> Option<&mut Self> {
        let bytes = data.get_mut(..Self::LEN)?;
        bytemuck::try_from_bytes_mut::<Self>(bytes).ok()
    }

    pub fn contains(&self, key: [u8; PUBKEY_BYTES]) -> bool {
        let Some(len) = self.valid_len() else {
            return false;
        };
        self.tee_verifiers[..len]
            .iter()
            .any(|existing| existing.inner() == key)
    }

    pub fn valid_len(&self) -> Option<usize> {
        let len = usize::try_from(self.len).ok()?;
        (len <= MAX_TEE_VERIFIERS).then_some(len)
    }
}
