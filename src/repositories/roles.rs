use crate::{
    errors::AppError,
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions},
};
use sqlx::{Pool, Postgres};

pub async fn get_roles(pool: &Pool<Postgres>) -> Result<Vec<Role>, AppError> {
    Ok(
        sqlx::query_as("SELECT id, name, description FROM roles ORDER BY id")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_role_with_permissions(
    pool: &Pool<Postgres>,
    role_id: i32,
) -> Result<RoleWithPermissions, AppError> {
    let role: Role =
        sqlx::query_as("SELECT id, name, description FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_one(pool)
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
    .fetch_all(pool)
    .await?;

    Ok(RoleWithPermissions {
        id: role.id,
        name: role.name,
        description: role.description,
        permissions,
    })
}

pub async fn get_role_id_by_name(pool: &Pool<Postgres>, name: &str) -> Result<i32, AppError> {
    let (id,): (i32,) = sqlx::query_as("SELECT id FROM roles WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(id)
}

pub async fn get_user_permission_strings_by_email(
    pool: &Pool<Postgres>,
    email: &str,
) -> Result<Vec<String>, AppError> {
    let is_super_admin: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE u.email = $1 AND r.name = 'super_admin'
        )
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await?;

    if is_super_admin {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT resource, action FROM permissions")
                .fetch_all(pool)
                .await?;
        return Ok(rows.into_iter().map(|(r, a)| format!("{}:{}", r, a)).collect());
    }

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
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(r, a)| format!("{}:{}", r, a)).collect())
}

pub async fn get_emails_by_role_id(
    pool: &Pool<Postgres>,
    role_id: i32,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT u.email FROM users u JOIN user_roles ur ON u.id = ur.user_id WHERE ur.role_id = $1",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(e,)| e).collect())
}

pub async fn create_role(pool: &Pool<Postgres>, new_role: &NewRole) -> Result<Role, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING id, name, description",
    )
    .bind(&new_role.name)
    .bind(&new_role.description)
    .fetch_one(pool)
    .await?)
}

pub async fn set_role_permissions(
    pool: &Pool<Postgres>,
    role_id: i32,
    permission_ids: &[i32],
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

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

pub async fn delete_role(pool: &Pool<Postgres>, role_id: i32) -> Result<(), AppError> {
    let built_in = ["guest", "member", "admin", "super_admin"];
    let (name,): (String,) =
        sqlx::query_as("SELECT name FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_one(pool)
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
        .execute(pool)
        .await?;

    Ok(())
}
