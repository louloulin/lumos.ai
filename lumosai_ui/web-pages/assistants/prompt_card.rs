#![allow(non_snake_case)]

use daisy_rsx::*;
use crate::types::{Rbac, Prompt};
use dioxus::prelude::*;

#[component]
pub fn PromptCard(team_id: i32, rbac: Rbac, prompt: Prompt) -> Element {
    let description: String = prompt
        .description
        .as_deref()
        .unwrap_or("No description")
        .chars()
        .filter(|&c| c != '\n' && c != '\t' && c != '\r')
        .collect();
    rsx! {
        Card {
            class: "cursor-pointer hover:bg-base-200 w-full",
            popover_target: format!("view-trigger-{}-{}", prompt.id, team_id),
            CardHeader {
                class: "truncate ellipses flex justify-between p-2",
                title: "{prompt.name}",
                super::visibility::VisLabel {
                    visibility: prompt.visibility
                }
            }
            CardBody {
                class: "m-0 p-2",
                div {
                    class: "flex w-full",
                    if let Some(object_id) = prompt.image_icon_object_id {
                        img {
                            width: "96",
                            height: "96",
                            src: format!("/team/{}/image/{}", team_id, object_id)
                        }
                    } else {
                        Avatar {
                            avatar_size: AvatarSize::Large,
                            avatar_type: AvatarType::User
                        }
                    }
                    div {
                        class: "ml-6 flex flex-col space-between",
                        p {
                            class: "text-sm line-clamp-3",
                            "{description}"
                        }
                        div {
                            class: "text-xs",
                            "Last update ",
                            RelativeTime {
                                format: RelativeTimeFormat::Relative,
                                datetime: "{prompt.updated_at}"
                            }
                        }
                    }
                }
            }
        }
    }
}
