#![allow(non_snake_case)]
use daisy_rsx::*;
use crate::types::Role as DBRole;
use dioxus::prelude::*;

#[component]
pub fn Role(role: DBRole) -> Element {
    match role {
        DBRole::SystemAdministrator => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Highlight,
                "System Administrator"
            }
        ),
        DBRole::TeamManager => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Neutral,
                "Team Manager"
            }
        ),
        DBRole::Collaborator => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Neutral,
                "Collaborator"
            }
        ),
        DBRole::Admin => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Highlight,
                "Admin"
            }
        ),
        DBRole::Member => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Neutral,
                "Member"
            }
        ),
        DBRole::Viewer => rsx!(
            Label {
                class: "mr-2",
                label_role: LabelRole::Neutral,
                "Viewer"
            }
        ),
    }
}
