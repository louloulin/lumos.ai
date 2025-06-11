#![allow(non_snake_case)]
use daisy_rsx::*;
use crate::types::{ModelWithPrompt, ModelType};
use dioxus::prelude::*;

#[component]
pub fn ModelTable(models: Vec<ModelWithPrompt>, team_id: i32) -> Element {
    rsx!(
        Card {
            class: "has-data-table",
            CardHeader {
                title: "Models"
            }
            CardBody {
                table {
                    class: "table table-sm",
                    thead {
                        th { "Name" }
                        th {
                            class: "max-sm:hidden",
                            "Base URL"
                        }
                        th { "Model Type" }

                        th {
                            class: "max-sm:hidden",
                            "TPM Limit"
                        }
                        th {
                            class: "max-sm:hidden",
                            "RPM Limit"
                        }
                        th {
                            class: "max-sm:hidden",
                            "Context Length"
                        }
                        th {
                            "Edit"
                        }
                        th {
                            class: "text-right",
                            "Action"
                        }
                    }
                    tbody {
                        for model in &models {
                            tr {
                                td {
                                    strong {
                                        "{model.name}"
                                    }
                                }
                                td {
                                    class: "max-sm:hidden",
                                    code {
                                        class: "wrap-anywhere",
{model.base_url.as_deref().unwrap_or("N/A")}
                                    }
                                }
                                td {
                                    super::model_type::Model {
                                        model_type: ModelType::Custom // Default fallback
                                    }
                                }
                                td {
                                    class: "max-sm:hidden",
{model.tpm_limit.map(|x| x.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                }
                                td {
                                    class: "max-sm:hidden",
{model.rpm_limit.map(|x| x.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                }
                                td {
                                    class: "max-sm:hidden",
{model.context_size.map(|x| x.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                }
                                td {
                                    Button {
                                        popover_target: format!("edit-model-form-{}", model.id),
                                        button_scheme: ButtonScheme::Neutral,
                                        "Edit"
                                    }
                                }
                                td {
                                    class: "text-right",
                                    DropDown {
                                        direction: Direction::Left,
                                        button_text: "...",
                                        DropDownLink {
                                            popover_target: format!("delete-trigger-{}-{}",
                                                model.id, team_id),
                                            href: "#",
                                            target: "_top",
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
