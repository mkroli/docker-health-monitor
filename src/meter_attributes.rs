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
use opentelemetry::{KeyValue, Value};

pub trait OptionalAttribute {
    fn attribute(self, key: &str) -> Option<KeyValue>;
}

impl<T> OptionalAttribute for Option<T>
where
    Value: From<T>,
{
    fn attribute(self, key: &str) -> Option<KeyValue> {
        self.map(|value| KeyValue::new(key.to_string(), value))
    }
}

pub trait MeterAttributes {
    fn attributes(self) -> Vec<KeyValue>;
}

impl MeterAttributes for ContainerSummary {
    fn attributes(self) -> Vec<KeyValue> {
        let id = self.id.attribute("id");
        let name = self
            .names
            .and_then(|names| names.first().cloned())
            .map(|name| name.strip_prefix('/').map_or(name.clone(), String::from))
            .attribute("name");
        let image = self.image.attribute("image");

        let labels: Vec<KeyValue> = self
            .labels
            .map(|labels| {
                labels
                    .into_iter()
                    .map(|(k, v)| KeyValue::new(format!("label_{k}"), v))
                    .collect()
            })
            .unwrap_or(Vec::new());

        let mut attributes: Vec<KeyValue> = vec![id, name, image].into_iter().flatten().collect();
        attributes.extend(labels);
        attributes
    }
}
