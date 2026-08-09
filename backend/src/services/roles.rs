use crate::{
    errors::{AppError, AuthError},
    repositories::{permissions as permissions_repo, redis, roles as roles_repo},
    structs::auth::AuthenticatedUser,
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions, SetRolePermissions},
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use sqlx::{Pool, Postgres};

/// 指派角色前的守門：任何人都不得把 `super_admin` 指派出去。
///
/// 這條的存在理由：內建 `admin` 角色（baseline 的 role 3「後台全功能」）拿到的是
/// `user:read/create/update/delete`，**刻意沒有** `role:*`——「管帳號」與「管角色」
/// 是分開的兩層。但建立管理員的 API 收 `role_ids`，若不在這裡擋，一般管理員只要
/// 有 `user:create` 就能建一個 super_admin 帳號再登入，整條邊界就沒了。
///
/// 呼叫點：`services/users.rs` 的 create_user 與 set_user_roles（含讀 app_settings
/// 預設角色那條 fallback 路徑）。
pub async fn ensure_assignable(
    pool: &Pool<Postgres>,
    actor: &AuthenticatedUser,
    role_ids: &[i32],
) -> Result<(), AppError> {
    if roles_repo::contains_super_admin(pool, role_ids).await? {
        return Err(AuthError::Forbidden.into());
    }
    // 指派角色 = 把該角色的整組權限交到對方手上。少了這道，`role:assign` + `user:create`
    // 就等於「挑一個現成的高權角色，掛到我自己新建的帳號上，再登入進去」——
    // 連自己造角色都不必，下面 set_role_permissions 那道防線直接被繞過。
    let granting = roles_repo::permission_strings_for_roles(pool, role_ids).await?;
    ensure_no_amplification(actor, &granting, "指派角色")
}

/// 權限放大防護：非 super_admin 不得把自己沒有的權限授出去。
///
/// 這是整條提權鏈的收口。原本 `ensure_not_built_in` 只擋「改內建角色」，擋不住
/// **另外造一個**角色：`role:create` 開新角色 → `role:update` 把權限塞滿 →
/// `user:create` + `role:assign` 建帳號掛上去 → 用那組帳密登入，就拿到了除
/// super_admin 外的全部權限。有了這條，任何人能授出的權限上限就是自己擁有的那組。
///
/// super_admin 直接放行：其權限來自程式（`get_user_permission_strings_by_id` 偵測到
/// 該 role 就回傳整張 permissions 表），本來就是全集。
///
/// ⚠️ 檢查對象是**授出後的整組權限**而非「新增的那些」。所以非 super_admin 也改不動
/// 一個已經比自己大的角色（即使他只是想拿掉某項權限）—— 這個方向是刻意的：
/// 管不了的角色就整個別碰，比「可以改一半」好推理。
fn ensure_no_amplification(
    actor: &AuthenticatedUser,
    granting: &[String],
    action: &str,
) -> Result<(), AppError> {
    if actor.is_super_admin {
        return Ok(());
    }
    let missing: Vec<&str> = granting
        .iter()
        .filter(|p| !actor.permissions.contains(p))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        // 對外只回 403（同 ensure_assignable 的既有行為），細節留在 log ——
        // 這條真的被觸發時多半是提權嘗試，要查得到是誰、想拿什麼。
        tracing::warn!(
            "user {} ({}) 嘗試{}授出自身沒有的權限: {}",
            actor.id,
            actor.name,
            action,
            missing.join(", ")
        );
        return Err(AuthError::Forbidden.into());
    }
    Ok(())
}

pub async fn get_roles(pool: &Pool<Postgres>) -> Result<Vec<Role>, AppError> {
    roles_repo::get_roles(pool).await
}

pub async fn get_role(pool: &Pool<Postgres>, role_id: i32) -> Result<RoleWithPermissions, AppError> {
    roles_repo::get_role_with_permissions(pool, role_id).await
}

pub async fn create_role(pool: &Pool<Postgres>, new_role: NewRole) -> Result<Role, AppError> {
    roles_repo::create_role(pool, &new_role).await
}

pub async fn set_role_permissions(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    actor: &AuthenticatedUser,
    role_id: i32,
    body: SetRolePermissions,
) -> Result<(), AppError> {
    // 內建角色的權限組不可改。delete_role 早就有這道保護，這支漏了 ——
    // 少了它，有 role:update 的人可以把自己所屬角色補滿權限（自我提權）。
    roles_repo::ensure_not_built_in(pool, role_id).await?;
    // 內建角色擋掉了，自訂角色沒有 —— `role:create` 開一個新的照樣能塞滿權限。
    // 授出的權限不得超過自己這組。
    let granting = permissions_repo::permission_strings_by_ids(pool, &body.permission_ids).await?;
    ensure_no_amplification(actor, &granting, "設定角色權限")?;
    let ids = roles_repo::get_ids_by_role_id(pool, role_id).await?;
    roles_repo::set_role_permissions(pool, role_id, &body.permission_ids).await?;
    redis::invalidate_identity_for_ids(redis_pool, &ids).await;
    Ok(())
}

pub async fn delete_role(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    role_id: i32,
) -> Result<(), AppError> {
    let ids = roles_repo::get_ids_by_role_id(pool, role_id).await?;
    roles_repo::delete_role(pool, role_id).await?;
    redis::invalidate_identity_for_ids(redis_pool, &ids).await;
    Ok(())
}

pub async fn get_permissions(pool: &Pool<Postgres>) -> Result<Vec<Permission>, AppError> {
    permissions_repo::get_permissions(pool).await
}

/// `ensure_no_amplification` 是整條提權鏈唯一的純函式部分，也是唯一能不碰 DB 就測的。
#[cfg(test)]
mod tests {
    use super::*;

    fn actor(is_super_admin: bool, permissions: &[&str]) -> AuthenticatedUser {
        AuthenticatedUser {
            id: 7,
            name: "tester".to_string(),
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
            is_super_admin,
        }
    }

    fn granting(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn is_forbidden(r: Result<(), AppError>) -> bool {
        matches!(r, Err(AppError::AuthError(AuthError::Forbidden)))
    }

    #[test]
    fn subset_of_own_permissions_is_allowed() {
        let a = actor(false, &["blog:update", "blog:delete", "image:read"]);
        assert!(ensure_no_amplification(&a, &granting(&["blog:update"]), "測試").is_ok());
        assert!(
            ensure_no_amplification(&a, &granting(&["blog:update", "image:read"]), "測試").is_ok()
        );
    }

    #[test]
    fn granting_permission_actor_lacks_is_forbidden() {
        let a = actor(false, &["blog:update"]);
        // 這是整條鏈的核心：role:create + role:update 造一個比自己大的角色
        assert!(is_forbidden(ensure_no_amplification(
            &a,
            &granting(&["blog:update", "user:create"]),
            "測試"
        )));
        assert!(is_forbidden(ensure_no_amplification(
            &a,
            &granting(&["platform:update"]),
            "測試"
        )));
    }

    #[test]
    fn super_admin_may_grant_anything() {
        // super_admin 的權限來自程式而非 role_permissions，permissions 一律視為全集
        let a = actor(true, &[]);
        assert!(ensure_no_amplification(&a, &granting(&["platform:update"]), "測試").is_ok());
    }

    #[test]
    fn empty_grant_is_allowed_for_anyone() {
        // 建立零權限管理員（`new_user_default_roles` 的預設路徑）不該被這條擋下
        let a = actor(false, &[]);
        assert!(ensure_no_amplification(&a, &granting(&[]), "測試").is_ok());
    }
}
