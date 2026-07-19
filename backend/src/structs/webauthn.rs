use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse};

#[derive(Serialize, sqlx::FromRow)]
pub struct PasskeyListItem {
    pub id: i64,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct PasskeyRegisterFinishData {
    pub label: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Serialize)]
pub struct PasskeyLoginBeginResponse {
    pub auth_id: String,
    pub options: RequestChallengeResponse,
}

#[derive(Deserialize)]
pub struct PasskeyLoginFinishData {
    pub auth_id: String,
    pub credential: PublicKeyCredential,
}
