//! E2E 测试：Auth 认证流程

#[path = "framework.rs"]
mod framework;
use framework::E2ETestContext;

/// 测试 6: 完整的注册和登录流程
#[tokio::test]
async fn test_complete_auth_flow() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 1. 注册用户
    let user = ctx.register_test_user("e2e_user@test.com", "TestPassword123").await;
    assert!(user.is_ok(), "User registration should succeed");

    let user = user.unwrap();
    assert_eq!(user.email, "e2e_user@test.com");
    assert!(user.is_active());

    // 2. 登录获取 Token
    let token = ctx.login_test_user("e2e_user@test.com", "TestPassword123").await;
    assert!(token.is_ok(), "Login should succeed");

    let token = token.unwrap();
    assert!(!token.is_empty());
    assert!(token.contains('.'), "Should be valid JWT format");

    // 3. 验证 Token
    let validated = ctx.auth.validate_token(&token).await;
    assert!(validated.is_ok(), "Token validation should succeed");

    let validated_user = validated.unwrap();
    assert_eq!(validated_user.email, "e2e_user@test.com");

    ctx.teardown().await.unwrap();
}

/// 测试 7: Auth 错误处理
#[tokio::test]
async fn test_auth_error_handling() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 1. 弱密码应该被拒绝
    let result = ctx.register_test_user("weak@test.com", "weak").await;
    assert!(result.is_err(), "Weak password should be rejected");

    // 2. 注册成功后
    ctx.register_test_user("test@test.com", "StrongPass123").await.unwrap();

    // 3. 重复注册应该失败
    let result = ctx.register_test_user("test@test.com", "AnotherPass123").await;
    assert!(result.is_err(), "Duplicate registration should fail");

    // 4. 错误密码应该失败
    let result = ctx.login_test_user("test@test.com", "WrongPassword").await;
    assert!(result.is_err(), "Wrong password should fail");

    // 5. 不存在的用户应该失败
    let result = ctx.login_test_user("nonexistent@test.com", "Password123").await;
    assert!(result.is_err(), "Non-existent user should fail");

    ctx.teardown().await.unwrap();
}

/// 测试 8: Token 刷新流程
#[tokio::test]
async fn test_token_refresh() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 注册和登录
    ctx.register_test_user("refresh@test.com", "TestPass123").await.unwrap();
    let token = ctx.login_test_user("refresh@test.com", "TestPass123").await.unwrap();

    // 刷新 Token
    let refreshed = ctx.auth.refresh_token(&token).await;
    assert!(refreshed.is_ok(), "Token refresh should succeed");

    let new_token = refreshed.unwrap();
    assert_ne!(token, new_token.token, "New token should be different");

    // 新 Token 应该可用
    let validated = ctx.auth.validate_token(&new_token.token).await;
    assert!(validated.is_ok(), "Refreshed token should be valid");

    ctx.teardown().await.unwrap();
}

/// 测试 9: 权限检查
#[tokio::test]
async fn test_permission_check() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建普通用户
    let user = ctx.register_test_user("user@test.com", "UserPass123").await.unwrap();

    // 检查默认权限
    let has_user_role = ctx.auth.check_permission(&user.id, "user").await.unwrap();
    assert!(has_user_role, "User should have 'user' role");

    let has_admin_role = ctx.auth.check_permission(&user.id, "admin").await.unwrap();
    assert!(!has_admin_role, "User should not have 'admin' role");

    ctx.teardown().await.unwrap();
}




