//! Exact-replay class membership and class-filtered verifier selection (ENG-256).

use crate::error::AuctionError;
use crate::{InstructionAccounts, Pubkey};
use bytemuck::{Pod, Zeroable};

/// A node joins the policy's replay class. `payer` funds the class account on the first join.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct JoinReplayClassV1Accounts<'a, T> {
    pub node: &'a T,
    pub payer: &'a T,
    pub replay_class: &'a T,
    pub config_policy: &'a T,
    pub system_program: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for JoinReplayClassV1Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [node, payer, replay_class, config_policy, system_program, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self {
            node,
            payer,
            replay_class,
            config_policy,
            system_program,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for JoinReplayClassV1Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.node)
            .chain(std::iter::once(self.payer))
            .chain(std::iter::once(self.replay_class))
            .chain(std::iter::once(self.config_policy))
            .chain(std::iter::once(self.system_program))
    }
}

/// A member leaves its replay class.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct LeaveReplayClassV1Accounts<'a, T> {
    pub node: &'a T,
    pub replay_class: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for LeaveReplayClassV1Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [node, replay_class, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self { node, replay_class })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for LeaveReplayClassV1Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.node).chain(std::iter::once(self.replay_class))
    }
}

/// An admin removes a member (an on-chain suspension).
#[derive(Clone, Debug)]
#[repr(C)]
pub struct RemoveReplayClassMemberV1Accounts<'a, T> {
    pub admin: &'a T,
    pub replay_class: &'a T,
    pub config_policy: &'a T,
}

impl<'a, T> TryFrom<&'a [T]> for RemoveReplayClassMemberV1Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [admin, replay_class, config_policy, ..] = accounts else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        Ok(Self {
            admin,
            replay_class,
            config_policy,
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for RemoveReplayClassMemberV1Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.admin)
            .chain(std::iter::once(self.replay_class))
            .chain(std::iter::once(self.config_policy))
    }
}

/// Verifier selection for an exact-replay escrow: the draw of `SelectBundleVerifiersV2`, restricted to the members
/// of the policy's replay class. The dispute account is present for a replacement draw only.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct SelectReplayVerifiersV1Accounts<'a, T> {
    pub bundle_escrow: &'a T,
    pub auction_verifiers: &'a T,
    pub slot_hashes: &'a T,
    pub replay_class: &'a T,
    pub config_policy: &'a T,
    pub bundle_verification_dispute: Option<&'a T>,
}

impl<'a, T> TryFrom<&'a [T]> for SelectReplayVerifiersV1Accounts<'a, T> {
    type Error = AuctionError;

    fn try_from(accounts: &'a [T]) -> Result<Self, Self::Error> {
        let [bundle_escrow, auction_verifiers, slot_hashes, replay_class, config_policy, remaining_accounts @ ..] =
            accounts
        else {
            return Err(AuctionError::NotEnoughAccounts);
        };
        if remaining_accounts.len() > 1 {
            return Err(AuctionError::InvalidVerifierSelectionAccounts);
        }
        Ok(Self {
            bundle_escrow,
            auction_verifiers,
            slot_hashes,
            replay_class,
            config_policy,
            bundle_verification_dispute: remaining_accounts.first(),
        })
    }
}

impl<'a, T> InstructionAccounts<'a, T> for SelectReplayVerifiersV1Accounts<'a, T> {
    fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        std::iter::once(self.bundle_escrow)
            .chain(std::iter::once(self.auction_verifiers))
            .chain(std::iter::once(self.slot_hashes))
            .chain(std::iter::once(self.replay_class))
            .chain(std::iter::once(self.config_policy))
            .chain(self.bundle_verification_dispute)
    }
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct JoinReplayClassV1Args {
    /// Must be the policy's `replay.class_id`.
    pub class_id: [u8; 32],
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct LeaveReplayClassV1Args {
    pub class_id: [u8; 32],
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct RemoveReplayClassMemberV1Args {
    pub class_id: [u8; 32],
    pub member: Pubkey,
}

#[derive(Clone, Copy, Zeroable, PartialEq, Eq, Debug, Pod)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct SelectReplayVerifiersV1Args {}
