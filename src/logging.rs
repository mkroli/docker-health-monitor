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

use bollard::models::ContainerSummary;
use log::SetLoggerError;

pub fn init() -> Result<(), SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%3f]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stderr())
        .level(log::LevelFilter::Info)
        .apply()
}

pub trait Informational {
    fn info(&self) -> String;
}

impl Informational for ContainerSummary {
    fn info(&self) -> String {
        let container_names_desc = self.names.clone().unwrap_or_default().join(", ");
        let container_id_desc = self.id.clone().unwrap_or("n/a".to_string());
        format!("{container_names_desc} ({container_id_desc})")
    }
}
