use crypto::sha2::Sha256;
use secp256k1::Message;
use secp256k1::Secp256k1;

use crypto::digest::Digest;

use super::error::TerraRustScriptError;

pub struct Signature {}

impl Signature {
    pub fn verify<C: secp256k1::Verification + secp256k1::Context>(
        secp: &Secp256k1<C>,
        pub_key: &str,
        signature: &str,
        blob: &str,
    ) -> Result<(), TerraRustScriptError> {
        let public = base64::decode(pub_key)?;
        let sig = base64::decode(signature)?;
        let pk = secp256k1::PublicKey::from_slice(public.as_slice())?;
        let mut sha = Sha256::new();
        let mut sha_result: [u8; 32] = [0; 32];
        sha.input_str(blob);
        sha.result(&mut sha_result);

        let message: Message = Message::from_slice(&sha_result)?;
        let secp_sig = secp256k1::ecdsa::Signature::from_compact(sig.as_slice())?;
        secp.verify_ecdsa(&message, &secp_sig, &pk)?;
        Ok(())
    }
}
