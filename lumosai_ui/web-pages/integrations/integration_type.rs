#![allow(non_snake_case)]
use crate::types::{IntegrationType, LabelRole};
use dioxus::prelude::*;
use dioxus::prelude::*;

#[component]
pub fn Integration(integration_type: IntegrationType) -> Element {
    match integration_type {
        IntegrationType::McpServer => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "MCP Server" }
        ),
        IntegrationType::BuiltIn => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "Built In" }
        ),
        IntegrationType::OpenAPI => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "Open API" }
        ),
        IntegrationType::OAuth2 => rsx!(
            span { class: crate::role_class(LabelRole::Info), "OAuth2" }
        ),
        IntegrationType::ApiKey => rsx!(
            span { class: crate::role_class(LabelRole::Success), "API Key" }
        ),
        IntegrationType::Custom => rsx!(
            span { class: crate::role_class(LabelRole::Warning), "Custom" }
        ),
    }
}
