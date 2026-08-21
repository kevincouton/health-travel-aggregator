use chassis::auth;
use sqlx::PgPool;

#[test]
fn password_hash_roundtrip() {
    let hash = auth::hash_password("secret123").unwrap();
    assert!(auth::verify_password("secret123", &hash).is_ok());
    assert!(auth::verify_password("wrong", &hash).is_err());
}

#[sqlx::test]
async fn session_lifecycle(pool: PgPool) {
    let user = chassis::users::create(
        &pool,
        "sess@example.com",
        chassis::users::UserRole::Patient,
        None,
    )
    .await
    .unwrap();
    let token = auth::create_session(&pool, user.id, 7).await.unwrap();
    let resolved = auth::resolve_session(&pool, &token).await.unwrap();
    assert_eq!(resolved.unwrap().id, user.id);
    auth::delete_session(&pool, &token).await.unwrap();
    assert!(auth::resolve_session(&pool, &token)
        .await
        .unwrap()
        .is_none());
}
