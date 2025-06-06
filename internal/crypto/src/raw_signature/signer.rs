// Copyright 2022 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    raw_signature::SigningAlg,
    time_stamp::{AsyncTimeStampProvider, TimeStampError, TimeStampProvider},
};

/// Implementations of the `RawSigner` trait generate a cryptographic signature
/// over an arbitrary byte array.
///
/// If an implementation _can_ be asynchronous, that is preferred.
pub trait RawSigner: TimeStampProvider {
    /// Return a raw signature over the original byte slice.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, RawSignerError>;

    /// Return the algorithm implemented by this signer.
    fn alg(&self) -> SigningAlg;

    /// Return the signing certificate chain.
    ///
    /// Each certificate should be encoded in DER format and sequenced from
    /// end-entity certificate to the outermost certificate authority.
    fn cert_chain(&self) -> Result<Vec<Vec<u8>>, RawSignerError>;

    /// Return the size in bytes of the largest possible expected signature.
    /// Signing will fail if the result of the [`sign`] function is larger
    /// than this value.
    ///
    /// [`sign`]: Self::sign
    fn reserve_size(&self) -> usize;

    /// Return an OCSP response for the signing certificate if available.
    ///
    /// By pre-querying the value for the signing certificate, the value can be
    /// cached which will reduce load on the certificate authority, as
    /// recommended by the C2PA spec.
    fn ocsp_response(&self) -> Option<Vec<u8>> {
        None
    }
}

/// Implementations of the `AsyncRawSigner` trait generate a cryptographic
/// signature over an arbitrary byte array.
///
/// Use this trait only when the implementation must be asynchronous.
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait AsyncRawSigner: AsyncTimeStampProvider + Sync {
    /// Return a raw signature over the original byte slice.
    async fn sign(&self, data: Vec<u8>) -> Result<Vec<u8>, RawSignerError>;

    /// Return the algorithm implemented by this signer.
    fn alg(&self) -> SigningAlg;

    /// Return the signing certificate chain.
    ///
    /// Each certificate should be encoded in DER format and sequenced from
    /// end-entity certificate to the outermost certificate authority.
    fn cert_chain(&self) -> Result<Vec<Vec<u8>>, RawSignerError>;

    /// Return the size in bytes of the largest possible expected signature.
    /// Signing will fail if the result of the [`sign`] function is larger
    /// than this value.
    ///
    /// [`sign`]: Self::sign
    fn reserve_size(&self) -> usize;

    /// Return an OCSP response for the signing certificate if available.
    ///
    /// By pre-querying the value for the signing certificate, the value can be
    /// cached which will reduce load on the certificate authority, as
    /// recommended by the C2PA spec.
    async fn ocsp_response(&self) -> Option<Vec<u8>> {
        None
    }
}

/// Implementations of the `AsyncRawSigner` trait generate a cryptographic
/// signature over an arbitrary byte array.
///
/// Use this trait only when the implementation must be asynchronous.
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait AsyncRawSigner: AsyncTimeStampProvider {
    /// Return a raw signature over the original byte slice.
    async fn sign(&self, data: Vec<u8>) -> Result<Vec<u8>, RawSignerError>;

    /// Return the algorithm implemented by this signer.
    fn alg(&self) -> SigningAlg;

    /// Return the signing certificate chain.
    ///
    /// Each certificate should be encoded in DER format and sequenced from
    /// end-entity certificate to the outermost certificate authority.
    fn cert_chain(&self) -> Result<Vec<Vec<u8>>, RawSignerError>;

    /// Return the size in bytes of the largest possible expected signature.
    /// Signing will fail if the result of the [`sign`] function is larger
    /// than this value.
    ///
    /// [`sign`]: Self::sign
    fn reserve_size(&self) -> usize;

    /// Return an OCSP response for the signing certificate if available.
    ///
    /// By pre-querying the value for the signing certificate, the value can be
    /// cached which will reduce load on the certificate authority, as
    /// recommended by the C2PA spec.
    async fn ocsp_response(&self) -> Option<Vec<u8>> {
        None
    }
}

/// Describes errors that can be identified when generating a raw signature.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum RawSignerError {
    /// The signing credentials are invalid.
    #[error("invalid signing credentials ({0})")]
    InvalidSigningCredentials(String),

    /// An I/O error occurred. This typically happens when loading
    /// public/private key material from files.
    ///
    /// NOTE: We do not directly capture the I/O error itself because it
    /// lacks an `Eq` implementation. Instead we capture the error description.
    #[error("I/O error ({0})")]
    IoError(String),

    /// An error was reported by the underlying cryptography implementation.
    #[error("an error was reported by the cryptography library: {0}")]
    CryptoLibraryError(String),

    /// An unexpected internal error occured while requesting the time stamp
    /// response.
    #[error("internal error ({0})")]
    InternalError(String),
}

impl From<std::io::Error> for RawSignerError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<openssl::error::ErrorStack> for RawSignerError {
    fn from(err: openssl::error::ErrorStack) -> Self {
        Self::CryptoLibraryError(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<crate::raw_signature::openssl::OpenSslMutexUnavailable> for RawSignerError {
    fn from(err: crate::raw_signature::openssl::OpenSslMutexUnavailable) -> Self {
        Self::InternalError(err.to_string())
    }
}

/// Return a built-in [`RawSigner`] instance using the provided signing
/// certificate and private key.
///
/// Which signers are available may vary depending on the platform and which
/// crate features were enabled. If the desired signing algorithm is
/// unavailable, will respond with `Err(RawSignerError::InternalError)`.
///
/// May return an `Err` response if the certificate chain or private key are
/// invalid.
#[allow(unused)] // arguments may or may not be used depending on crate features
pub fn signer_from_cert_chain_and_private_key(
    cert_chain: &[u8],
    private_key: &[u8],
    alg: SigningAlg,
    time_stamp_service_url: Option<String>,
) -> Result<Box<dyn RawSigner + Send + Sync>, RawSignerError> {
    #[cfg(any(target_arch = "wasm32", feature = "rust_native_crypto"))]
    {
        match crate::raw_signature::rust_native::signers::signer_from_cert_chain_and_private_key(
            cert_chain,
            private_key,
            alg,
            time_stamp_service_url.clone(),
        ) {
            Ok(signer) => return Ok(signer),
            Err(RawSignerError::InternalError(_)) => (),
            Err(err) => return Err(err),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        return crate::raw_signature::openssl::signers::signer_from_cert_chain_and_private_key(
            cert_chain,
            private_key,
            alg,
            time_stamp_service_url,
        );
    }

    Err(RawSignerError::InternalError(format!(
        "unsupported algorithm: {alg}"
    )))
}

/// Return a built-in [`AsyncRawSigner`] instance using the provided signing
/// certificate and private key.
///
/// Which signers are available may vary depending on the platform and which
/// crate features were enabled. If the desired signing algorithm is
/// unavailable, will respond with `Err(RawSignerError::InternalError)`.
///
/// May return an `Err` response if the certificate chain or private key are
/// invalid.
pub fn async_signer_from_cert_chain_and_private_key(
    cert_chain: &[u8],
    private_key: &[u8],
    alg: SigningAlg,
    time_stamp_service_url: Option<String>,
) -> Result<Box<dyn AsyncRawSigner + Send + Sync>, RawSignerError> {
    let sync_signer = signer_from_cert_chain_and_private_key(
        cert_chain,
        private_key,
        alg,
        time_stamp_service_url,
    )?;

    Ok(Box::new(AsyncRawSignerWrapper(sync_signer)))
}

struct AsyncRawSignerWrapper(Box<dyn RawSigner + Send + Sync>);

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl AsyncRawSigner for AsyncRawSignerWrapper {
    async fn sign(&self, data: Vec<u8>) -> Result<Vec<u8>, RawSignerError> {
        self.0.sign(&data)
    }

    fn alg(&self) -> SigningAlg {
        self.0.alg()
    }

    fn cert_chain(&self) -> Result<Vec<Vec<u8>>, RawSignerError> {
        self.0.cert_chain()
    }

    fn reserve_size(&self) -> usize {
        self.0.reserve_size()
    }

    async fn ocsp_response(&self) -> Option<Vec<u8>> {
        self.0.ocsp_response()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl AsyncTimeStampProvider for AsyncRawSignerWrapper {
    fn time_stamp_service_url(&self) -> Option<String> {
        self.0.time_stamp_service_url()
    }

    fn time_stamp_request_headers(&self) -> Option<Vec<(String, String)>> {
        self.0.time_stamp_request_headers()
    }

    fn time_stamp_request_body(&self, message: &[u8]) -> Result<Vec<u8>, TimeStampError> {
        self.0.time_stamp_request_body(message)
    }

    async fn send_time_stamp_request(
        &self,
        message: &[u8],
    ) -> Option<Result<Vec<u8>, TimeStampError>> {
        self.0.send_time_stamp_request(message)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) fn test_signer(alg: SigningAlg) -> Box<dyn RawSigner> {
    let (cert_chain, private_key) = match alg {
        SigningAlg::Ed25519 => (
            include_bytes!("../tests/fixtures/raw_signature/ed25519.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/ed25519.priv").as_slice(),
        ),
        SigningAlg::Es256 => (
            include_bytes!("../tests/fixtures/raw_signature/es256.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/es256.priv").as_slice(),
        ),
        SigningAlg::Es384 => (
            include_bytes!("../tests/fixtures/raw_signature/es384.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/es384.priv").as_slice(),
        ),
        SigningAlg::Es512 => (
            include_bytes!("../tests/fixtures/raw_signature/es512.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/es512.priv").as_slice(),
        ),
        SigningAlg::Ps256 => (
            include_bytes!("../tests/fixtures/raw_signature/ps256.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/ps256.priv").as_slice(),
        ),
        SigningAlg::Ps384 => (
            include_bytes!("../tests/fixtures/raw_signature/ps384.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/ps384.priv").as_slice(),
        ),
        SigningAlg::Ps512 => (
            include_bytes!("../tests/fixtures/raw_signature/ps512.pub").as_slice(),
            include_bytes!("../tests/fixtures/raw_signature/ps512.priv").as_slice(),
        ),
    };

    signer_from_cert_chain_and_private_key(cert_chain, private_key, alg, None).unwrap()
}
