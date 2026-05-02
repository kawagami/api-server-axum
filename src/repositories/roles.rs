use crate::{
    errors::AppError,
    state::AppState,
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions},
};

pub async fn get_roles(state: &AppState) -> Result<Vec<Role>, AppError> {
    Ok(
        sqlx::query_as("SELECT id, name, description FROM roles ORDER BY id")
            .fetch_all(state.get_pool())
            .await?,
    )
}

pub async fn get_role_with_permissions(
    state: &AppState,
    role_id: i32,
) -> Result<RoleWithPermissions, AppError> {
    let role: Role =
        sqlx::query_as("SELECT id, name, description FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_one(state.get_pool())
            .await?;

    let permissions: Vec<Permission> = sqlx::query_as(
        r#"
        SELECT p.id, p.resource, p.action, p.description
        FROM permissions p
        JOIN role_permissions rp ON p.id = rp.permission_id
        WHERE rp.role_id = $1
        ORDER BY p.resource, p.action
        "#,
    )
    .bind(role_id)
    .fetch_all(state.get_pool())
    .await?;

    Ok(RoleWithPermissions {
        id: role.id,
        name: role.name,
        description: role.description,
        permissions,
    })
}

pub async fn get_role_id_by_name(state: &AppState, name: &str) -> Result<i32, AppError> {
    let (id,): (i32,) = sqlx::query_as("SELECT id FROM roles WHERE name = $1")
        .bind(name)
        .fetch_one(state.get_pool())
        .await?;
    Ok(id)
}

pub async fn get_user_permission_strings_by_email(
    state: &AppState,
    email: &str,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT DISTINCT p.resource, p.action
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE u.email = $1
        "#,
    )
    .bind(email)
    .fetch_all(state.get_pool())
    .await?;

    Ok(rows.into_iter().map(|(r, a)| format!("{}:{}", r, a)).collect())
}

pub async fn get_emails_by_role_id(
    state: &AppState,
    role_id: i32,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT u.email FROM users u JOIN user_roles ur ON u.id = ur.user_id WHERE ur.role_id = $1",
    )
    .bind(role_id)
    .fetch_all(state.get_pool())
    .await?;
    Ok(rows.into_iter().map(|(e,)| e).collect())
}

pub async fn create_role(state: &AppState, new_role: &NewRole) -> Result<Role, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING id, name, description",
    )
    .bind(&new_role.name)
    .bind(&new_role.description)
    .fetch_one(state.get_pool())
    .await?)
}

pub async fn set_role_permissions(
    state: &AppState,
    role_id: i32,
    permission_ids: &[i32],
) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    for &perm_id in permission_ids {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(role_id)
        .bind(perm_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_role(state: &AppState, role_id: i32) -> Result<(), AppError> {
    let built_in = ["guest", "member", "admin", "super_admin"];
    let (name,): (String,) =
        sqlx::query_as("SELECT name FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_one(state.get_pool())
            .await?;

    if built_in.contains(&name.as_str()) {
        return Err(AppError::RequestError(
            crate::errors::RequestError::InvalidContent(
                "無法刪除內建角色".to_string(),
            ),
        ));
    }

    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role_id)
        .execute(state.get_pool())
        .await?;

    Ok(())
}
