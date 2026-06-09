use crate::error::AuctionError;
use crate::{
    ConfigPolicyV2Flags, InstructionAccounts, Pubkey, RequestTierConfigV2,
    CONFIG_POLICY_V2_TIER_CONFIG_COUNT,
};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct InitConfigPolicyV2Accounts<'a, T> {
    pub authority: &'a T,
    pub config_policy: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for InitConfigPolicyV2Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [authority, config_policy, system_program, ..] = accounts else {
            return Err(Self::Error::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            config_policy,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for InitConfigPolicyV2Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.authority)
            .chain(std::iter::once(self.config_policy))
            .chain(std::iter::once(self.system_program))
    }
}

#[derive(Pod, Clone, Copy, Zeroable, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InitConfigPolicyV2Args {
    pub config_policy_lamports: u64,
    pub initial_admin_authority: Pubkey,
    pub service_authority: Pubkey,
    pub policy_flags: ConfigPolicyV2Flags,
    pub minimum_bundle_auction_pairs: u64,
    pub max_auction_credits_per_update: u64,
    pub v2_verifiers_per_auction: u8,
    pub v2_verifier_quorum: u8,
    pub _reserved0: [u8; 6],
    pub tier_configs: [RequestTierConfigV2; CONFIG_POLICY_V2_TIER_CONFIG_COUNT],
}
