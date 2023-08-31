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

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;
use crate::metrics::Metrics;
use crate::monitor::DockerHealthMonitor;

pub mod cli;
pub mod container_health;
pub mod logging;
pub mod meter_attributes;
pub mod metrics;
pub mod monitor;

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> Result<()> {
    logging::init()?;
    let cli = Cli::parse();

    let metrics = Metrics::new()?;
    let meter = metrics.meter_provider();
    let server = tokio::spawn(metrics.run(cli.prometheus_address));

    let interval = cli.restart_interval();
    let monitor = DockerHealthMonitor::new(interval, &meter).await?;
    let (server, monitor) = tokio::join!(server, monitor.run());
    monitor?;
    server??;
    Ok(())
}
