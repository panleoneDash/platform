use crate::drive::Drive;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::op::LowLevelDriveOperation::CalculatedCostOperation;
use grovedb::{Element, PathQuery, TransactionArg};
use grovedb_costs::CostContext;
use grovedb_path::SubtreePath;

impl Drive {
    /// Gets the return value and the cost of a groveDB proved path query.
    /// Pushes the cost to `drive_operations` and returns the return value.
    /// Verbose should be generally set to false unless one needs to prove
    /// subsets of a proof.
    pub(super) fn grove_get_proved_path_query_with_conditional_v0<B: AsRef<[u8]>>(
        &self,
        root_path: SubtreePath<B>,
        key: &[u8],
        path_query_resolver: &impl Fn(Option<Element>) -> PathQuery,
        verbose: bool,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
    ) -> Result<Vec<u8>, Error> {
        let CostContext { value, cost } = self.grove.get_raw_optional(root_path, key, transaction);
        drive_operations.push(CalculatedCostOperation(cost));
        let maybe_element = value.map_err(Error::GroveDB)?;
        let path_query = path_query_resolver(maybe_element);

        let CostContext { value, cost } =
            self.grove
                .get_proved_path_query(&path_query, verbose, transaction);
        drive_operations.push(CalculatedCostOperation(cost));
        value.map_err(Error::GroveDB)
    }
}
