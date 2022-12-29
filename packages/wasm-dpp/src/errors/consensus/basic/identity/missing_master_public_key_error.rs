use dpp::consensus::basic::identity::MissingMasterPublicKeyError;
use dpp::consensus::ConsensusError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=MissingMasterPublicKeyError)]
pub struct MissingMasterPublicKeyErrorWasm {
    inner: MissingMasterPublicKeyError,
}

impl From<&MissingMasterPublicKeyError> for MissingMasterPublicKeyErrorWasm {
    fn from(e: &MissingMasterPublicKeyError) -> Self {
        Self { inner: e.clone() }
    }
}

#[wasm_bindgen(js_class=MissingMasterPublicKeyError)]
impl MissingMasterPublicKeyErrorWasm {
    #[wasm_bindgen(js_name=getCode)]
    pub fn get_code(&self) -> u32 {
        ConsensusError::from(self.inner.clone()).code()
    }
}