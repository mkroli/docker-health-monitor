# Docker-Health-Monitor

[![Build](https://github.com/mkroli/docker-health-monitor/actions/workflows/build.yml/badge.svg)](https://github.com/mkroli/docker-health-monitor/actions/workflows/build.yml)

Prometheus exporter of docker container's health checks with the option to restart unhealthy containers.

## Installation

## Binaries
[Latest Release](https://github.com/mkroli/docker-health-monitor/releases/latest)

## docker-compose
```yaml
version: '3'

services:
  docker-health-monitor:
    image: ghcr.io/mkroli/docker-health-monitor
    restart: always
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    ports:
      - 9092:9092
```

## Usage
```
Prometheus exporter of docker container's health checks with the option to restart unhealthy containers.

Usage: docker-health-monitor [OPTIONS]

Options:
      --prometheus <ADDRESS>             [env: DHM_PROMETHEUS_ADDRESS=] [default: 0.0.0.0:9092]
      --restart-interval <MILLISECONDS>  [env: DHM_RESTART_INTERVAL=]
  -h, --help                             Print help
  -V, --version                          Print version
```
