use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use web_pages::*;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/dashboard", get(dashboard))
        .route("/assistants", get(assistants_page))
        .route("/console", get(console_page))
        .route("/analytics", get(analytics_page))
        .route("/settings", get(settings_page))
}

async fn index() -> impl IntoResponse {
    dashboard().await
}

async fn dashboard() -> impl IntoResponse {
    // 模拟RBAC和团队数据
    let rbac = types::Rbac {
        email: "user@lumosai.com".to_string(),
        first_name: Some("Demo".to_string()),
        last_name: Some("User".to_string()),
        teams: vec![],
        active_role: "Admin".to_string(),
    };
    let team_id = 1;

    let page = rsx! {
        dashboard::DashboardPage {
            team_id: team_id,
            rbac: rbac
        }
    };

    Html(render(page))
}

async fn dashboard_old() -> impl IntoResponse {
    let page = rsx! {
        base_layout::BaseLayout {
            title: "LumosAI Dashboard".to_string(),
            fav_icon_src: "/static/favicon.svg".to_string(),
            collapse_svg_src: "/static/icons/collapse.svg".to_string(),
            stylesheets: vec![
                "/static/index.css".to_string(),
                "/static/output.css".to_string()
            ],
            section_class: "p-6 bg-base-100".to_string(),
            js_href: "/static/index.js".to_string(),

}

async fn assistants_page() -> impl IntoResponse {
    Html("<h1>Assistants Page - Coming Soon</h1>".to_string())
}

async fn console_page() -> impl IntoResponse {
    Html("<h1>Console Page - Coming Soon</h1>".to_string())
}

async fn analytics_page() -> impl IntoResponse {
    Html("<h1>Analytics Page - Coming Soon</h1>".to_string())
}

async fn settings_page() -> impl IntoResponse {
    Html("<h1>Settings Page - Coming Soon</h1>".to_string())
}
