#![allow(non_snake_case)]
use crate::types::{Visibility, LabelRole};
use daisy_rsx::*;
use dioxus::prelude::*;

#[component]
pub fn VisLabel(visibility: Visibility) -> Element {
    match visibility {
        Visibility::Company => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Danger)}", "{crate::visibility_to_string(visibility)}" }
        ),
        Visibility::Private => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Highlight)}", "{crate::visibility_to_string(visibility)}" }
        ),
        Visibility::Team => rsx!(
            span { class: "mr-2 {crate::role_class(LabelRole::Info)}", "{crate::visibility_to_string(visibility)}" }
        ),
    }
}
