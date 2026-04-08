use super::{
    AccountDiscriminator, AccountHeaderV1, AccountLayoutVersion, ParsedAccountLayout, Pubkey,
    RequestTier,
};
use crate::VERIFIERS_PER_AUCTION;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Pod, Clone, Copy, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct RawBundleEscrowV2Data {
    pub status: BundleEscrowV2Status,
    pub reward_tier: RequestTier,
    pub coordinator: Pubkey,
    pub requester_refund_recipient: Pubkey,
    pub bundle_version: u32,
    pub _reserved0: [u8; 4],
    pub bundle_hash: [u8; 32],
    pub total_input_tokens: u64,
    pub max_output_tokens: u64,
    pub winner_node_pubkey: Pubkey,
    pub winner_vote_account: Pubkey,
    pub clearing_price_per_output_token: u64,
    pub selected_verifiers: [Pubkey; VERIFIERS_PER_AUCTION],
    pub auction_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub verification_hash: [u8; 32],
    pub posted_output_tokens: u64,
    pub accepted_output_tokens: u64,
    pub settlement_deadline_slot: u64,
    pub result_deadline_slot: u64,
    pub verification_deadline_slot: u64,
    pub claim_deadline_slot: u64,
    pub winner_reward_claimed: u8,
    pub verifier_reward_claimed_bitmap: u8,
    pub quorum_verifier_bitmap: u8,
    pub verifier_page_count: u8,
    pub _reserved1: [u8; 4],
    pub verifier_reward_remaining: [u64; VERIFIERS_PER_AUCTION],
}

pub type BundleEscrowV2 = RawBundleEscrowV2Data;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBundleEscrowV2Transition {
    pub from: BundleEscrowV2Status,
    pub to: BundleEscrowV2Status,
}

impl InvalidBundleEscrowV2Transition {
    const fn new(from: BundleEscrowV2Status, to: BundleEscrowV2Status) -> Self {
        Self { from, to }
    }
}

#[derive(Debug)]
pub struct BundleEscrowV2Ref<'a> {
    header: &'a AccountHeaderV1,
    raw: &'a RawBundleEscrowV2Data,
}

#[derive(Debug)]
pub struct BundleEscrowV2Mut<'a> {
    header: &'a mut AccountHeaderV1,
    raw: &'a mut RawBundleEscrowV2Data,
}

impl<'a> BundleEscrowV2Ref<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleEscrowV2Data {
        self.raw
    }

    pub fn into_raw(self) -> &'a RawBundleEscrowV2Data {
        self.raw
    }
}

impl Deref for BundleEscrowV2Ref<'_> {
    type Target = RawBundleEscrowV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> BundleEscrowV2Mut<'a> {
    pub fn header(&self) -> &AccountHeaderV1 {
        self.header
    }

    pub fn layout(&self) -> ParsedAccountLayout {
        self.header.layout().unwrap()
    }

    pub fn as_raw(&self) -> &RawBundleEscrowV2Data {
        self.raw
    }

    pub fn as_raw_mut(&mut self) -> &mut RawBundleEscrowV2Data {
        self.raw
    }

    pub fn into_raw(self) -> &'a mut RawBundleEscrowV2Data {
        self.raw
    }
}

impl Deref for BundleEscrowV2Mut<'_> {
    type Target = RawBundleEscrowV2Data;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl DerefMut for BundleEscrowV2Mut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw
    }
}

impl RawBundleEscrowV2Data {
    pub const PAYLOAD_LEN: usize = std::mem::size_of::<RawBundleEscrowV2Data>();
    pub const LEN: usize = AccountHeaderV1::LEN + Self::PAYLOAD_LEN;

    pub fn from_bytes(bytes: &[u8]) -> Option<BundleEscrowV2Ref<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes::<AccountHeaderV1>(header_bytes).ok()?;
        if header.layout()
            != Some(ParsedAccountLayout::new(
                AccountDiscriminator::BundleEscrowV2,
                AccountLayoutVersion::V1,
            ))
        {
            return None;
        }

        let raw = bytemuck::try_from_bytes::<RawBundleEscrowV2Data>(raw_bytes).ok()?;
        Some(BundleEscrowV2Ref { header, raw })
    }

    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<BundleEscrowV2Mut<'_>> {
        if bytes.len() != Self::LEN {
            return None;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        let header = bytemuck::try_from_bytes_mut::<AccountHeaderV1>(header_bytes).ok()?;
        if header.layout()
            != Some(ParsedAccountLayout::new(
                AccountDiscriminator::BundleEscrowV2,
                AccountLayoutVersion::V1,
            ))
        {
            return None;
        }

        let raw = bytemuck::try_from_bytes_mut::<RawBundleEscrowV2Data>(raw_bytes).ok()?;
        Some(BundleEscrowV2Mut { header, raw })
    }

    pub fn read(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes).map(|account| *account.as_raw())
    }

    pub fn write_v1_bytes(&self, bytes: &mut [u8]) -> bool {
        if bytes.len() != Self::LEN {
            return false;
        }

        let (header_bytes, raw_bytes) = bytes.split_at_mut(AccountHeaderV1::LEN);
        header_bytes.copy_from_slice(bytemuck::bytes_of(&AccountHeaderV1::new(
            AccountDiscriminator::BundleEscrowV2,
        )));
        raw_bytes.copy_from_slice(bytemuck::bytes_of(self));
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn award(
        &mut self,
        auction_hash: [u8; 32],
        winner_node_pubkey: Pubkey,
        winner_vote_account: Pubkey,
        clearing_price_per_output_token: u64,
        selected_verifiers: [Pubkey; VERIFIERS_PER_AUCTION],
    ) -> Result<(), InvalidBundleEscrowV2Transition> {
        if self.status != BundleEscrowV2Status::Open {
            return Err(InvalidBundleEscrowV2Transition::new(
                self.status,
                BundleEscrowV2Status::Awarded,
            ));
        }

        self.auction_hash = auction_hash;
        self.winner_node_pubkey = winner_node_pubkey;
        self.winner_vote_account = winner_vote_account;
        self.clearing_price_per_output_token = clearing_price_per_output_token;
        self.selected_verifiers = selected_verifiers;
        self.status = BundleEscrowV2Status::Awarded;

        Ok(())
    }

    pub fn post_result(
        &mut self,
        result_hash: [u8; 32],
        posted_output_tokens: u64,
    ) -> Result<(), InvalidBundleEscrowV2Transition> {
        if self.status != BundleEscrowV2Status::Awarded {
            return Err(InvalidBundleEscrowV2Transition::new(
                self.status,
                BundleEscrowV2Status::ResultPosted,
            ));
        }

        self.result_hash = result_hash;
        self.posted_output_tokens = posted_output_tokens;
        self.status = BundleEscrowV2Status::ResultPosted;

        Ok(())
    }

    pub fn finalize(
        &mut self,
        final_status: BundleEscrowV2Status,
        verification_hash: [u8; 32],
        accepted_output_tokens: u64,
        quorum_verifier_bitmap: u8,
        verifier_page_count: u8,
        verifier_reward_remaining: [u64; VERIFIERS_PER_AUCTION],
    ) -> Result<(), InvalidBundleEscrowV2Transition> {
        if self.status != BundleEscrowV2Status::ResultPosted {
            return Err(InvalidBundleEscrowV2Transition::new(
                self.status,
                final_status,
            ));
        }

        match final_status {
            BundleEscrowV2Status::FinalizedVerified | BundleEscrowV2Status::FinalizedRejected => {}
            _ => {
                return Err(InvalidBundleEscrowV2Transition::new(
                    self.status,
                    final_status,
                ));
            }
        }

        self.verification_hash = verification_hash;
        self.accepted_output_tokens = accepted_output_tokens;
        self.quorum_verifier_bitmap = quorum_verifier_bitmap;
        self.verifier_page_count = verifier_page_count;
        self.verifier_reward_remaining = verifier_reward_remaining;
        self.status = final_status;

        Ok(())
    }

    pub fn claim_verifier_reward(&mut self, verifier_index: usize, claimed_amount: u64) -> bool {
        let remaining = self.verifier_reward_remaining[verifier_index]
            .checked_sub(claimed_amount)
            .expect("verifier reward underflow");
        self.verifier_reward_remaining[verifier_index] = remaining;
        debug_assert!(verifier_index < 8);
        if remaining == 0 {
            self.verifier_reward_claimed_bitmap |= 1u8 << verifier_index;
        }
        remaining == 0
    }

    pub fn expire(&mut self) -> Result<(), InvalidBundleEscrowV2Transition> {
        match self.status {
            BundleEscrowV2Status::Open
            | BundleEscrowV2Status::Awarded
            | BundleEscrowV2Status::ResultPosted => {
                self.status = BundleEscrowV2Status::Expired;
                Ok(())
            }
            status => Err(InvalidBundleEscrowV2Transition::new(
                status,
                BundleEscrowV2Status::Expired,
            )),
        }
    }

    pub fn all_quorum_verifier_rewards_claimed(&self) -> bool {
        self.verifier_reward_claimed_bitmap & self.quorum_verifier_bitmap
            == self.quorum_verifier_bitmap
    }

    pub fn final_reward_claims_complete(&self) -> bool {
        let winner_done = self.status != BundleEscrowV2Status::FinalizedVerified
            || self.winner_reward_claimed != 0;
        winner_done && self.all_quorum_verifier_rewards_claimed()
    }
}

impl Default for RawBundleEscrowV2Data {
    fn default() -> Self {
        Self {
            status: BundleEscrowV2Status::Open,
            reward_tier: RequestTier::Eco,
            coordinator: Pubkey::default(),
            requester_refund_recipient: Pubkey::default(),
            bundle_version: 0,
            _reserved0: [0; 4],
            bundle_hash: [0; 32],
            total_input_tokens: 0,
            max_output_tokens: 0,
            winner_node_pubkey: Pubkey::default(),
            winner_vote_account: Pubkey::default(),
            clearing_price_per_output_token: 0,
            selected_verifiers: [Pubkey::default(); VERIFIERS_PER_AUCTION],
            auction_hash: [0; 32],
            result_hash: [0; 32],
            verification_hash: [0; 32],
            posted_output_tokens: 0,
            accepted_output_tokens: 0,
            settlement_deadline_slot: 0,
            result_deadline_slot: 0,
            verification_deadline_slot: 0,
            claim_deadline_slot: 0,
            winner_reward_claimed: 0,
            verifier_reward_claimed_bitmap: 0,
            quorum_verifier_bitmap: 0,
            verifier_page_count: 0,
            _reserved1: [0; 4],
            verifier_reward_remaining: [0; VERIFIERS_PER_AUCTION],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(into = "u64", try_from = "u64")
)]
#[repr(transparent)]
pub struct BundleEscrowV2Status(u64);

#[allow(non_upper_case_globals)]
impl BundleEscrowV2Status {
    pub const Open: Self = Self(0);
    pub const Awarded: Self = Self(1);
    pub const ResultPosted: Self = Self(2);
    pub const FinalizedVerified: Self = Self(3);
    pub const FinalizedRejected: Self = Self(4);
    pub const Expired: Self = Self(5);

    pub const fn into_u64(self) -> u64 {
        self.0
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::FinalizedVerified | Self::FinalizedRejected | Self::Expired
        )
    }
}

impl Default for BundleEscrowV2Status {
    fn default() -> Self {
        Self::Open
    }
}

impl From<BundleEscrowV2Status> for u64 {
    fn from(value: BundleEscrowV2Status) -> Self {
        value.0
    }
}

impl TryFrom<u64> for BundleEscrowV2Status {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Awarded),
            2 => Ok(Self::ResultPosted),
            3 => Ok(Self::FinalizedVerified),
            4 => Ok(Self::FinalizedRejected),
            5 => Ok(Self::Expired),
            _ => Err(value),
        }
    }
}
