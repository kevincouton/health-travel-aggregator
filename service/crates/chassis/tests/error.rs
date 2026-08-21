use axum::{http::StatusCode, response::IntoResponse};
use chassis::error::ApiError;

#[test]
fn api_error_maps_to_status_code() {
    let cases: Vec<(ApiError, StatusCode)> = vec![
        (ApiError::NotFound, StatusCode::NOT_FOUND),
        (ApiError::Unauthorized, StatusCode::UNAUTHORIZED),
        (ApiError::Forbidden, StatusCode::FORBIDDEN),
        (ApiError::Validation("bad".into()), StatusCode::BAD_REQUEST),
        (ApiError::Conflict, StatusCode::CONFLICT),
        (ApiError::Internal, StatusCode::INTERNAL_SERVER_ERROR),
        (ApiError::BadRequest, StatusCode::BAD_REQUEST),
    ];
    for (err, expected) in cases {
        assert_eq!(err.into_response().status(), expected);
    }
}
