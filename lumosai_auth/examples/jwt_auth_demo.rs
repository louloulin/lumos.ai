//! JWT Auth 完整使用示例
//!
//! 演示真正的 JWT 认证系统的使用

use lumosai_auth::{AuthService, PasswordHashManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 LumosAI JWT 认证系统示例");
    println!("{}", "=".repeat(60));

    // 1. 创建认证服务
    println!("\n📝 步骤 1: 创建认证服务");
    let secret_key = "your-super-secret-key-at-least-32-bytes-long-for-production";
    let auth_service = AuthService::new(secret_key.to_string(), 3600);
    println!("✅ 认证服务创建成功（Token 有效期: 1 小时）");

    // 2. 注册新用户
    println!("\n📝 步骤 2: 注册新用户");
    let user = auth_service
        .register("alice@example.com", "SecurePassword123", None)
        .await?;
    println!("✅ 用户注册成功:");
    println!("   - ID: {}", user.id);
    println!("   - Email: {}", user.email);
    println!("   - Roles: {:?}", user.roles);
    println!("   - Status: {:?}", user.status);

    // 3. 用户登录（获取 JWT Token）
    println!("\n📝 步骤 3: 用户登录");
    let auth_token = auth_service
        .authenticate("alice@example.com", "SecurePassword123")
        .await?;
    println!("✅ 登录成功，获得 JWT Token:");
    println!("   - Type: {}", auth_token.token_type);
    println!("   - Expires in: {} 秒", auth_token.expires_in);
    println!("   - Token (前 50 字符): {}...", &auth_token.token[..50.min(auth_token.token.len())]);
    println!("   - 包含点号（JWT 格式）: {}", auth_token.token.contains('.'));

    // 4. 验证 Token
    println!("\n📝 步骤 4: 验证 Token");
    let validated_user = auth_service.validate_token(&auth_token.token).await?;
    println!("✅ Token 验证成功:");
    println!("   - User ID: {}", validated_user.id);
    println!("   - Email: {}", validated_user.email);
    println!("   - Active: {}", validated_user.is_active());

    // 5. 刷新 Token
    println!("\n📝 步骤 5: 刷新 Token");
    let refreshed_token = auth_service.refresh_token(&auth_token.token).await?;
    println!("✅ Token 刷新成功:");
    println!("   - 新旧 Token 不同: {}", refreshed_token.token != auth_token.token);
    println!("   - 新 Token 可用: {}", 
        auth_service.validate_token(&refreshed_token.token).await.is_ok());

    // 6. 权限检查
    println!("\n📝 步骤 6: 权限检查");
    let has_user_role = auth_service.check_permission(&user.id, "user").await?;
    let has_admin_role = auth_service.check_permission(&user.id, "admin").await?;
    println!("✅ 权限检查:");
    println!("   - Has 'user' role: {}", has_user_role);
    println!("   - Has 'admin' role: {}", has_admin_role);

    // 7. 注册管理员用户
    println!("\n📝 步骤 7: 注册管理员用户");
    let admin = auth_service
        .register(
            "admin@example.com",
            "AdminPassword123",
            Some(vec!["admin".to_string(), "user".to_string()])
        )
        .await?;
    println!("✅ 管理员注册成功:");
    println!("   - Email: {}", admin.email);
    println!("   - Roles: {:?}", admin.roles);

    // 8. 密码哈希演示
    println!("\n📝 步骤 8: 密码哈希演示");
    let password = "MySecurePassword123";
    let hash = PasswordHashManager::hash_password(password)?;
    println!("✅ 密码哈希:");
    println!("   - 原始密码长度: {}", password.len());
    println!("   - 哈希长度: {}", hash.len());
    println!("   - 哈希前缀: {}", &hash[..10]);
    println!("   - 哈希算法: Argon2（比 bcrypt 更安全）");

    let is_valid = PasswordHashManager::verify_password(password, &hash)?;
    println!("   - 验证成功: {}", is_valid);

    // 9. 错误处理演示
    println!("\n📝 步骤 9: 错误处理演示");
    
    // 弱密码
    match auth_service.register("weak@example.com", "weak", None).await {
        Err(e) => println!("✅ 弱密码被拒绝: {}", e),
        Ok(_) => println!("❌ 不应该接受弱密码"),
    }

    // 重复注册
    match auth_service.register("alice@example.com", "Password123", None).await {
        Err(e) => println!("✅ 重复注册被拒绝: {}", e),
        Ok(_) => println!("❌ 不应该允许重复注册"),
    }

    // 错误密码
    match auth_service.authenticate("alice@example.com", "WrongPassword").await {
        Err(e) => println!("✅ 错误密码被拒绝: {}", e),
        Ok(_) => println!("❌ 不应该接受错误密码"),
    }

    // 无效 Token
    match auth_service.validate_token("invalid.token.here").await {
        Err(e) => println!("✅ 无效 Token 被拒绝: {}", e),
        Ok(_) => println!("❌ 不应该接受无效 Token"),
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ 所有示例完成！");
    println!("\n💡 关键特性:");
    println!("   ✅ 真正的 JWT（不是 UUID）");
    println!("   ✅ Argon2 密码哈希（比 bcrypt 更安全）");
    println!("   ✅ Token 签名验证");
    println!("   ✅ Token 过期检查");
    println!("   ✅ Token 刷新机制");
    println!("   ✅ 密码强度验证");
    println!("   ✅ 角色和权限管理");
    println!("\n🔒 安全性:");
    println!("   - Token 有签名，无法伪造");
    println!("   - 密码使用 Argon2 哈希");
    println!("   - Token 自动过期");
    println!("   - 完整的错误处理");

    Ok(())
}

