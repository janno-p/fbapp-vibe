use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Template)]
#[template(path = "errors/401.html")]
struct UnauthorizedTemplate;

#[derive(Template)]
#[template(path = "errors/403.html")]
struct ForbiddenTemplate;

#[derive(Template)]
#[template(path = "errors/404.html")]
struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "errors/500.html")]
struct ServerErrorTemplate;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::Unexpected(_) => tracing::error!(error = %self, "unexpected server error"),
            _ => tracing::warn!(error = %self, "request error"),
        }
        match self {
            AppError::Unauthorized => render(StatusCode::UNAUTHORIZED, UnauthorizedTemplate),
            AppError::Forbidden => render(StatusCode::FORBIDDEN, ForbiddenTemplate),
            AppError::NotFound => render(StatusCode::NOT_FOUND, NotFoundTemplate),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Unexpected(_) => {
                render(StatusCode::INTERNAL_SERVER_ERROR, ServerErrorTemplate)
            }
        }
    }
}

fn render(status: StatusCode, template: impl Template) -> Response {
    match template.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(_) => (status, "An error occurred").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let response = AppError::Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forbidden_returns_403() {
        let response = AppError::Forbidden.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let response = AppError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_request_returns_400_with_message() {
        use axum::body::to_bytes;
        let response = AppError::BadRequest("email is required".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let body = String::from_utf8(bytes.to_vec()).expect("body is not utf-8");
        assert_eq!(body, "email is required");
    }

    #[tokio::test]
    async fn unexpected_returns_500() {
        let response =
            AppError::Unexpected(anyhow::anyhow!("db connection pool exhausted")).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
