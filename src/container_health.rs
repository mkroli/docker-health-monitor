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

use bollard::models::{Health, HealthStatusEnum};
use opentelemetry::Value;

#[derive(Clone, PartialEq)]
pub struct ContainerHealth {
    container_health_status: Option<HealthStatusEnum>,
}

impl ContainerHealth {
    pub fn values() -> Vec<ContainerHealth> {
        let mut values: Vec<ContainerHealth> = vec![
            HealthStatusEnum::EMPTY,
            HealthStatusEnum::NONE,
            HealthStatusEnum::STARTING,
            HealthStatusEnum::HEALTHY,
            HealthStatusEnum::UNHEALTHY,
        ]
        .into_iter()
        .map(|s| ContainerHealth {
            container_health_status: Some(s),
        })
        .collect();
        values.push(ContainerHealth {
            container_health_status: None,
        });
        values
    }
}

impl From<Option<Health>> for ContainerHealth {
    fn from(value: Option<Health>) -> Self {
        let container_health_status = value.and_then(|health| health.status);
        ContainerHealth {
            container_health_status,
        }
    }
}

impl From<ContainerHealth> for Value {
    fn from(value: ContainerHealth) -> Self {
        match value.container_health_status {
            Some(health_status) => format!("{health_status}"),
            None => "null".to_string(),
        }
        .into()
    }
}
