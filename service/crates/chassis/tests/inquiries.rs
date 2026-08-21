use chassis::{
    clinics::{self},
    error::ApiError,
    inquiries::{self, InquiryStatus},
    packages, treatments,
    users::{self, UserRole},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn patient_id(pool: &PgPool, email: &str) -> Uuid {
    users::create(pool, email, UserRole::Patient, None)
        .await
        .unwrap()
        .id
}

async fn provider_id(pool: &PgPool, email: &str) -> Uuid {
    users::create(pool, email, UserRole::ProviderAdmin, None)
        .await
        .unwrap()
        .id
}

async fn make_clinic(pool: &PgPool, owner: Uuid, slug: &str) -> clinics::Clinic {
    clinics::create(pool, owner, slug, slug, "US", "Austin", &[], None)
        .await
        .unwrap()
}

async fn make_treatment(pool: &PgPool, slug: &str) -> treatments::Treatment {
    treatments::seed(pool).await.unwrap();
    treatments::by_slug(pool, slug).await.unwrap().unwrap()
}

async fn make_package(pool: &PgPool, clinic_id: Uuid, treatment_id: Uuid) -> packages::Package {
    packages::create(
        pool,
        clinic_id,
        treatment_id,
        "Test Package",
        Some(1_000),
        Some(2_000),
        Some(7),
        &[],
        &[],
    )
    .await
    .unwrap()
}

#[sqlx::test]
async fn create_and_fetch(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "create-clinic").await;
    let treatment = make_treatment(&pool, "dental-implants").await;
    let package = make_package(&pool, clinic.id, treatment.id).await;

    let inquiry = inquiries::create(
        &pool,
        patient,
        clinic.id,
        Some(package.id),
        Some("Notes"),
        Some("March 2026"),
        "patient@example.com",
    )
    .await
    .unwrap();

    assert_eq!(inquiry.patient_user_id, patient);
    assert_eq!(inquiry.clinic_id, clinic.id);
    assert_eq!(inquiry.package_id, Some(package.id));
    assert_eq!(inquiry.status, InquiryStatus::New);
    assert_eq!(inquiry.contact_email, "patient@example.com");

    let fetched = inquiries::by_id(&pool, inquiry.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, inquiry.id);
}

#[sqlx::test]
async fn list_for_patient(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let other_patient = patient_id(&pool, "other-patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "list-clinic").await;

    let inquiry = inquiries::create(
        &pool,
        patient,
        clinic.id,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap();

    let _other = inquiries::create(
        &pool,
        other_patient,
        clinic.id,
        None,
        None,
        None,
        "other@example.com",
    )
    .await
    .unwrap();

    let list = inquiries::list_for_patient(&pool, patient).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, inquiry.id);
}

#[sqlx::test]
async fn list_for_provider(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner_a = provider_id(&pool, "owner-a@example.com").await;
    let owner_b = provider_id(&pool, "owner-b@example.com").await;
    let clinic_a = make_clinic(&pool, owner_a, "clinic-a").await;
    let clinic_b = make_clinic(&pool, owner_b, "clinic-b").await;

    let inquiry_a = inquiries::create(
        &pool,
        patient,
        clinic_a.id,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap();

    let _inquiry_b = inquiries::create(
        &pool,
        patient,
        clinic_b.id,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap();

    let list = inquiries::list_for_provider(&pool, owner_a).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, inquiry_a.id);
}

#[sqlx::test]
async fn update_status(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "update-clinic").await;

    let inquiry = inquiries::create(
        &pool,
        patient,
        clinic.id,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap();

    let updated = inquiries::update_status(&pool, inquiry.id, owner, InquiryStatus::Contacted)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, InquiryStatus::Contacted);

    let other = provider_id(&pool, "other@example.com").await;
    let none = inquiries::update_status(&pool, inquiry.id, other, InquiryStatus::Closed)
        .await
        .unwrap();
    assert!(none.is_none());
}

#[sqlx::test]
async fn missing_clinic_is_not_found(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let missing_clinic = Uuid::new_v4();

    let err = inquiries::create(
        &pool,
        patient,
        missing_clinic,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ApiError::NotFound));
}

#[sqlx::test]
async fn package_must_belong_to_clinic(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner_a = provider_id(&pool, "owner-a@example.com").await;
    let owner_b = provider_id(&pool, "owner-b@example.com").await;
    let clinic_a = make_clinic(&pool, owner_a, "clinic-a").await;
    let clinic_b = make_clinic(&pool, owner_b, "clinic-b").await;
    let treatment = make_treatment(&pool, "hair-transplant").await;
    let package_b = make_package(&pool, clinic_b.id, treatment.id).await;

    let err = inquiries::create(
        &pool,
        patient,
        clinic_a.id,
        Some(package_b.id),
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ApiError::BadRequest));
}

#[sqlx::test]
async fn empty_email_is_validation_error(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "email-clinic").await;

    let err = inquiries::create(&pool, patient, clinic.id, None, None, None, "  ")
        .await
        .unwrap_err();
    assert!(matches!(err, ApiError::Validation(_)));
}
