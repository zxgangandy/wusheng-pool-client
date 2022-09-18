
use snarkvm::{console::types::Field, prelude::*};

use ::rand::thread_rng;

/// A helper struct for an Aleo account.
#[derive(Debug)]
pub struct Account<N: Network> {
    /// The account private key.
    pub private_key: PrivateKey<N>,
    /// The account view key.
    pub view_key: ViewKey<N>,
    /// The account address.
    pub address: Address<N>,
}

impl<N: Network> FromStr for Account<N> {
    type Err = anyhow::Error;

    /// Initializes a new account from a private key string.
    fn from_str(private_key: &str) -> Result<Self, Self::Err> {
        Self::new_from(FromStr::from_str(private_key)?)
    }
}

impl<N: Network> Account<N> {
    /// Initializes a new account from private key.
    pub fn new_from(private_key: PrivateKey<N>) -> Result<Self> {
        Ok(Self {
            private_key,
            view_key: ViewKey::try_from(&private_key)?,
            address: Address::try_from(&private_key)?,
        })
    }

    /// Initializes a new account.
    pub fn new() -> Result<Self> {
        Self::new_from(PrivateKey::new(&mut thread_rng())?)
    }

    /// Signs a given message.
    pub fn sign(&self, message: &[Field<N>]) -> Result<Signature<N>> {
        Signature::sign(&self.private_key, message, &mut thread_rng())
    }

    /// Verifies a given message and signature.
    pub fn verify(&self, message: &[Field<N>], signature: &Signature<N>) -> bool {
        signature.verify(&self.address, message)
    }

    /// Returns the account private key.
    pub const fn private_key(&self) -> &PrivateKey<N> {
        &self.private_key
    }

    /// Returns the account view key.
    pub const fn view_key(&self) -> &ViewKey<N> {
        &self.view_key
    }

    /// Returns the account address.
    pub const fn address(&self) -> &Address<N> {
        &self.address
    }
}
