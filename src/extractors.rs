use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;

/// Maximum allowed body size for form submissions (16 KiB).
const MAX_FORM_BYTES: usize = 16 * 1024;

pub struct QsForm<T>(pub T);

impl<S, T> FromRequest<S> for QsForm<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::to_bytes(req.into_body(), MAX_FORM_BYTES)
            .await
            .map_err(|_| {
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "request body too large".to_string(),
                )
            })?;

        let parsed = serde_qs::from_bytes::<T>(&bytes)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        Ok(QsForm(parsed))
    }
}
