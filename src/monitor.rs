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

use std::collections::HashMap;
use std::time::Duration;

use crate::container_health::ContainerHealth;
use anyhow::{format_err, Result};
use bollard::container::ListContainersOptions;
use bollard::Docker;
use opentelemetry::metrics::{Counter, Meter, ObservableGauge, Observer};
use opentelemetry::KeyValue;
use tokio::time;

use crate::logging::Informational;
use crate::meter_attributes::MeterAttributes;

pub struct DockerHealthMonitor {
    docker: Docker,
    restart_interval: Option<Duration>,
    restart_counter: Counter<u64>,
    failed_restart_counter: Counter<u64>,
}

impl DockerHealthMonitor {
    pub async fn new(
        restart_interval: Option<Duration>,
        meter: &Meter,
    ) -> Result<DockerHealthMonitor> {
        let docker = Docker::connect_with_local_defaults()?;

        let container_health = meter
            .u64_observable_gauge("dhm.health")
            .with_description("The current state of the healthcheck")
            .init();
        let d = docker.clone();
        meter.register_callback(&[container_health.as_any()], move |observer| {
            if let Err(e) = tokio::task::block_in_place(|| {
                futures::executor::block_on(DockerHealthMonitor::check_health_state(
                    &d,
                    observer,
                    &container_health,
                ))
            }) {
                log::error!("HealthCheck failed: {e}")
            }
        })?;

        let restart_counter = meter
            .u64_counter("dhm.restarts")
            .with_description(
                "Number of successful restarts triggered due to a container being unhealthy",
            )
            .init();
        let failed_restart_counter = meter
            .u64_counter("dhm.restart_failures")
            .with_description(
                "Number of failed restarts triggered due to a container being unhealthy",
            )
            .init();

        Ok(DockerHealthMonitor {
            docker,
            restart_interval,
            restart_counter,
            failed_restart_counter,
        })
    }

    async fn health_state(docker: &Docker, container_id: &str) -> Result<ContainerHealth> {
        let container_inspect = docker.inspect_container(container_id, None).await?;
        let container_state = container_inspect.state.ok_or(format_err!(
            "Failed to get state from container {container_id}"
        ))?;
        let container_health_status = container_state.health.into();
        Ok(container_health_status)
    }

    async fn check_health_state(
        docker: &Docker,
        observer: &dyn Observer,
        container_health_gauge: &ObservableGauge<u64>,
    ) -> Result<()> {
        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };
        let containers = docker.list_containers(Some(options)).await?;
        for container in containers {
            let container_id = container
                .id
                .clone()
                .ok_or(format_err!("Failed to get ID from container"))?;
            let container_health_state =
                DockerHealthMonitor::health_state(docker, &container_id).await?;

            let mut attributes = container.attributes();
            for health_status in ContainerHealth::values() {
                attributes.push(KeyValue::new("health", health_status.clone()));
                let value = container_health_state == health_status;
                observer.observe_u64(container_health_gauge, value.into(), &attributes);
                attributes.pop();
            }
        }
        Ok(())
    }

    async fn restart_unhealthy_containers(&self) -> Result<()> {
        let mut filters = HashMap::new();
        filters.insert("health", vec!["unhealthy"]);
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        let unhealthy_containers = self.docker.list_containers(Some(options)).await?;
        for container in unhealthy_containers {
            let container_info = container.info();
            log::info!("Restarting unhealthy container: {container_info}");
            if let Some(id) = &container.id {
                self.docker.restart_container(id, None).await?;
                self.restart_counter.add(1, &container.attributes());
                log::info!("Restarted unhealthy container: {container_info}");
            } else {
                self.failed_restart_counter.add(1, &container.attributes());
                log::warn!(
                    "Failed to restart unhealthy container due to missing ID: {container_info}"
                );
            }
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let interval = self.restart_interval.map(time::interval);
        if let Some(mut interval) = interval {
            loop {
                interval.tick().await;
                if let Err(e) = self.restart_unhealthy_containers().await {
                    log::warn!("Failed to restart: {e}")
                }
            }
        }
        Ok(())
    }
}
