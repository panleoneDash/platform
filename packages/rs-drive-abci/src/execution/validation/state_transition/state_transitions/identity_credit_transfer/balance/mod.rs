use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::validation::state_transition::identity_credit_transfer::balance::v0::IdentityCreditTransferTransitionBalanceValidationV0;
use crate::execution::validation::state_transition::identity_credit_transfer::nonce::v0::IdentityCreditTransferTransitionIdentityNonceV0;
use crate::execution::validation::state_transition::processor::v0::{
    StateTransitionBalanceValidationV0, StateTransitionNonceValidationV0,
};
use crate::platform_types::platform::PlatformStateRef;
use dpp::block::block_info::BlockInfo;
use dpp::identity::PartialIdentity;
use dpp::state_transition::identity_credit_transfer_transition::IdentityCreditTransferTransition;
use dpp::validation::SimpleConsensusValidationResult;
use dpp::version::PlatformVersion;
use drive::grovedb::TransactionArg;

pub(crate) mod v0;
impl StateTransitionBalanceValidationV0 for IdentityCreditTransferTransition {
    fn validate_balance(
        &self,
        identity: Option<&mut PartialIdentity>,
        platform: &PlatformStateRef,
        block_info: &BlockInfo,
        tx: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .identity_credit_transfer_state_transition
            .balance
        {
            Some(0) => {
                self.validate_balance_v0(identity, platform, block_info, tx, platform_version)
            }
            Some(version) => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "identity credit transfer transition: validate_balance".to_string(),
                known_versions: vec![0],
                received: version,
            })),
            None => Err(Error::Execution(ExecutionError::VersionNotActive {
                method: "identity credit transfer transition: validate_balance".to_string(),
                known_versions: vec![0],
            })),
        }
    }
}
