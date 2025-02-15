use crate::error::Error;
use crate::platform_types::platform::PlatformRef;
use crate::rpc::core::CoreRPCLike;
use dpp::balances::credits::CREDITS_PER_DUFF;
use dpp::consensus::signature::{BasicECDSAError, SignatureError};

use dpp::consensus::state::identity::invalid_asset_lock_proof_value::InvalidAssetLockProofValueError;
use dpp::consensus::state::identity::IdentityAlreadyExistsError;
use dpp::dashcore::signer;
use dpp::dashcore::signer::double_sha;
use dpp::identity::KeyType;

use dpp::identity::state_transition::AssetLockProved;
use dpp::prelude::ConsensusValidationResult;
use dpp::serialization::Signable;
use dpp::state_transition::identity_create_transition::accessors::IdentityCreateTransitionAccessorsV0;

use dpp::state_transition::identity_create_transition::IdentityCreateTransition;
use dpp::state_transition::{StateTransition, StateTransitionLike};

use dpp::version::PlatformVersion;
use drive::state_transition_action::identity::identity_create::IdentityCreateTransitionAction;
use drive::state_transition_action::StateTransitionAction;

use crate::error::execution::ExecutionError;
use crate::execution::types::execution_operation::signature_verification_operation::SignatureVerificationOperation;
use crate::execution::types::execution_operation::ValidationOperation;
use crate::execution::types::state_transition_execution_context::{
    StateTransitionExecutionContext, StateTransitionExecutionContextMethodsV0,
};
use crate::execution::validation::state_transition::common::asset_lock::proof::validate::AssetLockProofValidation;
use dpp::version::DefaultForPlatformVersion;
use drive::grovedb::TransactionArg;

use crate::execution::validation::state_transition::common::asset_lock::transaction::fetch_asset_lock_transaction_output_sync::fetch_asset_lock_transaction_output_sync;
use crate::execution::validation::state_transition::common::validate_unique_identity_public_key_hashes_in_state::validate_unique_identity_public_key_hashes_in_state;

pub(in crate::execution::validation::state_transition::state_transitions::identity_create) trait IdentityCreateStateTransitionStateValidationV0
{
    fn validate_state_v0<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        execution_context: &mut StateTransitionExecutionContext,
        tx: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error>;

    fn transform_into_action_v0<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        execution_context: &mut StateTransitionExecutionContext,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error>;
}

impl IdentityCreateStateTransitionStateValidationV0 for IdentityCreateTransition {
    fn validate_state_v0<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        execution_context: &mut StateTransitionExecutionContext,
        tx: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let drive = platform.drive;
        let mut state_transition_execution_context =
            StateTransitionExecutionContext::default_for_platform_version(platform_version)?;
        let mut validation_result = ConsensusValidationResult::<StateTransitionAction>::default();

        let identity_id = self.identity_id();
        let balance =
            drive.fetch_identity_balance(identity_id.to_buffer(), tx, platform_version)?;

        // Balance is here to check if the identity does already exist
        if balance.is_some() {
            return Ok(ConsensusValidationResult::new_with_error(
                IdentityAlreadyExistsError::new(identity_id.to_owned()).into(),
            ));
        }

        // Validate asset lock proof state
        validation_result.merge(self.asset_lock_proof().validate(
            platform,
            tx,
            platform_version,
        )?);

        if !validation_result.is_valid() {
            return Ok(validation_result);
        }

        // Now we should check the state of added keys to make sure there aren't any that already exist
        validation_result.add_errors(
            validate_unique_identity_public_key_hashes_in_state(
                self.public_keys(),
                drive,
                &mut state_transition_execution_context,
                tx,
                platform_version,
            )?
            .errors,
        );

        if !validation_result.is_valid() {
            return Ok(validation_result);
        }

        self.transform_into_action_v0(platform, execution_context, platform_version)
    }

    fn transform_into_action_v0<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        execution_context: &mut StateTransitionExecutionContext,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let mut validation_result = ConsensusValidationResult::<StateTransitionAction>::default();

        let tx_out_validation = fetch_asset_lock_transaction_output_sync(
            platform.core_rpc,
            self.asset_lock_proof(),
            platform_version,
        )?;

        if !tx_out_validation.is_valid() {
            return Ok(ConsensusValidationResult::new_with_errors(
                tx_out_validation.errors,
            ));
        }

        let tx_out = tx_out_validation.into_data()?;
        let min_value = IdentityCreateTransition::get_minimal_asset_lock_value(platform_version)?;
        if tx_out.value < min_value {
            return Ok(ConsensusValidationResult::new_with_error(
                InvalidAssetLockProofValueError::new(tx_out.value, min_value).into(),
            ));
        }

        // Verify one time signature

        let signable_bytes = StateTransition::IdentityCreate(self.clone()).signable_bytes()?;

        let public_key_hash = tx_out
            .script_pubkey
            .p2pkh_public_key_hash_bytes()
            .ok_or_else(|| {
                Error::Execution(ExecutionError::CorruptedCodeExecution(
                    "output must be a valid p2pkh already",
                ))
            })?;

        execution_context.add_operation(ValidationOperation::DoubleSha256);
        execution_context.add_operation(ValidationOperation::SignatureVerification(
            SignatureVerificationOperation::new(KeyType::ECDSA_HASH160),
        ));

        if let Err(e) = signer::verify_hash_signature(
            &double_sha(signable_bytes),
            self.signature().as_slice(),
            public_key_hash,
        ) {
            return Ok(ConsensusValidationResult::new_with_error(
                SignatureError::BasicECDSAError(BasicECDSAError::new(e.to_string())).into(),
            ));
        }

        match IdentityCreateTransitionAction::try_from_borrowed(
            self,
            tx_out.value * CREDITS_PER_DUFF,
        ) {
            Ok(action) => {
                validation_result.set_data(action.into());
            }
            Err(error) => {
                validation_result.add_error(error);
            }
        }

        Ok(validation_result)
    }
}
