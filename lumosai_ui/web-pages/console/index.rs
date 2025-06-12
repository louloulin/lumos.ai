#![allow(non_snake_case)]
use crate::app_layout::SideBar;
use crate::console::model_popup::ModelPopup;
use crate::types::{Rbac, Capability, Prompt, SinglePrompt, BionicToolDefinition};
use dioxus::prelude::*;

pub fn new_conversation(
    team_id: i32,
    prompts: Vec<Prompt>,
    prompt: SinglePrompt,
    rbac: Rbac,
    capabilities: Vec<Capability>,
    enabled_tools: Vec<String>,
    available_tools: Vec<BionicToolDefinition>,
) -> String {
    // Rerverse it because that's how we display it.
    crate::render(rsx! {
        super::layout::ConsoleLayout {
            team_id,
            rbac: rbac.clone(),
            prompt: prompt.clone(),
            title: "AI Chat Console",
            selected_item: SideBar::Console,
            chat_history: vec![],
            pending_chat_state: super::PendingChatState::None,
            is_tts_disabled: true,
            capabilities,
            enabled_tools,
            available_tools,
            header: rsx!(
                Head {
                    team_id: team_id,
                    rbac: rbac.clone(),
                    prompts,
                    prompt: prompt.clone()
                }
            )
        }
    })
}

#[component]
fn Head(team_id: i32, rbac: Rbac, prompts: Vec<Prompt>, prompt: SinglePrompt) -> Element {
    rsx! {
        ModelPopup {
            team_id,
            current_model: prompt.name,
            on_close: move |_| {}
        }
    }
}
