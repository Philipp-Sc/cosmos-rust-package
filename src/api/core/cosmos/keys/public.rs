use bitcoin::bech32::{decode, encode, u5, FromBase32, ToBase32, Variant};
use crypto::digest::Digest;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;

pub use ed25519_dalek::PublicKey as Ed25519;
use serde::{Deserialize, Serialize};

use super::error::TerraRustScriptError;

static BECH32_PUBKEY_DATA_PREFIX_SECP256K1: [u8; 5] = [0xeb, 0x5a, 0xe9, 0x87, 0x21];
// "eb5ae98721";
static BECH32_PUBKEY_DATA_PREFIX_ED25519: [u8; 5] = [0x16, 0x24, 0xde, 0x64, 0x20]; // "eb5ae98721";

#[derive(Deserialize, Serialize, Debug, Clone)]
/// The public key we used to generate the cosmos/tendermind/terrad addresses
pub struct PublicKey {
    /// This is optional as we can generate non-pub keys without
    pub raw_pub_key: Option<Vec<u8>>,
    /// The raw bytes used to generate non-pub keys
    pub raw_address: Option<Vec<u8>>,
}
/*
upgrade eventually to support
Variant::Bech32M ?
 */
impl PublicKey {
    /// Generate a Cosmos/Tendermint/Terrad Public Key
    pub fn from_bitcoin_public_key(bpub: &bitcoin::key::PublicKey) -> PublicKey {
        let bpub_bytes = bpub.inner.serialize();
        //     eprintln!("B-PK-{}", hex::encode(bpub_bytes));
        let raw_pub_key = PublicKey::pubkey_from_public_key(&bpub_bytes);
        let raw_address = PublicKey::address_from_public_key(&bpub_bytes);

        PublicKey {
            raw_pub_key: Some(raw_pub_key),
            raw_address: Some(raw_address),
        }
    }
    /// Generate from secp256k1 Cosmos/Terrad Public Key
    pub fn from_public_key(bpub: &[u8]) -> PublicKey {
        let raw_pub_key = PublicKey::pubkey_from_public_key(bpub);
        let raw_address = PublicKey::address_from_public_key(bpub);

        PublicKey {
            raw_pub_key: Some(raw_pub_key),
            raw_address: Some(raw_address),
        }
    }
    /// Generate a Cosmos/Tendermint/Terrad Account
    pub fn from_account(
        acc_address: &str,
        prefix: &str,
    ) -> Result<PublicKey, TerraRustScriptError> {
        PublicKey::check_prefix_and_length(prefix, acc_address, 44).and_then(|vu5| {
            let vu8 = Vec::from_base32(vu5.as_slice()).map_err(|source| {
                TerraRustScriptError::Conversion {
                    key: acc_address.into(),
                    source,
                }
            })?;
            Ok(PublicKey {
                raw_pub_key: None,
                raw_address: Some(vu8),
            })
        })
    }
    /// build a public key from a tendermint public key
    pub fn from_tendermint_key(
        tendermint_public_key: &str,
    ) -> Result<PublicKey, TerraRustScriptError> {
        // Len 83 == PubKeySecp256k1 key with a prefix of 0xEB5AE987
        // Len 82 == PubKeyEd25519 key with a prefix of 0x1624DE64

        let len = tendermint_public_key.len();
        if len == 83 {
            PublicKey::check_prefix_and_length("terravalconspub", tendermint_public_key, len)
                .and_then(|vu5| {
                    let vu8 = Vec::from_base32(vu5.as_slice()).map_err(|source| {
                        TerraRustScriptError::Conversion {
                            key: tendermint_public_key.into(),
                            source,
                        }
                    })?;
                    log::debug!("{:#?}", hex::encode(&vu8));
                    if vu8.starts_with(&BECH32_PUBKEY_DATA_PREFIX_SECP256K1) {
                        let public_key = PublicKey::public_key_from_pubkey(&vu8)?;
                        let raw = PublicKey::address_from_public_key(&public_key);

                        Ok(PublicKey {
                            raw_pub_key: Some(vu8),
                            raw_address: Some(raw),
                        })
                    } else {
                        Err(TerraRustScriptError::ConversionSECP256k1)
                    }
                })
        } else if len == 82 {
            //  eprintln!("ED25519 keys are not currently supported");
            // todo!()

            PublicKey::check_prefix_and_length("terravalconspub", tendermint_public_key, len)
                .and_then(|vu5| {
                    let vu8 = Vec::from_base32(vu5.as_slice()).map_err(|source| {
                        TerraRustScriptError::Conversion {
                            key: tendermint_public_key.into(),
                            source,
                        }
                    })?;
                    //   log::debug!("{:#?}", hex::encode(&vu8));
                    log::info!("ED25519 public keys are not fully supported");
                    if vu8.starts_with(&BECH32_PUBKEY_DATA_PREFIX_ED25519) {
                        //   let public_key = PublicKey::pubkey_from_ed25519_public_key(&vu8);
                        let raw = PublicKey::address_from_public_ed25519_key(&vu8)?;
                        Ok(PublicKey {
                            raw_pub_key: Some(vu8),
                            raw_address: Some(raw),
                        })
                    } else {
                        //     eprintln!("{}", hex::encode(&vu8));
                        Err(TerraRustScriptError::ConversionED25519)
                    }
                })

            /* */
        } else {
            Err(TerraRustScriptError::ConversionLength(len))
        }
    }
    /// build a terravalcons address from a tendermint hex key
    /// the tendermint_hex_address should be a hex code of 40 length
    pub fn from_tendermint_address(
        tendermint_hex_address: &str,
    ) -> Result<PublicKey, TerraRustScriptError> {
        let len = tendermint_hex_address.len();
        if len == 40 {
            let raw = hex::decode(tendermint_hex_address)?;
            Ok(PublicKey {
                raw_pub_key: None,
                raw_address: Some(raw),
            })
        } else {
            Err(TerraRustScriptError::ConversionLengthED25519Hex(len))
        }
    }
    /// Generate a Operator address for this public key (used by the validator)
    pub fn from_operator_address(valoper_address: &str) -> Result<PublicKey, TerraRustScriptError> {
        PublicKey::check_prefix_and_length("terravaloper", valoper_address, 51).and_then(|vu5| {
            let vu8 = Vec::from_base32(vu5.as_slice()).map_err(|source| {
                TerraRustScriptError::Conversion {
                    key: valoper_address.into(),
                    source,
                }
            })?;
            Ok(PublicKey {
                raw_pub_key: None,
                raw_address: Some(vu8),
            })
        })
    }

    /// Generate Public key from raw address
    pub fn from_raw_address(raw_address: &str) -> Result<PublicKey, TerraRustScriptError> {
        let vec1 = hex::decode(raw_address)?;

        Ok(PublicKey {
            raw_pub_key: None,
            raw_address: Some(vec1),
        })
    }
    fn check_prefix_and_length(
        prefix: &str,
        data: &str,
        length: usize,
    ) -> Result<Vec<u5>, TerraRustScriptError> {
        let (hrp, decoded_str, _) =
            decode(data).map_err(|source| TerraRustScriptError::Conversion {
                key: data.into(),
                source,
            })?;
        if hrp == prefix && data.len() == length {
            Ok(decoded_str)
        } else {
            Err(TerraRustScriptError::Bech32DecodeExpanded(
                hrp,
                data.len(),
                prefix.into(),
                length,
            ))
        }
    }
    /**
     * Gets a bech32-words pubkey from a compressed bytes Secp256K1 public key.
     *
     * @param publicKey raw public key
     */
    pub fn pubkey_from_public_key(public_key: &[u8]) -> Vec<u8> {
        [
            BECH32_PUBKEY_DATA_PREFIX_SECP256K1.to_vec(),
            public_key.to_vec(),
        ]
        .concat()
    }
    /**
     * Gets a bech32-words pubkey from a compressed bytes Ed25519 public key.
     *
     * @param publicKey raw public key
     */
    pub fn pubkey_from_ed25519_public_key(public_key: &[u8]) -> Vec<u8> {
        [
            BECH32_PUBKEY_DATA_PREFIX_ED25519.to_vec(),
            public_key.to_vec(),
        ]
        .concat()
    }
    /// Translate from a BECH32 prefixed key to a standard public key
    pub fn public_key_from_pubkey(pub_key: &[u8]) -> Result<Vec<u8>, TerraRustScriptError> {
        if pub_key.starts_with(&BECH32_PUBKEY_DATA_PREFIX_SECP256K1) {
            let len = BECH32_PUBKEY_DATA_PREFIX_SECP256K1.len();
            let len2 = pub_key.len();
            Ok(Vec::from(&pub_key[len..len2]))
        } else if pub_key.starts_with(&BECH32_PUBKEY_DATA_PREFIX_ED25519) {
            let len = BECH32_PUBKEY_DATA_PREFIX_ED25519.len();
            let len2 = pub_key.len();
            let vec = &pub_key[len..len2];
            let ed25519_pubkey = ed25519_dalek::PublicKey::from_bytes(vec)?;
            Ok(ed25519_pubkey.to_bytes().to_vec())
        } else {
            log::info!("pub key does not start with BECH32 PREFIX");
            Err(TerraRustScriptError::Bech32DecodeErr)
        }
    }

    /**
    * Gets a raw address from a compressed bytes public key.
    *
    * @param publicKey raw public key

    */

    pub fn address_from_public_key(public_key: &[u8]) -> Vec<u8> {
        // Vec<bech32::u5> {

        let mut hasher = Ripemd160::new();
        let mut sha = Sha256::new();
        let mut sha_result: [u8; 32] = [0; 32];
        let mut ripe_result: [u8; 20] = [0; 20];
        sha.input(public_key);
        sha.result(&mut sha_result);
        hasher.input(&sha_result);
        hasher.result(&mut ripe_result);
        let address: Vec<u8> = ripe_result.to_vec();
        address
    }
    /**
    * Gets a raw address from a  ed25519 public key.
    *
    * @param publicKey raw public key

    */

    pub fn address_from_public_ed25519_key(
        public_key: &[u8],
    ) -> Result<Vec<u8>, TerraRustScriptError> {
        // Vec<bech32::u5> {

        if public_key.len() != (32 + 5/* the 5 is the BECH32 ED25519 prefix */) {
            Err(TerraRustScriptError::ConversionPrefixED25519(
                public_key.len(),
                hex::encode(public_key),
            ))
        } else {
            // eprintln!("a_pub_ed_key {}", hex::encode(public_key));
            log::debug!(
                "address_from_public_ed25519_key public key - {}",
                hex::encode(public_key)
            );
            //  let mut hasher = Ripemd160::new();
            let mut sha = Sha256::new();
            let mut sha_result: [u8; 32] = [0; 32];
            //  let mut ripe_result: [u8; 20] = [0; 20];
            // let v = &public_key[5..37];

            sha.input(&public_key[5..]);
            sha.result(&mut sha_result);
            //    hasher.input(public_key);
            //hasher.input(v);
            //    hasher.input(&sha_result);
            //   hasher.result(&mut ripe_result);

            let address: Vec<u8> = sha_result[0..20].to_vec();
            // let address: Vec<u8> = ripe_result.to_vec();
            //     eprintln!("address_from_public_ed_key {}", hex::encode(&address));
            log::debug!(
                "address_from_public_ed25519_key sha result - {}",
                hex::encode(&address)
            );
            Ok(address)
        }
    }
    /// The main account used in most things
    pub fn account(&self, prefix: &str) -> Result<String, TerraRustScriptError> {
        match &self.raw_address {
            Some(raw) => {
                let data = encode(prefix, raw.to_base32(), Variant::Bech32);
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => Err(TerraRustScriptError::Implementation),
        }
    }
    /// The operator address used for validators
    pub fn operator_address(&self, prefix: &str) -> Result<String, TerraRustScriptError> {
        match &self.raw_address {
            Some(raw) => {
                let data = encode(
                    &format!("{}{}", prefix, "valoper"),
                    raw.to_base32(),
                    Variant::Bech32,
                );
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => Err(TerraRustScriptError::Implementation),
        }
    }
    /// application public key - Application keys are associated with a public key terrapub- and an address terra-
    pub fn application_public_key(&self, prefix: &str) -> Result<String, TerraRustScriptError> {
        match &self.raw_pub_key {
            Some(raw) => {
                let data = encode(
                    &format!("{}{}", prefix, "pub"),
                    raw.to_base32(),
                    Variant::Bech32,
                );
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => {
                log::warn!("Missing Public Key. Can't continue");
                Err(TerraRustScriptError::Implementation)
            }
        }
    }
    /// The operator address used for validators public key.
    pub fn operator_address_public_key(
        &self,
        prefix: &str,
    ) -> Result<String, TerraRustScriptError> {
        match &self.raw_pub_key {
            Some(raw) => {
                let data = encode(
                    &format!("{}{}", prefix, "valoperpub"),
                    raw.to_base32(),
                    Variant::Bech32,
                );
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => Err(TerraRustScriptError::Implementation),
        }
    }
    /// This is a unique key used to sign block hashes. It is associated with a public key terravalconspub.
    pub fn tendermint(&self, prefix: &str) -> Result<String, TerraRustScriptError> {
        match &self.raw_address {
            Some(raw) => {
                let data = encode(
                    &format!("{}{}", prefix, "valcons"),
                    raw.to_base32(),
                    Variant::Bech32,
                );
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => Err(TerraRustScriptError::Implementation),
        }
    }
    /// This is a unique key used to sign block hashes. It is associated with a public key terravalconspub.
    pub fn tendermint_pubkey(&self, prefix: &str) -> Result<String, TerraRustScriptError> {
        match &self.raw_pub_key {
            Some(raw) => {
                // eprintln!("{} - tendermint_pubkey", hex::encode(raw));
                let b32 = raw.to_base32();
                let data = encode(&format!("{}{}", prefix, "valconspub"), b32, Variant::Bech32);
                match data {
                    Ok(acc) => Ok(acc),
                    Err(_) => Err(TerraRustScriptError::Bech32DecodeErr),
                }
            }
            None => Err(TerraRustScriptError::Implementation),
        }
    }
}
