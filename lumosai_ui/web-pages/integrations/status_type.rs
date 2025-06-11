#![allow(non_snake_case)]
use daisy_rsx::*;
use crate::types::IntegrationStatus;
use dioxus::prelude::*;

#[component]
pub fn Status(integration_status: IntegrationStatus) -> Element {
    match integration_status {
        IntegrationStatus::Configured => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "Configured"
            }
        ),
        _ => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "Awaiting Configuration"
            }
        ),
    }
}
