use crate::{
    errors::{AppError, AuthError, RequestError, SystemError},
    repositories::{passkeys, redis, users},
    services::auth as auth_service,
    state::AppState,
    structs::webauthn::PasskeyLoginBeginResponse,
};
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, DiscoverableAuthentication, DiscoverableKey, Passkey,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential, Uuid, WebauthnError,
};
use webauthn_rs_proto::ResidentKeyRequirement;

// 挑戰壽命：Conditional UI 掛在登入頁超過此時間才點 autofill 會失敗，前端 401 靜默重 begin 一次
const CHALLENGE_TTL_SECS: u64 = 300;

fn reg_key(user_id: i64) -> String {
    format!("webauthn:reg:{}", user_id)
}

fn auth_key(auth_id: &str) -> String {
    format!("webauthn:auth:{}", auth_id)
}

// 失敗細節只進 log，對外統一 401 不外洩
fn webauthn_err(e: WebauthnError) -> AppError {
    tracing::warn!("webauthn ceremony failed: {:?}", e);
    AppError::AuthError(AuthError::WebauthnFailed)
}

// instance 由 app_settings（platform 保留 key）建構、熱重載；未設定/無效時 passkey 功能不可用
fn get_webauthn(state: &AppState) -> Result<webauthn_rs::Webauthn, AppError> {
    state.get_settings().webauthn().ok_or_else(|| {
        AppError::SystemError(SystemError::Internal(
            "WebAuthn 未設定（/admin/platform 的 webauthn_rp_id / webauthn_rp_origin）".to_string(),
        ))
    })
}

// CredentialID 序列化即 base64url 字串，取出作 DB 查詢 key
fn cred_id_string(cred_id: &CredentialID) -> Result<String, AppError> {
    match serde_json::to_value(cred_id)? {
        serde_json::Value::String(s) => Ok(s),
        _ => Err(AppError::SystemError(SystemError::Internal(
            "credential id 序列化非字串".to_string(),
        ))),
    }
}

async fn load_user_passkeys(state: &AppState, user_id: i64) -> Result<Vec<Passkey>, AppError> {
    Ok(passkeys::passkeys_by_user_id(state.get_pool(), user_id)
        .await?
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect())
}

pub async fn begin_registration(
    state: &AppState,
    user_id: i64,
) -> Result<CreationChallengeResponse, AppError> {
    let (handle, name) = users::get_webauthn_identity_by_id(state.get_pool(), user_id)
        .await?
        .ok_or(AppError::AuthError(AuthError::UserNotFound))?;

    let exclude: Vec<CredentialID> = load_user_passkeys(state, user_id)
        .await?
        .iter()
        .map(|p| p.cred_id().clone())
        .collect();
    let exclude = (!exclude.is_empty()).then_some(exclude);

    let (mut ccr, reg_state) = get_webauthn(state)?
        .start_passkey_registration(handle, &name, &name, exclude)
        .map_err(webauthn_err)?;

    // start_passkey_registration 預設不要求 resident key，但 Conditional UI（discoverable login）
    // 只認 resident credential——不改這裡的話註冊照樣成功、autofill 卻永遠不會跳出
    if let Some(selection) = ccr.public_key.authenticator_selection.as_mut() {
        selection.resident_key = Some(ResidentKeyRequirement::Required);
        selection.require_resident_key = true;
    }

    redis::cache_set(
        state.get_redis_pool(),
        &reg_key(user_id),
        &serde_json::to_string(&reg_state)?,
        CHALLENGE_TTL_SECS,
    )
    .await?;

    Ok(ccr)
}

pub async fn finish_registration(
    state: &AppState,
    user_id: i64,
    label: &str,
    credential: &RegisterPublicKeyCredential,
) -> Result<(), AppError> {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 64 {
        return Err(AppError::RequestError(RequestError::UnprocessableContent(
            "label 長度須為 1–64 字".to_string(),
        )));
    }

    // GETDEL 一次性消費，挑戰不存在 = 過期或重放
    let state_json = redis::cache_getdel(state.get_redis_pool(), &reg_key(user_id))
        .await?
        .ok_or(AppError::AuthError(AuthError::WebauthnFailed))?;
    let reg_state: PasskeyRegistration = serde_json::from_str(&state_json)?;

    let passkey = get_webauthn(state)?
        .finish_passkey_registration(credential, &reg_state)
        .map_err(webauthn_err)?;

    let cred_id = cred_id_string(passkey.cred_id())?;
    let inserted = passkeys::insert(
        state.get_pool(),
        user_id,
        &cred_id,
        &serde_json::to_value(&passkey)?,
        label,
    )
    .await?;
    if !inserted {
        return Err(AppError::RequestError(RequestError::Conflict(
            "此 passkey 已註冊過".to_string(),
        )));
    }
    Ok(())
}

pub async fn begin_login(state: &AppState) -> Result<PasskeyLoginBeginResponse, AppError> {
    let (rcr, auth_state) = get_webauthn(state)?
        .start_discoverable_authentication()
        .map_err(webauthn_err)?;

    let auth_id = Uuid::new_v4().to_string();
    redis::cache_set(
        state.get_redis_pool(),
        &auth_key(&auth_id),
        &serde_json::to_string(&auth_state)?,
        CHALLENGE_TTL_SECS,
    )
    .await?;

    Ok(PasskeyLoginBeginResponse { auth_id, options: rcr })
}

pub async fn finish_login(
    state: &AppState,
    auth_id: &str,
    credential: &PublicKeyCredential,
) -> Result<String, AppError> {
    let webauthn = get_webauthn(state)?;
    let state_json = redis::cache_getdel(state.get_redis_pool(), &auth_key(auth_id))
        .await?
        .ok_or(AppError::AuthError(AuthError::WebauthnFailed))?;
    let auth_state: DiscoverableAuthentication = serde_json::from_str(&state_json)?;

    // discoverable credential 的 userHandle 反查 user，免輸帳號
    let (handle, _cred_id) = webauthn
        .identify_discoverable_authentication(credential)
        .map_err(webauthn_err)?;
    let user_id = users::get_id_by_webauthn_handle(state.get_pool(), handle)
        .await?
        .ok_or(AppError::AuthError(AuthError::WebauthnFailed))?;

    let user_passkeys = load_user_passkeys(state, user_id).await?;
    let discoverable: Vec<DiscoverableKey> =
        user_passkeys.iter().map(DiscoverableKey::from).collect();

    let auth_result = webauthn
        .finish_discoverable_authentication(credential, auth_state, &discoverable)
        .map_err(webauthn_err)?;

    // counter/backup 旗標有變才回寫整包，否則只更 last_used_at
    if let Some(mut passkey) = user_passkeys
        .into_iter()
        .find(|p| p.cred_id() == auth_result.cred_id())
    {
        let updated = passkey.update_credential(&auth_result) == Some(true);
        let cred_id = cred_id_string(auth_result.cred_id())?;
        let updated_json = if updated { Some(serde_json::to_value(&passkey)?) } else { None };
        passkeys::touch_after_auth(state.get_pool(), &cred_id, updated_json.as_ref()).await?;
    }

    auth_service::complete_admin_login(
        state.get_pool(),
        state.get_redis_pool(),
        &state.get_config().jwt_secret,
        user_id,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // resident key mutate 是 Conditional UI 能否運作的關鍵——見 begin_registration 內註解
    #[test]
    fn start_registration_challenge_requires_resident_key() {
        let rp_origin = webauthn_rs::prelude::Url::parse("http://localhost:3000").unwrap();
        let webauthn = webauthn_rs::WebauthnBuilder::new("localhost", &rp_origin)
            .unwrap()
            .build()
            .unwrap();

        let (mut ccr, _state) = webauthn
            .start_passkey_registration(Uuid::new_v4(), "tester", "tester", None)
            .unwrap();

        if let Some(selection) = ccr.public_key.authenticator_selection.as_mut() {
            selection.resident_key = Some(ResidentKeyRequirement::Required);
            selection.require_resident_key = true;
        }

        let selection = ccr
            .public_key
            .authenticator_selection
            .expect("challenge 應帶 authenticator_selection");
        assert_eq!(selection.resident_key, Some(ResidentKeyRequirement::Required));
        assert!(selection.require_resident_key);
    }
}
