use chassis::{users, users::UserRole};
use sqlx::PgPool;

#[sqlx::test]
async fn create_and_fetch_user(pool: PgPool) {
    let user = users::create(&pool, "alice@example.com", UserRole::Patient, Some("Alice"))
        .await
        .unwrap();
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.role, UserRole::Patient);
    let found = users::by_email(&pool, "alice@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, user.id);
}

#[sqlx::test]
async fn duplicate_email_is_conflict(pool: PgPool) {
    users::create(&pool, "dup@example.com", UserRole::Patient, None)
        .await
        .unwrap();
    let err = users::create(&pool, "dup@example.com", UserRole::Patient, None)
        .await
        .unwrap_err();
    assert!(matches!(err, chassis::error::ApiError::Conflict));
}
