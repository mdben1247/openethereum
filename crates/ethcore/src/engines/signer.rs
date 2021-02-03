// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! A signer used by Engines which need to sign messages.

use ethereum_types::{Address, H256};
use crypto::publickey::{self, Signature};

/// Everything that an Engine needs to sign messages.
pub trait EngineSigner: Send + Sync {
    /// Sign a consensus message hash.
    fn sign(&self, hash: H256) -> Result<Signature, publickey::Error>;

    /// Signing address
    fn address(&self) -> Address;
}

/// Creates a new `EngineSigner` from given key pair.
pub fn from_keypair(keypair: publickey::KeyPair) -> Box<dyn EngineSigner> {
    Box::new(Signer(keypair))
}

struct Signer(publickey::KeyPair);

impl EngineSigner for Signer {
    fn sign(&self, hash: H256) -> Result<Signature, publickey::Error> {
        publickey::sign(self.0.secret(), &hash)
    }

    fn address(&self) -> Address {
        self.0.address()
    }
}

#[cfg(test)]
mod test_signer {

    extern crate ethkey;

    use std::sync::Arc;

    use accounts::{self, AccountProvider, SignError};
    use self::ethkey::Password;

    use super::*;

    impl EngineSigner for (Arc<AccountProvider>, Address, Password) {
        fn sign(&self, hash: H256) -> Result<Signature, crypto::publickey::Error> {
            match self.0.sign(self.1, Some(self.2.clone()), hash) {
                Err(SignError::NotUnlocked) => unreachable!(),
                Err(SignError::NotFound) => Err(crypto::publickey::Error::InvalidAddress),
                Err(SignError::SStore(accounts::Error::EthCryptoPublicKey(err))) => Err(err),
                Err(SignError::SStore(accounts::Error::EthCrypto(err))) => {
                    warn!("Low level crypto error: {:?}", err);
                    Err(crypto::publickey::Error::InvalidSecretKey)
                }
                Err(SignError::SStore(err)) => {
                    warn!("Error signing for engine: {:?}", err);
                    Err(crypto::publickey::Error::InvalidSignature)
                }
                Ok(ok) => Ok(ok),
            }
        }

        fn address(&self) -> Address {
            self.1
        }
    }
}
