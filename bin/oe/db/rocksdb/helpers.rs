// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use super::kvdb_rocksdb::{CompactionProfile, DatabaseConfig};
use ethcore::client::{ClientConfig, DatabaseCompactionProfile};
use ethcore_db::NUM_COLUMNS;
use std::{collections::HashMap, path::Path};

pub fn compaction_profile(
    profile: &DatabaseCompactionProfile,
    db_path: &Path,
) -> CompactionProfile {
    match profile {
        &DatabaseCompactionProfile::Auto => CompactionProfile::auto(db_path),
        &DatabaseCompactionProfile::SSD => CompactionProfile::ssd(),
        &DatabaseCompactionProfile::HDD => CompactionProfile::hdd(),
    }
}

/// Spreads the `total` (in MiB) memory budget across the db columns.
/// If it's `None`, the default memory budget will be used for each column.
/// 90% of the memory budget is assigned to the first column, `col0`, which is where we store the
/// state.
pub fn memory_per_column(total: Option<usize>) -> HashMap<u32, usize> {
    let mut memory_per_column = HashMap::new();
    if let Some(budget) = total {
        // spend 90% of the memory budget on the state column, but at least 256 MiB
        memory_per_column.insert(ethcore_db::COL_STATE, std::cmp::max(budget * 9 / 10, 256));
        // spread the remaining 10% evenly across columns
        let rest_budget = budget / 10 / (ethcore_db::NUM_COLUMNS as usize - 1);

        for i in 1..ethcore_db::NUM_COLUMNS {
            // but at least 16 MiB for each column
            memory_per_column.insert(i, std::cmp::max(rest_budget, 16));
        }
    }
    memory_per_column
}

pub fn client_db_config(client_path: &Path, client_config: &ClientConfig) -> DatabaseConfig {
    let mut client_db_config = DatabaseConfig::with_columns(NUM_COLUMNS);

    client_db_config.memory_budget = memory_per_column(client_config.db_cache_size);
    client_db_config.compaction = compaction_profile(&client_config.db_compaction, &client_path);

    client_db_config
}
