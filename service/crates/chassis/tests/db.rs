use sqlx::PgPool;

#[sqlx::test]
async fn test_users_table_exists(pool: PgPool) {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);
}
