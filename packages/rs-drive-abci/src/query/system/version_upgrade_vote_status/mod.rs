mod v0;

use crate::error::query::QueryError;
use crate::error::Error;
use crate::platform_types::platform::Platform;
use crate::platform_types::platform_state::PlatformState;
use crate::query::QueryValidationResult;
use dapi_grpc::platform::v0::get_protocol_version_upgrade_vote_status_request::Version;
use dapi_grpc::platform::v0::GetProtocolVersionUpgradeVoteStatusRequest;
use dapi_grpc::Message;
use dpp::check_validation_result_with_data;
use dpp::validation::ValidationResult;
use dpp::version::PlatformVersion;

impl<C> Platform<C> {
    /// Querying of version upgrade vote status
    pub(in crate::query) fn query_version_upgrade_vote_status(
        &self,
        state: &PlatformState,
        query_data: &[u8],
        platform_version: &PlatformVersion,
    ) -> Result<QueryValidationResult<Vec<u8>>, Error> {
        let GetProtocolVersionUpgradeVoteStatusRequest { version } = check_validation_result_with_data!(
            GetProtocolVersionUpgradeVoteStatusRequest::decode(query_data).map_err(|e| {
                QueryError::InvalidArgument(format!("invalid query proto message: {}", e))
            })
        );

        let Some(version) = version else {
            return Ok(QueryValidationResult::new_with_error(
                QueryError::DecodingError("could not decode identity keys query".to_string()),
            ));
        };

        let feature_version_bounds = &platform_version
            .drive_abci
            .query
            .system
            .version_upgrade_vote_status;

        let feature_version = match &version {
            Version::V0(_) => 0,
        };
        if !feature_version_bounds.check_version(feature_version) {
            return Ok(QueryValidationResult::new_with_error(
                QueryError::UnsupportedQueryVersion(
                    "version_upgrade_vote_status".to_string(),
                    feature_version_bounds.min_version,
                    feature_version_bounds.max_version,
                    platform_version.protocol_version,
                    feature_version,
                ),
            ));
        }
        match version {
            Version::V0(get_protocol_version_upgrade_vote_status_request) => self
                .query_version_upgrade_vote_status_v0(
                    state,
                    get_protocol_version_upgrade_vote_status_request,
                    platform_version,
                ),
        }
    }
}
