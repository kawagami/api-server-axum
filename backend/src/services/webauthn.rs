use crate::{
    errors::{AppError, AuthError, RequestError, SystemError},
    repositories::{passkeys, redis, users},
    services::auth as auth_service,
    state::AppState,
    structs::webauthn::{PasskeyListItem, PasskeyLoginBeginResponse},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, DiscoverableAuthentication, Passkey,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential, Uuid, WebauthnError,
};
use webauthn_rs_proto::ResidentKeyRequirement;

/// 管理頁列表：只列自己的 passkey（不回公鑰 / credential_id）。
pub async fn list_own(state: &AppState, user_id: i64) -> Result<Vec<PasskeyListItem>, AppError> {
    passkeys::list_by_user_id(state.get_pool(), user_id).await
}

/// 刪除自己的一把 passkey。**「只能刪自己的」由 SQL 的 `AND user_id = $2` 保證**，
/// 刪不到（不存在或不是本人的）一律 404 —— 不區分兩者，避免用回應碼探測他人 passkey id。
pub async fn delete_own(state: &AppState, user_id: i64, id: i64) -> Result<(), AppError> {
    if passkeys::delete_own(state.get_pool(), user_id, id).await? {
        Ok(())
    } else {
        Err(AppError::RequestError(RequestError::NotFound))
    }
}

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

/// DB `credential_id` 欄 = base64url（無 padding）字串，作查詢 key。
///
/// ⚠️ 0.5 時 `CredentialID` 是 `HumanBinaryData`，直接 serde_json 序列化就得到這個字串；
/// 0.6 改成裸 `Vec<u8>`（base64 只掛在 struct 欄位的 serde_as 上），再走 serde_json 會變成數字陣列。
/// 所以明確自己編碼，格式與 0.5 寫進 DB 的存量一致（見測試 `cred_id_string_matches_stored_format`）。
fn cred_id_string(cred_id: &CredentialID) -> String {
    URL_SAFE_NO_PAD.encode(cred_id)
}

async fn load_user_passkeys(state: &AppState, user_id: i64) -> Result<Vec<Passkey>, AppError> {
    Ok(passkeys::passkeys_by_user_id(state.get_pool(), user_id)
        .await?
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect())
}

/// start_passkey_registration 預設不要求 resident key，但 Conditional UI（discoverable login）
/// 只認 resident credential——少了這步註冊照樣成功、autofill 卻永遠不會跳出。
///
/// 抽成獨立函式是為了讓測試驗得到**這一份**邏輯：測試若自己複製一次 mutate，
/// 刪掉 production 這段也照樣綠。
fn require_resident_key(ccr: &mut CreationChallengeResponse) {
    if let Some(selection) = ccr.public_key.authenticator_selection.as_mut() {
        selection.resident_key = Some(ResidentKeyRequirement::Required);
        selection.require_resident_key = true;
    }
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

    require_resident_key(&mut ccr);

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

    let cred_id = cred_id_string(passkey.cred_id());
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
    let (handle, cred_id) = webauthn
        .identify_discoverable_authentication(credential)
        .map_err(webauthn_err)?;
    let user_id = users::get_id_by_webauthn_handle(state.get_pool(), handle)
        .await?
        .ok_or(AppError::AuthError(AuthError::WebauthnFailed))?;

    // userHandle 對到的 user 名下找不到這把 credential = 偽造 / 已刪除，直接拒絕
    let mut passkey = load_user_passkeys(state, user_id)
        .await?
        .into_iter()
        .find(|p| p.cred_id().as_slice() == cred_id)
        .ok_or(AppError::AuthError(AuthError::WebauthnFailed))?;

    let auth_result = webauthn
        .finish_discoverable_authentication_v2(credential, auth_state, &passkey)
        .map_err(webauthn_err)?;

    // counter/backup 旗標有變才回寫整包，否則只更 last_used_at
    let updated = passkey.update_credential(&auth_result) == Some(true);
    let cred_id = cred_id_string(auth_result.cred_id());
    let updated_json = if updated { Some(serde_json::to_value(&passkey)?) } else { None };
    passkeys::touch_after_auth(state.get_pool(), &cred_id, updated_json.as_ref()).await?;

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

    /// resident key 是 Conditional UI 能否運作的關鍵——見 `require_resident_key` 的註解。
    /// 這條驗的是 production 那個函式本身：先確認 webauthn_rs 的預設**不是** Required
    /// （否則這步就沒必要存在，測試也失去意義），再確認函式把它改過來。
    #[test]
    fn require_resident_key_upgrades_the_default_challenge() {
        let rp_origin = webauthn_rs::prelude::Url::parse("http://localhost:3000").unwrap();
        let webauthn = webauthn_rs::WebauthnBuilder::new("localhost", &rp_origin)
            .unwrap()
            .build()
            .unwrap();

        let (mut ccr, _state) = webauthn
            .start_passkey_registration(Uuid::new_v4(), "tester", "tester", None)
            .unwrap();

        let before = ccr
            .public_key
            .authenticator_selection
            .clone()
            .expect("challenge 應帶 authenticator_selection");
        assert_ne!(
            before.resident_key,
            Some(ResidentKeyRequirement::Required),
            "上游預設已是 Required，require_resident_key 可以拿掉了"
        );

        require_resident_key(&mut ccr);

        let after = ccr
            .public_key
            .authenticator_selection
            .expect("challenge 應帶 authenticator_selection");
        assert_eq!(after.resident_key, Some(ResidentKeyRequirement::Required));
        assert!(after.require_resident_key);
    }

    /// 0.5 寫進 `user_passkeys.passkey` 的 jsonb 必須能被現版反序列化 ——
    /// `load_user_passkeys` 用 `filter_map(.ok())`，格式不相容時舊 passkey 會**靜默消失**（登入只會說找不到）。
    /// 樣本是 webauthn-rs-core 0.5.5 自己的序列化測試資料（attestation 簡化成 passkey 常見的 None），外包 `Passkey { cred }`。
    const PASSKEY_V05_JSON: &str = r#"{"cred":{"cred_id":"uZcVDBVS68E_MtAgeQpElJxldF_6cY9sSvbWqx_qRh8wiu42lyRBRmh5yFeD_r9k130dMbFHBHI9RTFgdJQIzQ","cred":{"type_":"ES256","key":{"EC_EC2":{"curve":"SECP256R1","x":[194,126,127,109,252,23,131,21,252,6,223,99,44,254,140,27,230,17,94,5,133,28,104,41,144,69,171,149,161,26,200,243],"y":[143,123,183,156,24,178,21,248,117,159,162,69,171,52,188,252,26,59,6,47,103,92,19,58,117,103,249,0,219,8,95,196]}}},"counter":2,"transports":null,"user_verified":true,"backup_eligible":true,"backup_state":true,"registration_policy":"required","extensions":{"cred_protect":"NotRequested","hmac_create_secret":"NotRequested","appid":"NotRequested","cred_props":"Ignored"},"attestation":{"data":"None","metadata":"None"},"attestation_format":"None"}}"#;

    #[test]
    fn passkey_serialized_by_v05_still_deserializes() {
        let passkey: Passkey =
            serde_json::from_str(PASSKEY_V05_JSON).expect("0.5 格式的 passkey 應能反序列化");
        // 再序列化回去也要是同一種形狀（cred_id 仍是字串），否則 update_credential 回寫後下次讀會壞
        let reser = serde_json::to_value(&passkey).unwrap();
        assert!(reser["cred"]["cred_id"].is_string());
        let _: Passkey = serde_json::from_value(reser).unwrap();
    }

    /// `credential_id` 欄是 0.5 時代寫入的 base64url 字串，現版算出的 key 必須逐字相同，否則登入對不到列。
    #[test]
    fn cred_id_string_matches_stored_format() {
        let passkey: Passkey = serde_json::from_str(PASSKEY_V05_JSON).unwrap();
        assert_eq!(
            cred_id_string(passkey.cred_id()),
            "uZcVDBVS68E_MtAgeQpElJxldF_6cY9sSvbWqx_qRh8wiu42lyRBRmh5yFeD_r9k130dMbFHBHI9RTFgdJQIzQ"
        );
    }
}
