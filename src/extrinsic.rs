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

//////////////////////
// inspired by unit tests in substrate/node/executor/lib.rs


use runtime_io;
use keyring::{AuthorityKeyring, AccountKeyring};
use runtime_primitives::{generic, generic::Era, ApplyOutcome, ApplyError, ApplyResult, Perbill};
use node_runtime::{Header, Block, UncheckedExtrinsic, CheckedExtrinsic, Call, Runtime, Balances,
		BuildStorage, GenesisConfig, BalancesConfig, SessionConfig, StakingConfig, System,
		SystemConfig, GrandpaConfig, IndicesConfig, Event, Log};
use {balances, indices, session, system, staking, consensus, timestamp, treasury, contract};
		
const GENESIS_HASH: [u8; 32] = [69u8; 32];

fn alice() -> AccountId {
	AccountKeyring::Alice.into()
}

fn bob() -> AccountId {
	AccountKeyring::Bob.into()
}

fn charlie() -> AccountId {
	AccountKeyring::Charlie.into()
}

fn dave() -> AccountId {
	AccountKeyring::Dave.into()
}

fn eve() -> AccountId {
	AccountKeyring::Eve.into()
}

fn ferdie() -> AccountId {
	AccountKeyring::Ferdie.into()
}

fn sign(xt: CheckedExtrinsic) -> UncheckedExtrinsic {
	match xt.signed {
		Some((signed, index)) => {
			let era = Era::mortal(256, 0);
			let payload = (index.into(), xt.function, era, GENESIS_HASH);
			let key = AccountKeyring::from_public(&signed).unwrap();
			let signature = payload.using_encoded(|b| {
				if b.len() > 256 {
					key.sign(&runtime_io::blake2_256(b))
				} else {
					key.sign(b)
				}
			}).into();
			UncheckedExtrinsic {
				signature: Some((indices::address::Address::Id(signed), signature, payload.0, era)),
				function: payload.1,
			}
		}
		None => UncheckedExtrinsic {
			signature: None,
			function: xt.function,
		},
	}
}


fn xt() -> UncheckedExtrinsic {
		sign(CheckedExtrinsic {
			signed: Some((alice(), 0)),
			function: Call::Balances(balances::Call::transfer::<Runtime>(bob().into(), 69)),
		})
	}

