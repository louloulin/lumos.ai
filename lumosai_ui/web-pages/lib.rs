/*!
# LumosAI Web Pages

Core UI components for LumosAI applications, based on bionic-gpt's proven design patterns.

This library provides modern web components built with Dioxus and DaisyUI.
*/

use dioxus::prelude::Element;

// Mock types for UI components
pub mod types;
pub use types::*;

// Core layout components
pub mod app_layout;
pub mod base_layout;
pub mod menu;
pub mod confirm_modal;
pub mod snackbar;
pub mod hero;
pub mod profile;
pub mod profile_popup;
pub mod logout_form;
pub mod charts;

// Feature modules (UI only)
pub mod analytics;
pub mod api_keys;
pub mod assistants;
pub mod audit_trail;
pub mod console;
pub mod dashboard;
pub mod datasets;
pub mod documents;
pub mod history;
pub mod integrations;
pub mod models;
pub mod my_assistants;
pub mod pipelines;
pub mod rate_limits;
pub mod settings;
pub mod team;
pub mod teams;
pub mod workflows;

// Re-export commonly used components
pub use confirm_modal::ConfirmModal;

// Utility functions
pub fn render(page: Element) -> String {
    let html = dioxus_ssr::render_element(page);
    format!("<!DOCTYPE html><html lang='en'>{}</html>", html)
}

// Routes module
pub mod routes;
