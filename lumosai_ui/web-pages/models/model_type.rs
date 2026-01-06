#![allow(non_snake_case)]
use crate::types::{ModelType, LabelRole};
use daisy_rsx::*;
use dioxus::prelude::*;

#[component]
pub fn Model(model_type: ModelType) -> Element {
    match model_type {
        ModelType::LLM => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Info)}", "Large Language Model" }
        ),
        ModelType::Embeddings => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Highlight)}", "Embeddings Model" }
        ),
        ModelType::TextToSpeech => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Warning)}", "Text To Speech" }
        ),
        ModelType::Image => rsx!(
            span { class: "truncate {crate::role_class(LabelRole::Neutral)}", "Image Generation" }
        ),
        ModelType::OpenAI => rsx!(
            span { class: crate::role_class(LabelRole::Success), "OpenAI" }
        ),
        ModelType::Anthropic => rsx!(
            span { class: crate::role_class(LabelRole::Info), "Anthropic" }
        ),
        ModelType::Local => rsx!(
            span { class: crate::role_class(LabelRole::Neutral), "Local" }
        ),
        ModelType::Custom => rsx!(
            span { class: crate::role_class(LabelRole::Warning), "Custom" }
        ),
    }
}
