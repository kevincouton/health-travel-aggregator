use chassis::{
    clinics::{self},
    error::ApiError,
    packages, treatments,
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

async fn clinic(pool: &PgPool, owner: Uuid, slug: &str) -> clinics::Clinic {
    clinics::create(pool, owner, slug, slug, "US", "Austin", &[], None)
        .await
        .unwrap()
}

async fn treatment(pool: &PgPool, slug: &str) -> treatments::Treatment {
    chassis::db::migrate(pool).await.unwrap();
    treatments::seed(pool).await.unwrap();
    treatments::by_slug(pool, slug).await.unwrap().unwrap()
}

#[sqlx::test]
async fn create_and_fetch(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "owner@example.com").await;
    let clinic = clinic(&pool, owner, "create-fetch").await;
    let treatment = treatment(&pool, "dental-implants").await;

    let package = packages::create(
        &pool,
        clinic.id,
        treatment.id,
        "Dental Implant Package",
        Some(1_000),
        Some(2_000),
        Some(7),
        &["consultation".into()],
        &["flights".into()],
    )
    .await
    .unwrap();

    assert_eq!(package.clinic_id, clinic.id);
    assert_eq!(package.treatment_id, treatment.id);
    assert_eq!(package.name, "Dental Implant Package");
    assert_eq!(package.price_min, Some(1_000));
    assert!(!package.is_published);

    let fetched = packages::by_id(&pool, package.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, package.id);
}

#[sqlx::test]
async fn list_for_clinic_and_public_filter(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "list@example.com").await;
    let clinic = clinic(&pool, owner, "list-clinic").await;
    let treatment = treatment(&pool, "hair-transplant").await;

    let draft = packages::create(
        &pool,
        clinic.id,
        treatment.id,
        "Draft Package",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap();

    let published = packages::create(
        &pool,
        clinic.id,
        treatment.id,
        "Published Package",
        Some(500),
        Some(1_000),
        Some(3),
        &[],
        &[],
    )
    .await
    .unwrap();

    let updated = packages::update(
        &pool,
        published.id,
        "Published Package",
        Some(500),
        Some(1_000),
        Some(3),
        &[],
        &[],
        true,
    )
    .await
    .unwrap()
    .unwrap();
    assert!(updated.is_published);

    let all = packages::list_for_clinic(&pool, clinic.id).await.unwrap();
    assert_eq!(all.len(), 2);

    let public = packages::list_public_for_clinic(&pool, clinic.id)
        .await
        .unwrap();
    assert_eq!(public.len(), 1);
    assert_eq!(public[0].id, published.id);

    let draft_in_list = packages::by_id(&pool, draft.id).await.unwrap().unwrap();
    assert!(!draft_in_list.is_published);
}

#[sqlx::test]
async fn list_public_for_treatment(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "treatment-list@example.com").await;
    let clinic = clinic(&pool, owner, "treatment-clinic").await;
    treatments::seed(&pool).await.unwrap();

    let dental = treatments::by_slug(&pool, "dental-implants")
        .await
        .unwrap()
        .unwrap();
    let hair = treatments::by_slug(&pool, "hair-transplant")
        .await
        .unwrap()
        .unwrap();

    let dental_pkg = packages::create(
        &pool,
        clinic.id,
        dental.id,
        "Dental",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap();
    packages::update(
        &pool,
        dental_pkg.id,
        "Dental",
        None,
        None,
        None,
        &[],
        &[],
        true,
    )
    .await
    .unwrap();

    let _hair_pkg = packages::create(
        &pool,
        clinic.id,
        hair.id,
        "Hair",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap();

    let list = packages::list_public_for_treatment(&pool, dental.id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, dental_pkg.id);
}

#[sqlx::test]
async fn update_package(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "update@example.com").await;
    let clinic = clinic(&pool, owner, "update-clinic").await;
    let treatment = treatment(&pool, "ivf").await;

    let package = packages::create(
        &pool,
        clinic.id,
        treatment.id,
        "Old",
        Some(100),
        Some(200),
        Some(1),
        &["a".into()],
        &["b".into()],
    )
    .await
    .unwrap();

    let updated = packages::update(
        &pool,
        package.id,
        "New",
        Some(300),
        Some(400),
        Some(2),
        &["c".into()],
        &["d".into()],
        true,
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.name, "New");
    assert_eq!(updated.price_min, Some(300));
    assert_eq!(updated.price_max, Some(400));
    assert_eq!(updated.duration_days, Some(2));
    assert_eq!(updated.inclusions, vec!["c"]);
    assert_eq!(updated.exclusions, vec!["d"]);
    assert!(updated.is_published);
}

#[sqlx::test]
async fn delete_package(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "delete@example.com").await;
    let clinic = clinic(&pool, owner, "delete-clinic").await;
    let treatment = treatment(&pool, "cosmetic-surgery").await;

    let package = packages::create(
        &pool,
        clinic.id,
        treatment.id,
        "To Delete",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap();

    assert!(packages::delete(&pool, package.id).await.unwrap());
    assert!(packages::by_id(&pool, package.id).await.unwrap().is_none());
    assert!(!packages::delete(&pool, package.id).await.unwrap());
}

#[sqlx::test]
async fn invalid_clinic_or_treatment_is_conflict(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "conflict@example.com").await;
    let clinic = clinic(&pool, owner, "conflict-clinic").await;
    let missing_clinic = Uuid::new_v4();
    let missing_treatment = Uuid::new_v4();

    let err = packages::create(
        &pool,
        missing_clinic,
        missing_treatment,
        "Bad",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ApiError::Conflict));

    let err = packages::create(
        &pool,
        clinic.id,
        missing_treatment,
        "Bad",
        None,
        None,
        None,
        &[],
        &[],
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ApiError::Conflict));
}
