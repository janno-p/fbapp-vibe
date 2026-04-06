use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unexpected(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };
        match &self {
            AppError::Unexpected(_) => tracing::error!(error = %self, "unexpected server error"),
            _ => tracing::warn!(error = %self, "request error"),
        }
        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        String::from_utf8(bytes.to_vec()).expect("body is not utf-8")
    }

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let response = AppError::Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(body_string(response).await, "unauthorized");
    }

    #[tokio::test]
    async fn bad_request_returns_400_with_message() {
        let response = AppError::BadRequest("email is required".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_string(response).await, "email is required");
    }

    #[tokio::test]
    async fn unexpected_returns_500_with_generic_message() {
        let response =
            AppError::Unexpected(anyhow::anyhow!("db connection pool exhausted")).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        // Internal detail must not leak to the response body
        assert_eq!(body_string(response).await, "internal server error");
    }
}
