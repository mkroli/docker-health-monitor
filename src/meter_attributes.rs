/*
 * Copyright 2023 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use bollard::secret::ContainerSummary;
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelSet)]
pub struct ContainerSummaryLabels {
    pub id: Option<String>,
    pub image: Option<String>,
    pub name: Option<String>,
    pub health: Option<String>,
}

impl From<ContainerSummary> for ContainerSummaryLabels {
    fn from(c: ContainerSummary) -> Self {
        let name = c
            .names
            .iter()
            .flat_map(|names| names.first().cloned())
            .map(|name| name.strip_prefix('/').map_or(name.clone(), String::from))
            .next();
        ContainerSummaryLabels {
            id: c.id,
            image: c.image,
            name,
            health: None,
        }
    }
}
