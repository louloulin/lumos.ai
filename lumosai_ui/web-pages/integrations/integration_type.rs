#![allow(non_snake_case)]
use daisy_rsx::*;
use crate::types::IntegrationType;
use dioxus::prelude::*;

#[component]
pub fn Integration(integration_type: IntegrationType) -> Element {
    match integration_type {
        IntegrationType::McpServer => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "MCP Server"
            }
        ),
        IntegrationType::BuiltIn => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "Built In"
            }
        ),
        IntegrationType::OpenAPI => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "Open API"
            }
        ),
        IntegrationType::OAuth2 => rsx!(
            Label {
                label_role: LabelRole::Info,
                "OAuth2"
            }
        ),
        IntegrationType::ApiKey => rsx!(
            Label {
                label_role: LabelRole::Success,
                "API Key"
            }
        ),
        IntegrationType::Custom => rsx!(
            Label {
                label_role: LabelRole::Warning,
                "Custom"
            }
        ),
    }
}
