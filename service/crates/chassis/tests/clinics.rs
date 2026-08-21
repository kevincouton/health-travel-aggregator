use chassis::{
    clinics::{self, ClinicStatus},
    error::ApiError,
    users::{self, UserRole},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn provider(pool: &PgPool, email: &str) -> Uuid {
    users::create(pool, email, UserRole::ProviderAdmin, None)
        .await
        .unwrap()
        .id
}

async fn approve(pool: &PgPool, id: Uuid) {
    sqlx::query("UPDATE clinics SET status = 'approved' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test]
async fn create_and_fetch(pool: PgPool) {
    let owner = provider(&pool, "owner@example.com").await;
    let clinic = clinics::create(
        &pool,
        owner,
        "Test Clinic",
        "test-clinic",
        "US",
        "New York",
        &["JCI".into()],
        Some("A great clinic"),
    )
    .await
    .unwrap();

    assert_eq!(clinic.owner_user_id, owner);
    assert_eq!(clinic.name, "Test Clinic");
    assert_eq!(clinic.slug, "test-clinic");
    assert_eq!(clinic.status, ClinicStatus::Draft);

    let by_id = clinics::by_id(&pool, clinic.id).await.unwrap().unwrap();
    assert_eq!(by_id.id, clinic.id);

    let by_slug = clinics::by_slug(&pool, "test-clinic")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_slug.id, clinic.id);
}

#[sqlx::test]
async fn list_for_owner_is_scoped(pool: PgPool) {
    let a = provider(&pool, "a@example.com").await;
    let b = provider(&pool, "b@example.com").await;

    let clinic_a = clinics::create(&pool, a, "Clinic A", "clinic-a", "US", "Austin", &[], None)
        .await
        .unwrap();
    let _clinic_b = clinics::create(&pool, b, "Clinic B", "clinic-b", "US", "Boston", &[], None)
        .await
        .unwrap();

    let list = clinics::list_for_owner(&pool, a).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, clinic_a.id);
}

#[sqlx::test]
async fn list_public_only_approved_and_filtered(pool: PgPool) {
    let owner = provider(&pool, "public@example.com").await;

    let approved = clinics::create(
        &pool,
        owner,
        "Approved Clinic",
        "approved-clinic",
        "US",
        "Miami",
        &[],
        None,
    )
    .await
    .unwrap();
    approve(&pool, approved.id).await;

    let _draft = clinics::create(
        &pool,
        owner,
        "Draft Clinic",
        "draft-clinic",
        "US",
        "Miami",
        &[],
        None,
    )
    .await
    .unwrap();

    let all = clinics::list_public(&pool, None, None).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, approved.id);

    let us = clinics::list_public(&pool, Some("US"), None).await.unwrap();
    assert_eq!(us.len(), 1);

    let miami = clinics::list_public(&pool, Some("US"), Some("Miami"))
        .await
        .unwrap();
    assert_eq!(miami.len(), 1);

    let boston = clinics::list_public(&pool, Some("US"), Some("Boston"))
        .await
        .unwrap();
    assert!(boston.is_empty());
}

#[sqlx::test]
async fn update_own_clinic(pool: PgPool) {
    let owner = provider(&pool, "update@example.com").await;
    let clinic = clinics::create(&pool, owner, "Old", "old-slug", "US", "Denver", &[], None)
        .await
        .unwrap();

    let updated = clinics::update(
        &pool,
        clinic.id,
        owner,
        "New",
        "new-slug",
        "CA",
        "Toronto",
        &["ISO".into()],
        Some("Updated"),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.name, "New");
    assert_eq!(updated.slug, "new-slug");
    assert_eq!(updated.country_code, "CA");
    assert_eq!(updated.accreditations, vec!["ISO"]);
}

#[sqlx::test]
async fn update_by_other_owner_is_not_found(pool: PgPool) {
    let owner = provider(&pool, "owner2@example.com").await;
    let other = provider(&pool, "other@example.com").await;
    let clinic = clinics::create(
        &pool,
        owner,
        "Clinic",
        "secure-clinic",
        "US",
        "Dallas",
        &[],
        None,
    )
    .await
    .unwrap();

    let err = clinics::update(
        &pool,
        clinic.id,
        other,
        "Hacked",
        "hacked",
        "US",
        "Dallas",
        &[],
        None,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, ApiError::NotFound));
}

#[sqlx::test]
async fn duplicate_slug_is_conflict(pool: PgPool) {
    let a = provider(&pool, "dup-a@example.com").await;
    let b = provider(&pool, "dup-b@example.com").await;

    clinics::create(&pool, a, "First", "shared-slug", "US", "Houston", &[], None)
        .await
        .unwrap();

    let err = clinics::create(
        &pool,
        b,
        "Second",
        "shared-slug",
        "US",
        "Houston",
        &[],
        None,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, ApiError::Conflict));
}
