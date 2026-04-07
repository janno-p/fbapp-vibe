use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;

pub struct QsForm<T>(pub T);

impl<S, T> FromRequest<S> for QsForm<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        let parsed = serde_qs::from_bytes::<T>(&bytes)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        Ok(QsForm(parsed))
    }
}
