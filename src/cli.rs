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

use anyhow::Result;
use bollard::{API_DEFAULT_VERSION, Docker};
use clap::{Args, ColorChoice, Parser};

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
        value_name = "ADDRESS",
        default_value_t = SocketAddr::new(IpAddr::from(Ipv4Addr::UNSPECIFIED), 9092),
        env = "DHM_PROMETHEUS_ADDRESS",
    )]
    pub prometheus_address: SocketAddr,
    #[arg(
        long = "restart-interval",
        value_name = "MILLISECONDS",
        env = "DHM_RESTART_INTERVAL"
    )]
    pub restart_interval: Option<u64>,
    #[command(flatten, next_help_heading = "Docker connection")]
    pub connection: DockerConnection,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct DockerConnection {
    #[arg(
        long = "unix-socket",
        value_name = "PATH",
        env = "DHM_DOCKER_UNIX_SOCKET"
    )]
    unix_socket: Option<String>,
    #[arg(long = "http", value_name = "URL", env = "DHM_DOCKER_HTTP_URL")]
    http_url: Option<String>,
}

impl DockerConnection {
    fn unix_connection(&self) -> Option<Result<Docker>> {
        self.unix_socket
            .as_ref()
            .map(|path| Docker::connect_with_unix(path, 3, API_DEFAULT_VERSION).map_err(Into::into))
    }

    fn http_connection(&self) -> Option<Result<Docker>> {
        self.http_url.as_ref().map(|http_url| {
            Docker::connect_with_http(http_url, 3, API_DEFAULT_VERSION).map_err(Into::into)
        })
    }

    fn default_connection() -> Result<Docker> {
        Docker::connect_with_local_defaults().map_err(Into::into)
    }

    pub fn connect(&self) -> Result<Docker> {
        let docker = self
            .unix_connection()
            .or_else(|| self.http_connection())
            .unwrap_or_else(DockerConnection::default_connection)?;
        Ok(docker)
    }
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
