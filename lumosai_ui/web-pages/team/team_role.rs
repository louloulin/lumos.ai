#![allow(non_snake_case)]
use crate::types::{Role as DBRole, LabelRole};
use daisy_rsx::*;
use dioxus::prelude::*;

#[component]
pub fn Role(role: DBRole) -> Element {
    match role {
        DBRole::SystemAdministrator => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Highlight)}", "System Administrator" }
        ),
        DBRole::TeamManager => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Neutral)}", "Team Manager" }
        ),
        DBRole::Collaborator => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Neutral)}", "Collaborator" }
        ),
        DBRole::Admin => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Highlight)}", "Admin" }
        ),
        DBRole::Member => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Neutral)}", "Member" }
        ),
        DBRole::Viewer => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Neutral)}", "Viewer" }
        ),
    }
}
