//! Authentication System Demo
//! 
//! This example demonstrates the comprehensive authentication and authorization
//! capabilities of Lumos.ai's enterprise-grade security system.

use lumosai_core::auth::{
    AuthConfig, AuthManager, 
    jwt::JWTManager,
    rbac::RBACManager,
    api_keys::ApiKeyManager,
    session::SessionManager,
    oauth2::{OAuth2Manager, OAuth2Provider},
    multi_tenant::{TenantManager, SubscriptionPlan},
};
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lumos.ai Enterprise Authentication System Demo");
    println!("================================================\n");

    // 1. Basic Authentication Manager Demo
    println!("1️⃣ Basic Authentication Manager");
    println!("--------------------------------");
    
    let config = AuthConfig::default();
    let auth_manager = AuthManager::new(config);
    
    // Create a user
    let user = auth_manager.create_user("demo@lumos.ai", "secure_password", None).await?;
    println!("✅ Created user: {}", user.email);
    
    // Authenticate user
    let auth_token = auth_manager.authenticate("demo@lumos.ai", "secure_password").await?;
    println!("✅ Authentication successful, token: {}...", &auth_token.token[..20]);
    
    // Validate token
    let validated_user = auth_manager.validate_token(&auth_token.token).await?;
    println!("✅ Token validation successful for user: {}", validated_user.email);
    
    println!();

    // 2. JWT Token Management Demo
    println!("2️⃣ JWT Token Management");
    println!("------------------------");
    
    let jwt_manager = JWTManager::new("demo-secret".to_string(), Duration::from_secs(3600));
    let jwt_token = jwt_manager.generate_token(&user).await?;
    println!("✅ Generated JWT token: {}...", &jwt_token.token[..30]);
    
    let jwt_user = jwt_manager.validate_token(&jwt_token.token).await?;
    println!("✅ JWT validation successful for user: {}", jwt_user.email);
    
    let refreshed_token = jwt_manager.refresh_token(&jwt_token.token).await?;
    println!("✅ Token refreshed: {}...", &refreshed_token.token[..30]);
    
    println!();

    // 3. RBAC System Demo
    println!("3️⃣ Role-Based Access Control (RBAC)");
    println!("------------------------------------");
    
    let mut rbac = RBACManager::new();
    let user_id = user.id;
    
    // Assign roles
    rbac.assign_role(&user_id, "developer").await?;
    println!("✅ Assigned 'developer' role to user");
    
    // Check permissions
    let can_create_agents = rbac.check_permission(&user_id, "agents:create").await?;
    println!("✅ Can create agents: {}", can_create_agents);
    
    let can_admin_delete = rbac.check_permission(&user_id, "admin:delete").await?;
    println!("✅ Can admin delete: {}", can_admin_delete);
    
    // Assign admin role
    rbac.assign_role(&user_id, "admin").await?;
    println!("✅ Assigned 'admin' role to user");
    
    let can_admin_delete_now = rbac.check_permission(&user_id, "admin:delete").await?;
    println!("✅ Can admin delete now: {}", can_admin_delete_now);
    
    println!();

    // 4. API Key Management Demo
    println!("4️⃣ API Key Management");
    println!("----------------------");
    
    let mut api_manager = ApiKeyManager::new(Duration::from_secs(7200));
    
    // Generate API key
    let api_key = api_manager.generate_key(user_id, "demo-key", vec!["agents:read".to_string(), "tools:execute".to_string()]).await?;
    println!("✅ Generated API key: {}", api_key);
    
    // Validate API key
    let api_user = api_manager.validate_key(&api_key).await?;
    println!("✅ API key validation successful for user: {}", api_user.id);
    
    // List user keys
    let user_keys = api_manager.list_user_keys(&user_id).await;
    println!("✅ User has {} API keys", user_keys.len());
    
    println!();

    // 5. Session Management Demo
    println!("5️⃣ Session Management");
    println!("----------------------");
    
    let mut session_manager = SessionManager::new(Duration::from_secs(1800));
    
    // Create session
    let session_id = session_manager.create_session(user_id).await?;
    println!("✅ Created session: {}", session_id);
    
    // Validate session
    let session_user = session_manager.validate_session(&session_id).await?;
    println!("✅ Session validation successful for user: {}", session_user.id);
    
    // List user sessions
    let user_sessions = session_manager.list_user_sessions(&user_id).await;
    println!("✅ User has {} active sessions", user_sessions.len());
    
    println!();

    // 6. OAuth2 Integration Demo
    println!("6️⃣ OAuth2 Integration");
    println!("----------------------");
    
    let mut oauth2_manager = OAuth2Manager::new();
    
    // Add Google provider
    let google_provider = OAuth2Provider::google(
        "demo-client-id".to_string(),
        "demo-client-secret".to_string(),
        "http://localhost:3000/auth/callback".to_string(),
    );
    oauth2_manager.add_provider(google_provider);
    
    // Add GitHub provider
    let github_provider = OAuth2Provider::github(
        "demo-github-client".to_string(),
        "demo-github-secret".to_string(),
        "http://localhost:3000/auth/github/callback".to_string(),
    );
    oauth2_manager.add_provider(github_provider);
    
    let providers = oauth2_manager.list_providers();
    println!("✅ Configured {} OAuth2 providers: {:?}", providers.len(), providers.iter().map(|p| &p.name).collect::<Vec<_>>());
    
    // Start authorization flow
    let auth_url = oauth2_manager.start_authorization("google", Some("http://localhost:3000/dashboard".to_string())).await?;
    println!("✅ Google OAuth2 authorization URL: {}...", &auth_url[..50]);
    
    println!();

    // 7. Multi-Tenant System Demo
    println!("7️⃣ Multi-Tenant System");
    println!("-----------------------");
    
    let mut tenant_manager = TenantManager::new();
    let owner_id = Uuid::new_v4();
    
    // Create tenants with different plans
    let startup_tenant = tenant_manager.create_tenant(
        "Startup Corp".to_string(),
        owner_id,
        SubscriptionPlan::Starter,
    ).await?;
    println!("✅ Created startup tenant: {} (Plan: {:?})", startup_tenant.name, startup_tenant.subscription_plan);
    
    let enterprise_tenant = tenant_manager.create_tenant(
        "Enterprise Inc".to_string(),
        owner_id,
        SubscriptionPlan::Enterprise,
    ).await?;
    println!("✅ Created enterprise tenant: {} (Plan: {:?})", enterprise_tenant.name, enterprise_tenant.subscription_plan);
    
    // Check resource limits
    let startup_can_add_users = tenant_manager.check_resource_limit(&startup_tenant.id, "users", 20).await?;
    let startup_cannot_add_many = tenant_manager.check_resource_limit(&startup_tenant.id, "users", 50).await?;
    println!("✅ Startup can add 20 users: {}, can add 50 users: {}", startup_can_add_users, startup_cannot_add_many);
    
    let enterprise_can_add_many = tenant_manager.check_resource_limit(&enterprise_tenant.id, "users", 500).await?;
    println!("✅ Enterprise can add 500 users: {}", enterprise_can_add_many);
    
    // Set custom domain
    tenant_manager.set_tenant_domain(&enterprise_tenant.id, "enterprise.lumos.ai".to_string()).await?;
    println!("✅ Set custom domain for enterprise tenant");
    
    let found_tenant = tenant_manager.get_tenant_by_domain("enterprise.lumos.ai").await;
    println!("✅ Found tenant by domain: {}", found_tenant.unwrap().name);
    
    // Update usage
    tenant_manager.update_usage(&startup_tenant.id, "users", 15).await?;
    tenant_manager.update_usage(&startup_tenant.id, "agents", 25).await?;
    
    let usage = tenant_manager.get_usage(&startup_tenant.id).await.unwrap();
    println!("✅ Startup tenant usage - Users: {}, Agents: {}", usage.current_users, usage.current_agents);
    
    println!();

    // 8. Security Features Demo
    println!("8️⃣ Security Features");
    println!("---------------------");
    
    // Test invalid credentials
    let invalid_auth = auth_manager.authenticate("invalid@email.com", "wrong_password").await;
    println!("✅ Invalid authentication properly rejected: {}", invalid_auth.is_err());
    
    // Test invalid token
    let invalid_token = auth_manager.validate_token("invalid_token").await;
    println!("✅ Invalid token properly rejected: {}", invalid_token.is_err());
    
    // Test invalid API key
    let invalid_api_key = api_manager.validate_key("invalid_api_key").await;
    println!("✅ Invalid API key properly rejected: {}", invalid_api_key.is_err());
    
    // Test invalid session
    let invalid_session = session_manager.validate_session("invalid_session").await;
    println!("✅ Invalid session properly rejected: {}", invalid_session.is_err());
    
    println!();

    // 9. Performance and Statistics
    println!("9️⃣ Performance and Statistics");
    println!("------------------------------");
    
    let session_stats = session_manager.get_stats();
    println!("✅ Session statistics - Total: {}, Active: {}", session_stats.total_sessions, session_stats.active_sessions);
    
    let all_roles = rbac.list_roles();
    println!("✅ Available roles: {:?}", all_roles.iter().map(|r| &r.name).collect::<Vec<_>>());
    
    let all_permissions = rbac.list_permissions();
    println!("✅ Available permissions: {}", all_permissions.len());
    
    println!();

    println!("🎉 Authentication System Demo Completed Successfully!");
    println!("=====================================================");
    println!();
    println!("✨ Key Features Demonstrated:");
    println!("   • JWT token generation and validation");
    println!("   • Role-based access control (RBAC)");
    println!("   • API key management");
    println!("   • Session management");
    println!("   • OAuth2 integration");
    println!("   • Multi-tenant architecture");
    println!("   • Security validation");
    println!("   • Performance monitoring");
    println!();
    println!("🔒 Enterprise-grade security features are fully operational!");

    Ok(())
}
