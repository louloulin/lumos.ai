#![allow(non_snake_case)]
use daisy_rsx::*;

use dioxus::prelude::*;

#[component]
pub fn RateTable(rate_limits: Vec<crate::types::RateLimit>, team_id: i32) -> Element {
    rsx!(
        Card {
            class: "has-data-table mt-6",
            CardHeader {
                title: "Rate Limits"
            }
            CardBody {
                table {
                    class: "table table-sm",
                    thead {
                        th { "API Key ID" }
                        th { "TPM Limit" }
                        th { "RPM Limit" }
                        th {
                            class: "text-right",
                            "Action"
                        }
                    }
                    tbody {
                        for limit in rate_limits {
                            tr {
                                td {
{limit.api_key_id.map(|id| id.to_string()).unwrap_or("N/A".to_string())}
                                }
                                td {
                                    Label {
                                        label_role: LabelRole::Success,
{limit.tpm_limit.map(|l| l.to_string()).unwrap_or("N/A".to_string())}
                                    }
                                }
                                td {
                                    Label {
                                        label_role: LabelRole::Success,
{limit.rpm_limit.map(|l| l.to_string()).unwrap_or("N/A".to_string())}
                                    }
                                }
                                td {
                                    class: "text-right",
                                    DropDown {
                                        direction: Direction::Left,
                                        button_text: "...",
                                        DropDownLink {
                                            popover_target: format!("delete-trigger-{}-{}",
                                            limit.id, team_id),
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
