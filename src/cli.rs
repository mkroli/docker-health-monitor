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

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use clap::ColorChoice;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    propagate_version = true,
    color = ColorChoice::Auto,
)]
pub struct Cli {
    #[arg(
        long = "prometheus",
        env = "DHM_PROMETHEUS_ADDRESS",
        value_name = "ADDRESS",
        default_value_t = SocketAddr::new(IpAddr::from(Ipv4Addr::UNSPECIFIED), 9092),
    )]
    pub prometheus_address: SocketAddr,
    #[arg(
        long = "restart-interval",
        env = "DHM_RESTART_INTERVAL",
        value_name = "MILLISECONDS"
    )]
    pub restart_interval: Option<u64>,
}

impl Cli {
    pub fn restart_interval(&self) -> Option<Duration> {
        self.restart_interval.map(Duration::from_millis)
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
