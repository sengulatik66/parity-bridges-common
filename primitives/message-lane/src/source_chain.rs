// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Primitives of message lane module, that are used on the source chain.

use crate::{InboundLaneData, LaneId};

use frame_support::Parameter;
use sp_std::fmt::Debug;

/// The sender of the message on the source chain.
pub type Sender<AccountId> = frame_system::RawOrigin<AccountId>;

/// Target chain API. Used by source chain to verify target chain proofs.
///
/// All implementations of this trait should only work with finalized data that
/// can't change. Wrong implementation may lead to invalid lane states (i.e. lane
/// that's stuck) and/or processing messages without paying fees.
pub trait TargetHeaderChain<Payload, AccountId> {
	/// Error type.
	type Error: Debug + Into<&'static str>;

	/// Proof that messages have been received by target chain.
	type MessagesDeliveryProof: Parameter;

	/// Verify message payload before we accept it.
	///
	/// **CAUTION**: this is very important function. Incorrect implementation may lead
	/// to stuck lanes and/or relayers loses.
	///
	/// The proper implementation must ensure that the delivery-transaction with this
	/// payload would (at least) be accepted into target chain transaction pool AND
	/// eventually will be successfully 'mined'. The most obvious incorrect implementation
	/// example would be implementation for BTC chain that accepts payloads larger than
	/// 1MB. BTC nodes aren't accepting transactions that are larger than 1MB, so relayer
	/// will be unable to craft valid transaction => this (and all subsequent) messages will
	/// never be delivered.
	fn verify_message(payload: &Payload) -> Result<(), Self::Error>;

	/// Verify messages delivery proof and return lane && nonce of the latest recevied message.
	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<AccountId>), Self::Error>;
}

/// Lane message verifier.
///
/// Runtime developer may implement any additional validation logic over message-lane mechanism.
/// E.g. if lanes should have some security (e.g. you can only accept Lane1 messages from
/// Submitter1, Lane2 messages for those who has submitted first message to this lane, disable
/// Lane3 until some block, ...), then it may be built using this verifier.
///
/// Any fee requirements should also be enforced here.
pub trait LaneMessageVerifier<Submitter, Payload, Fee> {
	/// Error type.
	type Error: Debug + Into<&'static str>;

	/// Verify message payload and return Ok(()) if message is valid and allowed to be sent over the lane.
	fn verify_message(
		submitter: &Sender<Submitter>,
		delivery_and_dispatch_fee: &Fee,
		lane: &LaneId,
		payload: &Payload,
	) -> Result<(), Self::Error>;
}

/// Message delivery payment. It is called as a part of submit-message transaction. Transaction
/// submitter is paying (in source chain tokens/assets) for:
///
/// 1) submit-message-transaction-fee itself. This fee is not included in the
/// `delivery_and_dispatch_fee` and is witheld by the regular transaction payment mechanism;
/// 2) message-delivery-transaction-fee. It is submitted to the target node by relayer;
/// 3) message-dispatch fee. It is paid by relayer for processing message by target chain;
/// 4) message-receiving-delivery-transaction-fee. It is submitted to the source node
/// by relayer.
///
/// So to be sure that any non-altruist relayer would agree to deliver message, submitter
/// should set `delivery_and_dispatch_fee` to at least (equialent of): sum of fees from (2)
/// to (4) above, plus some interest for the relayer.
pub trait MessageDeliveryAndDispatchPayment<AccountId, Balance> {
	/// Error type.
	type Error: Debug + Into<&'static str>;

	/// Withhold/write-off delivery_and_dispatch_fee from submitter account to
	/// some relayers-fund account.
	fn pay_delivery_and_dispatch_fee(
		submitter: &Sender<AccountId>,
		fee: &Balance,
		relayer_fund_account: &AccountId,
	) -> Result<(), Self::Error>;

	/// Pay reward for delivering message to the given relayer account.
	fn pay_relayer_reward(
		confirmation_relayer: &AccountId,
		relayer: &AccountId,
		reward: &Balance,
		relayer_fund_account: &AccountId,
	);

	/// Perform some initialization in externalities-provided environment.
	///
	/// For instance you may ensure that particular required accounts or storage items are present.
	/// Returns the number of storage reads performed.
	fn initialize(_relayer_fund_account: &AccountId) -> usize {
		0
	}
}
