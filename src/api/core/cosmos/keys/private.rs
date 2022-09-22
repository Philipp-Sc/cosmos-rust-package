use bitcoin::util::bip32::{ExtendedPrivKey, IntoDerivationPath};
use bitcoin::Network;
//use crypto::sha2::Sha256;
use secp256k1::Secp256k1;

//use crypto::digest::Digest;
use hkd32::mnemonic::{Phrase, Seed};

use rand_core::OsRng;

use super::error::TerraRustScriptError;
use super::public::PublicKey;

/// The Private key structure that is used to generate signatures and public keys
/// WARNING: No Security Audit has been performed
#[derive(Clone)]
pub struct PrivateKey {
    #[allow(missing_docs)]
    pub account: u32,
    #[allow(missing_docs)]
    pub index: u32,
    #[allow(missing_docs)]
    pub coin_type: u32,
    /// The 24 words used to generate this private key
    mnemonic: Option<Phrase>,
    #[allow(dead_code)]
    /// This is used for testing
    root_private_key: ExtendedPrivKey,
    /// The private key
    private_key: ExtendedPrivKey,
}

impl PrivateKey {
    /// Generate a new private key
    pub fn new<C: secp256k1::Signing + secp256k1::Context>(
        secp: &Secp256k1<C>,
        coin_type: u32,
    ) -> Result<PrivateKey, TerraRustScriptError> {
        let phrase =
            hkd32::mnemonic::Phrase::random(&mut OsRng, hkd32::mnemonic::Language::English);

        PrivateKey::gen_private_key_phrase(secp, phrase, 0, 0, coin_type, "")
    }
    /// generate a new private key with a seed phrase
    pub fn new_seed<C: secp256k1::Signing + secp256k1::Context>(
        secp: &Secp256k1<C>,
        seed_phrase: &str,
        coin_type: u32,
    ) -> Result<PrivateKey, TerraRustScriptError> {
        let phrase =
            hkd32::mnemonic::Phrase::random(&mut OsRng, hkd32::mnemonic::Language::English);

        PrivateKey::gen_private_key_phrase(secp, phrase, 0, 0, coin_type, seed_phrase)
    }
    /// for private key recovery. This is also used by wallet routines to re-hydrate the structure
    pub fn from_words<C: secp256k1::Signing + secp256k1::Context>(
        secp: &Secp256k1<C>,
        words: &str,
        account: u32,
        index: u32,
        coin_type: u32,
    ) -> Result<PrivateKey, TerraRustScriptError> {
        match hkd32::mnemonic::Phrase::new(words, hkd32::mnemonic::Language::English) {
            Ok(phrase) => {
                PrivateKey::gen_private_key_phrase(secp, phrase, account, index, coin_type, "")
            }
            Err(_) => Err(TerraRustScriptError::Phrasing),
        }
    }

    /// for private key recovery with seed phrase
    pub fn from_words_seed<C: secp256k1::Signing + secp256k1::Context>(
        secp: &Secp256k1<C>,
        words: &str,
        seed_pass: &str,
        coin_type: u32,
    ) -> Result<PrivateKey, TerraRustScriptError> {
        match hkd32::mnemonic::Phrase::new(words, hkd32::mnemonic::Language::English) {
            Ok(phrase) => {
                PrivateKey::gen_private_key_phrase(secp, phrase, 0, 0, coin_type, seed_pass)
            }
            Err(_) => Err(TerraRustScriptError::Phrasing),
        }
    }

    /// generate the public key for this private key
    pub fn public_key<C: secp256k1::Signing + secp256k1::Context>(
        &self,
        secp: &Secp256k1<C>,
    ) -> PublicKey {
        let x = &self.private_key.private_key.public_key(secp);
        PublicKey::from_bitcoin_public_key(x)
    }

    pub fn raw_key(&self) -> Vec<u8> {
        self.private_key.private_key.to_bytes()
    }

    fn gen_private_key_phrase<C: secp256k1::Signing + secp256k1::Context>(
        secp: &Secp256k1<C>,
        phrase: Phrase,
        account: u32,
        index: u32,
        coin_type: u32,
        seed_phrase: &str,
    ) -> Result<PrivateKey, TerraRustScriptError> {
        let seed = phrase.to_seed(seed_phrase);
        let root_private_key =
            ExtendedPrivKey::new_master(Network::Bitcoin, seed.as_bytes()).unwrap();
        let path = format!("m/44'/{}'/{}'/0/{}", coin_type, account, index);
        let derivation_path = path.into_derivation_path()?;

        let private_key = root_private_key.derive_priv(secp, &derivation_path)?;
        Ok(PrivateKey {
            account,
            index,
            coin_type,
            mnemonic: Some(phrase),
            root_private_key,
            private_key,
        })
    }

    /// the words used to generate this private key
    pub fn words(&self) -> Option<&str> {
        self.mnemonic.as_ref().map(|phrase| phrase.phrase())
    }

    /// used for testing
    /// could potentially be used to recreate the private key instead of words
    #[allow(dead_code)]
    pub(crate) fn seed(&self, passwd: &str) -> Option<Seed> {
        self.mnemonic.as_ref().map(|phrase| phrase.to_seed(passwd))
    }
}
