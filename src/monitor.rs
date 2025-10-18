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
use std::fmt::Debug;
use std::time::Duration;

use anyhow::{Result, format_err};
use bollard::Docker;
use bollard::query_parameters::InspectContainerOptions;
use bollard::query_parameters::ListContainersOptionsBuilder;
use bollard::query_parameters::RestartContainerOptions;
use prometheus_client::collector::Collector;
use prometheus_client::encoding::EncodeMetric;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tokio::time;

use crate::container_health::ContainerHealth;
use crate::logging::Informational;
use crate::meter_attributes::ContainerSummaryLabels;

pub struct DockerHealthMonitor {
    docker: Docker,
    restart_interval: Option<Duration>,
    error_counter: Counter,
    restart_counter: Family<ContainerSummaryLabels, Counter>,
    failed_restart_counter: Family<ContainerSummaryLabels, Counter>,
}

#[derive(Debug)]
struct DockerHealthMonitorCollector {
    docker: Docker,
    error_counter: Counter,
}

impl DockerHealthMonitor {
    pub async fn new(
        docker: Docker,
        restart_interval: Option<Duration>,
        registry: &mut Registry,
    ) -> Result<DockerHealthMonitor> {
        let error_counter = Counter::default();
        registry.register("errors", "Docker client errors", error_counter.clone());

        let restart_counter = Family::<ContainerSummaryLabels, Counter>::default();
        registry.register(
            "restarts",
            "Number of successful restarts triggered due to a container being unhealthy",
            restart_counter.clone(),
        );

        let failed_restart_counter = Family::<ContainerSummaryLabels, Counter>::default();
        registry.register(
            "restart_failures",
            "Number of failed restarts triggered due to a container being unhealthy",
            failed_restart_counter.clone(),
        );

        let collector = DockerHealthMonitorCollector {
            docker: docker.clone(),
            error_counter: error_counter.clone(),
        };
        registry.register_collector(Box::new(collector));

        Ok(DockerHealthMonitor {
            docker,
            restart_interval,
            error_counter,
            restart_counter,
            failed_restart_counter,
        })
    }

    async fn health_state(docker: &Docker, container_id: &str) -> Result<ContainerHealth> {
        let container_inspect = docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await?;
        let container_state = container_inspect.state.ok_or(format_err!(
            "Failed to get state from container {container_id}"
        ))?;
        let container_health_status = container_state.health.into();
        Ok(container_health_status)
    }

    async fn check_health_state(
        docker: &Docker,
        mut encoder: prometheus_client::encoding::DescriptorEncoder<'_>,
    ) -> Result<()> {
        let options = ListContainersOptionsBuilder::new().all(true).build();
        let containers = docker.list_containers(Some(options)).await?;
        let family = Family::<ContainerSummaryLabels, Gauge>::default();
        for container in containers {
            let container_id = container
                .id
                .clone()
                .ok_or(format_err!("Failed to get ID from container"))?;
            let container_health_state =
                DockerHealthMonitor::health_state(docker, &container_id).await?;

            for health_status in ContainerHealth::values() {
                let mut labels: ContainerSummaryLabels = container.clone().into();
                labels.health = Some(health_status.clone().into());
                let gauge = family.get_or_create(&labels);
                gauge.set((container_health_state == health_status).into());
            }
        }
        let metric_encoder = encoder.encode_descriptor(
            "health",
            "The current state of the healthcheck",
            None,
            family.metric_type(),
        )?;
        family.encode(metric_encoder)?;
        Ok(())
    }

    async fn restart_unhealthy_containers(&self) -> Result<()> {
        let mut filters = HashMap::new();
        filters.insert("health", vec!["unhealthy"]);
        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&filters)
            .build();
        let unhealthy_containers = self.docker.list_containers(Some(options)).await?;
        for container in unhealthy_containers {
            let container_info = container.info();
            log::info!("Restarting unhealthy container: {container_info}");
            if let Some(id) = &container.id {
                self.docker
                    .restart_container(id, None::<RestartContainerOptions>)
                    .await?;
                self.restart_counter.get_or_create(&container.into()).inc();
                log::info!("Restarted unhealthy container: {container_info}");
            } else {
                self.failed_restart_counter
                    .get_or_create(&container.into())
                    .inc();
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
                    self.error_counter.inc();
                    log::warn!("Failed to restart: {e}")
                }
            }
        }
        Ok(())
    }
}

impl Collector for DockerHealthMonitorCollector {
    fn encode(
        &self,
        encoder: prometheus_client::encoding::DescriptorEncoder,
    ) -> std::result::Result<(), std::fmt::Error> {
        tokio::task::block_in_place(|| {
            futures::executor::block_on(DockerHealthMonitor::check_health_state(
                &self.docker,
                encoder,
            ))
        })
        .map_err(|e| {
            self.error_counter.inc();
            log::error!("HealthCheck failed: {e}");
            std::fmt::Error
        })
    }
}
