#![allow(non_snake_case)]
use super::actions_section::ActionsSection;
use super::connections_section::ConnectionsSection;
use super::integration_header::IntegrationHeader;
use crate::app_layout::{Layout, SideBar};
use daisy_rsx::*;
use crate::types::{Rbac, ApiKeyConnection, Integration, Oauth2Connection, BionicOpenAPI, BionicToolDefinition};
use dioxus::prelude::*;

pub fn view(
    team_id: i32,
    rbac: Rbac,
    integration: Integration,
    tool_definitions: Vec<BionicToolDefinition>,
    openapi: BionicOpenAPI,
    api_key_connections: Vec<ApiKeyConnection>,
    oauth2_connections: Vec<Oauth2Connection>,
) -> String {
    let page = rsx! {
        Layout {
            section_class: "p-4",
            selected_item: SideBar::Integrations,
            team_id: team_id,
            rbac: rbac.clone(),
            title: "Integrations",
            header: rsx!(
                Breadcrumb {
                    items: vec![
                            BreadcrumbItem {
                            text: "Integrations".into(),
                            href: Some(crate::routes::integrations::Index { team_id }.to_string())
                        },
                        BreadcrumbItem {
                            text: integration.name.clone(),
                            href: None
                        }
                    ]
                }
            ),

            div {
                class: "p-4 max-w-3xl w-full mx-auto",

                IntegrationHeader {
                    team_id,
                    rbac: rbac.clone(),
                    integration: integration.clone(),
                    logo_url: openapi.clone().get_logo_url(),
                    description: openapi.clone().get_description().map(|s| s.to_string())
                }

                hr {
                    class: "mt-5 mb-5"
                }

                ConnectionsSection {
                    team_id,
                    integration_id: integration.id,
                    rbac,
                    openapi: openapi.clone(),
                    api_key_connections,
                    oauth2_connections
                }

                ActionsSection {
                    logo_url: openapi.clone().get_logo_url(),
                    tool_definitions
                }
            }
        }
    };

    crate::render(page)
}
