use crate::{error::AuctionError, InstructionAccounts, MAX_BUNDLE_VERIFIER_PAGES};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Debug)]
pub struct AuthorizeBundleDisputeEvidenceV5Accounts<'a, T> {
    pub submitter: &'a T,
    pub bundle_escrow: &'a T,
    pub bundle_verification_dispute: &'a T,
    pub instructions_sysvar: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for AuthorizeBundleDisputeEvidenceV5Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [submitter, bundle_escrow, bundle_verification_dispute, instructions_sysvar, ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self {
            submitter,
            bundle_escrow,
            bundle_verification_dispute,
            instructions_sysvar,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for AuthorizeBundleDisputeEvidenceV5Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        [
            self.submitter,
            self.bundle_escrow,
            self.bundle_verification_dispute,
            self.instructions_sysvar,
        ]
        .into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct AuthorizeBundleDisputeEvidenceV5Args {
    pub verification_hash: [u8; 32],
    pub page_hashes: [[u8; 32]; MAX_BUNDLE_VERIFIER_PAGES as usize],
    pub quorum_verifier_bitmap: u8,
    pub _reserved: [u8; 7],
}

/// Signed separately from the unchanged 232-byte finalization message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct BundleDisputeEvidenceV5Message {
    pub domain: [u8; 32],
    pub bundle_escrow: [u8; 32],
    pub settlement_deadline_slot: u64,
    pub replacement_deadline_slot: u64,
    pub verification_hash: [u8; 32],
    pub page_count: u8,
    pub _reserved: [u8; 7],
    pub page_hashes: [[u8; 32]; MAX_BUNDLE_VERIFIER_PAGES as usize],
}

impl BundleDisputeEvidenceV5Message {
    pub fn new(
        bundle_escrow: [u8; 32],
        settlement_deadline_slot: u64,
        replacement_deadline_slot: u64,
        page_count: u8,
        verification_hash: [u8; 32],
        page_hashes: [[u8; 32]; MAX_BUNDLE_VERIFIER_PAGES as usize],
    ) -> Self {
        Self {
            domain: *b"ambient.dispute.pages.v1\0\0\0\0\0\0\0\0",
            bundle_escrow,
            settlement_deadline_slot,
            replacement_deadline_slot,
            verification_hash,
            page_count,
            _reserved: [0; 7],
            page_hashes,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
