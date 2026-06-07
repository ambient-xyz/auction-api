use crate::error::AuctionError;
use crate::{ConfigPolicyV2Flags, InstructionAccounts, Pubkey, RequestTierConfigV2};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SetConfigPolicyV2Accounts<'a, T> {
    pub authority: &'a T,
    pub config_policy: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for SetConfigPolicyV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [authority, config_policy, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            config_policy,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SetConfigPolicyV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.authority).chain(std::iter::once(self.config_policy))
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct ConfigPolicyV2PatchKind(pub u8);

impl ConfigPolicyV2PatchKind {
    pub const FLAGS: Self = Self(0);
    pub const AUTHORITY: Self = Self(1);
    pub const VERIFIER_SETTINGS: Self = Self(2);
    pub const TIER_CONFIG: Self = Self(3);
    pub const MAX_AUCTION_CREDITS_PER_UPDATE: Self = Self(4);
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct ConfigPolicyV2AuthorityKind(pub u8);

impl ConfigPolicyV2AuthorityKind {
    pub const ADMIN: Self = Self(0);
    pub const SERVICE: Self = Self(1);
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SetConfigPolicyV2Args {
    pub patch_kind: ConfigPolicyV2PatchKind,
    pub authority_kind: ConfigPolicyV2AuthorityKind,
    pub authority_index: u8,
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    pub _reserved0: [u8; 3],
    pub tier: u64,
    pub policy_flags: ConfigPolicyV2Flags,
    pub max_auction_credits_per_update: u64,
    pub authority: Pubkey,
    pub tier_config: RequestTierConfigV2,
}
