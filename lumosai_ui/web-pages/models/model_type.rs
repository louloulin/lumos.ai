#![allow(non_snake_case)]
use daisy_rsx::*;
use crate::types::ModelType;
use dioxus::prelude::*;

#[component]
pub fn Model(model_type: ModelType) -> Element {
    match model_type {
        ModelType::LLM => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Info,
                "Large Language Model"
            }
        ),
        ModelType::Embeddings => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Highlight,
                "Embeddings Model"
            }
        ),
        ModelType::TextToSpeech => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Warning,
                "Text To Speech"
            }
        ),
        ModelType::Image => rsx!(
            Label {
                class: "truncate",
                label_role: LabelRole::Neutral,
                "Image Generation"
            }
        ),
        ModelType::OpenAI => rsx!(
            Label {
                label_role: LabelRole::Success,
                "OpenAI"
            }
        ),
        ModelType::Anthropic => rsx!(
            Label {
                label_role: LabelRole::Info,
                "Anthropic"
            }
        ),
        ModelType::Local => rsx!(
            Label {
                label_role: LabelRole::Neutral,
                "Local"
            }
        ),
        ModelType::Custom => rsx!(
            Label {
                label_role: LabelRole::Warning,
                "Custom"
            }
        ),
    }
}
