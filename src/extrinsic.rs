// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! an extract of substrate/core/test-runtime/src/lib.rs

#![cfg_attr(not(feature = "std"), no_std)]

/*
#[cfg(feature = "std")]
pub mod genesismap;
pub mod system;

use rstd::{prelude::*, marker::PhantomData};
*/
use parity_codec::{Encode, Decode, Input};

use primitives::Blake2Hasher;
//use trie_db::{TrieMut, Trie};
//use substrate_trie::{TrieDB, TrieDBMut, PrefixedMemoryDB};

/*
use substrate_client::{
	runtime_api as client_api, block_builder::api as block_builder_api, decl_runtime_apis,
	impl_runtime_apis,
};
*/
use runtime_primitives::{
	ApplyResult, transaction_validity::TransactionValidity,
	create_runtime_str,
	traits::{
		BlindCheckable, BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT,
		GetNodeBlockType, GetRuntimeBlockType, AuthorityIdFor,
	},
};
//use runtime_version::RuntimeVersion;
pub use primitives::hash::H256;
use primitives::{ed25519, sr25519, OpaqueMetadata};
//#[cfg(any(feature = "std", test))]
//use runtime_version::NativeVersion;
use inherents::{CheckInherentsResult, InherentData};
//use cfg_if::cfg_if;

/*
/// Test runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("test"),
	impl_name: create_runtime_str!("parity-test"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
};

fn version() -> RuntimeVersion {
	VERSION
}

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}
*/


/// Calls in transactions.
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Transfer {
	pub from: AccountId,
	pub to: AccountId,
	pub amount: u64,
	pub nonce: u64,
}

impl Transfer {
	/// Convert into a signed extrinsic.
	#[cfg(feature = "std")]
	pub fn into_signed_tx(self) -> Extrinsic {
		let signature = keyring::AccountKeyring::from_public(&self.from)
			.expect("Creates keyring from public key.").sign(&self.encode()).into();
		Extrinsic::Transfer(self, signature)
	}
}

/// Extrinsic for test-runtime.
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Extrinsic {
	AuthoritiesChange(Vec<AuthorityId>),
	Transfer(Transfer, AccountSignature),
	IncludeData(Vec<u8>),
}

#[cfg(feature = "std")]
impl serde::Serialize for Extrinsic
{
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
		self.using_encoded(|bytes| seq.serialize_bytes(bytes))
	}
}

impl BlindCheckable for Extrinsic {
	type Checked = Self;

	fn check(self) -> Result<Self, &'static str> {
		match self {
			Extrinsic::AuthoritiesChange(new_auth) => Ok(Extrinsic::AuthoritiesChange(new_auth)),
			Extrinsic::Transfer(transfer, signature) => {
				if runtime_primitives::verify_encoded_lazy(&signature, &transfer, &transfer.from) {
					Ok(Extrinsic::Transfer(transfer, signature))
				} else {
					Err(runtime_primitives::BAD_SIGNATURE)
				}
			},
			Extrinsic::IncludeData(data) => Ok(Extrinsic::IncludeData(data)),
		}
	}
}

impl ExtrinsicT for Extrinsic {
	fn is_signed(&self) -> Option<bool> {
		Some(true)
	}
}

impl Extrinsic {
	pub fn transfer(&self) -> &Transfer {
		match self {
			Extrinsic::Transfer(ref transfer, _) => transfer,
			_ => panic!("cannot convert to transfer ref"),
		}
	}
}


// The identity type used by authorities.
pub type AuthorityId = ed25519::Public;
// The signature type used by authorities.
pub type AuthoritySignature = ed25519::Signature;
/// An identifier for an account on this system.
pub type AccountId = sr25519::Public;
// The signature type used by accounts/transactions.
pub type AccountSignature = sr25519::Signature;
/// A simple hash type for all our hashing.
pub type Hash = H256;
/// The block number type used in this runtime.
pub type BlockNumber = u64;
/// Index of a transaction.
pub type Index = u64;
/// The item of a block digest.
pub type DigestItem = runtime_primitives::generic::DigestItem<H256, AuthorityId, AuthoritySignature>;
/// The digest of a block.
pub type Digest = runtime_primitives::generic::Digest<DigestItem>;
/// A test block.
pub type Block = runtime_primitives::generic::Block<Header, Extrinsic>;
/// A test block's header.
pub type Header = runtime_primitives::generic::Header<BlockNumber, BlakeTwo256, DigestItem>;

/*
/// Run whatever tests we have.
pub fn run_tests(mut input: &[u8]) -> Vec<u8> {
	use runtime_io::print;

	print("run_tests...");
	let block = Block::decode(&mut input).unwrap();
	print("deserialized block.");
	let stxs = block.extrinsics.iter().map(Encode::encode).collect::<Vec<_>>();
	print("reserialized transactions.");
	[stxs.len() as u8].encode()
}

/// Changes trie configuration (optionally) used in tests.
pub fn changes_trie_config() -> primitives::ChangesTrieConfiguration {
	primitives::ChangesTrieConfiguration {
		digest_interval: 4,
		digest_levels: 2,
	}
}

/// A type that can not be decoded.
#[derive(PartialEq)]
pub struct DecodeFails<B: BlockT> {
	_phantom: PhantomData<B>,
}

impl<B: BlockT> Encode for DecodeFails<B> {
	fn encode(&self) -> Vec<u8> {
		Vec::new()
	}
}

impl<B: BlockT> DecodeFails<B> {
	/// Create a new instance.
	pub fn new() -> DecodeFails<B> {
		DecodeFails {
			_phantom: Default::default(),
		}
	}
}

impl<B: BlockT> Decode for DecodeFails<B> {
	fn decode<I: Input>(_: &mut I) -> Option<Self> {
		// decoding always fails
		None
	}
}

*/
