#![allow(non_snake_case)]
use crate::types::{IntegrationStatus, LabelRole};
use dioxus::prelude::*;
use dioxus::prelude::*;

#[component]
pub fn Status(integration_status: IntegrationStatus) -> Element {
    match integration_status {
        IntegrationStatus::Configured => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "Configured" }
        ),
        _ => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "Awaiting Configuration" }
        ),
    }
}
