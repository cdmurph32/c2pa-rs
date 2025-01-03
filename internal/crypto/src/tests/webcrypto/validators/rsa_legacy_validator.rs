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

use wasm_bindgen_test::wasm_bindgen_test;

use crate::{
    raw_signature::{RawSignatureValidationError, RawSignatureValidator},
    webcrypto::validators::RsaLegacyValidator,
};

const SAMPLE_DATA: &[u8] = b"some sample content to sign";

#[wasm_bindgen_test]
fn rs256() {
    let signature = include_bytes!("../../fixtures/raw_signature/legacy/rs256.raw_sig");
    let pub_key = include_bytes!("../../fixtures/raw_signature/legacy/rs256.pub_key");

    RsaLegacyValidator::Rsa256
        .validate(signature, SAMPLE_DATA, pub_key)
        .unwrap();
}

#[wasm_bindgen_test]
fn rs256_bad_signature() {
    let mut signature =
        include_bytes!("../../fixtures/raw_signature/legacy/rs256.raw_sig").to_vec();
    assert_ne!(signature[10], 10);
    signature[10] = 10;

    let pub_key = include_bytes!("../../fixtures/raw_signature/ps256.pub_key");

    assert_eq!(
        RsaLegacyValidator::Rsa256
            .validate(&signature, SAMPLE_DATA, pub_key)
            .unwrap_err(),
        RawSignatureValidationError::SignatureMismatch
    );
}

#[wasm_bindgen_test]
fn rs256_bad_data() {
    let signature = include_bytes!("../../fixtures/raw_signature/legacy/rs256.raw_sig");
    let pub_key = include_bytes!("../../fixtures/raw_signature/ps256.pub_key");

    let mut data = SAMPLE_DATA.to_vec();
    data[10] = 0;

    assert_eq!(
        RsaLegacyValidator::Rsa256
            .validate(signature, &data, pub_key)
            .unwrap_err(),
        RawSignatureValidationError::SignatureMismatch
    );
}

#[wasm_bindgen_test]
fn rs384() {
    let signature = include_bytes!("../../fixtures/raw_signature/legacy/rs384.raw_sig");
    let pub_key = include_bytes!("../../fixtures/raw_signature/legacy/rs384.pub_key");

    RsaLegacyValidator::Rsa384
        .validate(signature, SAMPLE_DATA, pub_key)
        .unwrap();
}

#[wasm_bindgen_test]
fn rs512() {
    let signature = include_bytes!("../../fixtures/raw_signature/legacy/rs512.raw_sig");
    let pub_key = include_bytes!("../../fixtures/raw_signature/legacy/rs512.pub_key");

    RsaLegacyValidator::Rsa512
        .validate(signature, SAMPLE_DATA, pub_key)
        .unwrap();
}
