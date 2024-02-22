//! Broadcast trait representing the action of broadcasting a new identity state transition to the platform.
//!
//! The [BroadcastRequestForNewIdentity] trait is designed for the creation and broadcasting of new identity state transitions.
//! This involves the generation of a state transition object, signing it, and then broadcasting it to the platform.
//!
//! This trait is expected to be implemented by objects that encapsulate the necessary data and logic to perform
//! these operations, including the handling of asset lock proof and signing operations.
use std::fmt::Debug;

use dapi_grpc::platform::v0::wait_for_state_transition_result_request::{
    Version, WaitForStateTransitionResultRequestV0,
};
use dapi_grpc::platform::v0::{
    BroadcastStateTransitionRequest, WaitForStateTransitionResultRequest,
};

use dpp::serialization::PlatformSerializable;

use dpp::state_transition::StateTransition;

use crate::error::Error;

/// Trait implemented by objects that can be used to broadcast new identity state transitions.
///
/// [BroadcastRequestForNewIdentity] trait is used when a new identity needs to be created and broadcasted on the platform.
/// It encapsulates the data, the signing process, and the logic required to perform the broadcast operation.
///
/// Implementors of this trait will typically be responsible for creating an identity state transition,
/// signing it with the provided private key and signer, and preparing it for transport to the platform.
///
/// ## Example
///
/// To broadcast a new [Identity](dpp::prelude::Identity) state transition, you would typically
/// create an [IdentityCreateTransition](dpp::state_transition::identity_create_transition::IdentityCreateTransition),
/// sign it, and use the `broadcast_new_identity` method provided by this trait:
///
/// ```rust, ignore
///
/// use rs_sdk::{Sdk, platform::{BroadcastNewIdentity, IdentityCreateTransition}};
/// use dpp::identity::signer::Signer;
/// use dpp::prelude::{AssetLockProof, PrivateKey};
/// use dpp::version::PlatformVersion;
///
/// let mut sdk = Sdk::new_mock();
/// let asset_lock_proof = AssetLockProof::new(/* parameters for the asset lock proof */);
/// let private_key = PrivateKey::from(/* private key data */);
/// let signer = /* implementation of Signer trait */;
/// let platform_version = PlatformVersion::latest();
///
/// let identity_transition = IdentityCreateTransition::new(/* parameters for the transition */);
/// let result = identity_transition.broadcast_new_identity(asset_lock_proof, private_key, &signer, &platform_version);
///
/// match result {
///     Ok(transport_request) => {
///         // The transport_request can now be sent to the platform to broadcast the new identity.
///     }
///     Err(e) => {
///         // Handle the error
///     }
/// }
/// ```
///
/// As [BroadcastRequestForStateTransition] is a trait, it can be implemented for any type that represents
/// a new identity creation operation, allowing for flexibility in how new identities are broadcasted.
pub trait BroadcastRequestForStateTransition: Send + Debug + Clone {
    /// Converts the current instance into an instance of the `TransportRequest` type, ready for broadcasting.
    ///
    /// This method takes ownership of the instance upon which it's called (hence `self`), and attempts to perform the conversion,
    /// including signing the state transition with the provided private key and signer.
    ///
    /// # Arguments
    ///
    /// * `asset_lock_proof` - The proof that locks the asset which is used to create the identity.
    /// * `asset_lock_proof_private_key` - The private key associated with the asset lock proof.
    /// * `signer` - The signer to be used for signing the state transition.
    /// * `platform_version` - The version of the platform for which the state transition is intended.
    ///
    /// # Returns
    /// On success, this method yields an instance of the `TransportRequest` type (`T`), which can be used to broadcast the new identity state transition to the platform.
    /// On failure, it yields an [`Error`].
    ///
    /// # Error Handling
    /// This method propagates any errors encountered during the signing or conversion process.
    /// These are returned as [`Error`] instances.
    fn broadcast_request_for_state_transition(
        &self,
    ) -> Result<BroadcastStateTransitionRequest, Error>;

    fn wait_for_state_transition_result_request(
        &self,
    ) -> Result<WaitForStateTransitionResultRequest, Error>;
}

impl BroadcastRequestForStateTransition for StateTransition {
    fn broadcast_request_for_state_transition(
        &self,
    ) -> Result<BroadcastStateTransitionRequest, Error> {
        Ok(BroadcastStateTransitionRequest {
            state_transition: self.serialize_to_bytes()?,
        })
    }

    fn wait_for_state_transition_result_request(
        &self,
    ) -> Result<WaitForStateTransitionResultRequest, Error> {
        Ok(WaitForStateTransitionResultRequest {
            version: Some(Version::V0(WaitForStateTransitionResultRequestV0 {
                state_transition_hash: self.transaction_id()?.to_vec(),
                prove: true,
            })),
        })
    }
}