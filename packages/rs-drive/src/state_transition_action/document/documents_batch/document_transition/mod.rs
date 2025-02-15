mod action_type;
/// document_base_transition_action
pub mod document_base_transition_action;
/// document_create_transition_action
pub mod document_create_transition_action;
/// document_delete_transition_action
pub mod document_delete_transition_action;
/// document_replace_transition_action
pub mod document_replace_transition_action;

pub use dpp::state_transition::documents_batch_transition::document_transition::action_type::DocumentTransitionActionType;

use derive_more::From;

use crate::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::DocumentBaseTransitionAction;
use crate::state_transition_action::document::documents_batch::document_transition::document_create_transition_action::{DocumentCreateTransitionAction, DocumentCreateTransitionActionAccessorsV0};
use crate::state_transition_action::document::documents_batch::document_transition::document_delete_transition_action::DocumentDeleteTransitionAction;
use crate::state_transition_action::document::documents_batch::document_transition::document_replace_transition_action::{DocumentReplaceTransitionAction, DocumentReplaceTransitionActionAccessorsV0};
use crate::state_transition_action::document::documents_batch::document_transition::document_delete_transition_action::v0::DocumentDeleteTransitionActionAccessorsV0;
use crate::state_transition_action::system::bump_identity_data_contract_nonce_action::BumpIdentityDataContractNonceAction;

/// version
pub const DOCUMENT_TRANSITION_ACTION_VERSION: u32 = 0;

/// action
#[derive(Debug, Clone, From)]
pub enum DocumentTransitionAction {
    /// create
    CreateAction(DocumentCreateTransitionAction),
    /// replace
    ReplaceAction(DocumentReplaceTransitionAction),
    /// delete
    DeleteAction(DocumentDeleteTransitionAction),
    /// bump identity data contract nonce
    BumpIdentityDataContractNonce(BumpIdentityDataContractNonceAction),
}

impl DocumentTransitionAction {
    /// base
    pub fn base(&self) -> Option<&DocumentBaseTransitionAction> {
        match self {
            DocumentTransitionAction::CreateAction(d) => Some(d.base()),
            DocumentTransitionAction::DeleteAction(d) => Some(d.base()),
            DocumentTransitionAction::ReplaceAction(d) => Some(d.base()),
            DocumentTransitionAction::BumpIdentityDataContractNonce(d) => None,
        }
    }

    /// base owned
    pub fn base_owned(self) -> Option<DocumentBaseTransitionAction> {
        match self {
            DocumentTransitionAction::CreateAction(d) => Some(d.base_owned()),
            DocumentTransitionAction::DeleteAction(d) => Some(d.base_owned()),
            DocumentTransitionAction::ReplaceAction(d) => Some(d.base_owned()),
            DocumentTransitionAction::BumpIdentityDataContractNonce(d) => None,
        }
    }
}
