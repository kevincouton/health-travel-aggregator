use chassis::{
    clinics::{self},
    error::ApiError,
    inquiries::{self, InquiryStatus},
    reviews,
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

async fn make_converted_inquiry(
    pool: &PgPool,
    patient: Uuid,
    clinic_id: Uuid,
) -> inquiries::Inquiry {
    let inquiry = inquiries::create(
        pool,
        patient,
        clinic_id,
        None,
        None,
        None,
        "patient@example.com",
    )
    .await
    .unwrap();

    let owner = clinics::by_id(pool, clinic_id)
        .await
        .unwrap()
        .unwrap()
        .owner_user_id;
    inquiries::update_status(pool, inquiry.id, owner, InquiryStatus::Converted)
        .await
        .unwrap()
        .unwrap()
}

#[sqlx::test]
async fn can_review_returns_converted_inquiry(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "review-clinic").await;
    let inquiry = make_converted_inquiry(&pool, patient, clinic.id).await;

    let eligible = reviews::can_review(&pool, patient, clinic.id)
        .await
        .unwrap();
    assert_eq!(eligible, Some(inquiry.id));
}

#[sqlx::test]
async fn can_review_none_for_new_inquiry(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "new-clinic").await;

    inquiries::create(
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

    let eligible = reviews::can_review(&pool, patient, clinic.id)
        .await
        .unwrap();
    assert_eq!(eligible, None);
}

#[sqlx::test]
async fn create_review_and_list(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "list-clinic").await;
    let inquiry = make_converted_inquiry(&pool, patient, clinic.id).await;

    let review = reviews::create(
        &pool,
        clinic.id,
        patient,
        inquiry.id,
        5,
        Some("Great clinic"),
    )
    .await
    .unwrap();
    assert_eq!(review.clinic_id, clinic.id);
    assert_eq!(review.patient_user_id, patient);
    assert_eq!(review.inquiry_id, inquiry.id);
    assert_eq!(review.rating, 5);
    assert_eq!(review.comment, Some("Great clinic".into()));

    let list = reviews::list_for_clinic(&pool, clinic.id).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, review.id);
}

#[sqlx::test]
async fn create_review_validates_rating(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "rating-clinic").await;
    let inquiry = make_converted_inquiry(&pool, patient, clinic.id).await;

    let err = reviews::create(&pool, clinic.id, patient, inquiry.id, 0, None)
        .await
        .unwrap_err();
    assert!(matches!(err, ApiError::Validation(_)));

    let err = reviews::create(&pool, clinic.id, patient, inquiry.id, 6, None)
        .await
        .unwrap_err();
    assert!(matches!(err, ApiError::Validation(_)));
}

#[sqlx::test]
async fn average_rating(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient_a = patient_id(&pool, "patient-a@example.com").await;
    let patient_b = patient_id(&pool, "patient-b@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "avg-clinic").await;

    let inquiry_a = make_converted_inquiry(&pool, patient_a, clinic.id).await;
    let inquiry_b = make_converted_inquiry(&pool, patient_b, clinic.id).await;

    reviews::create(&pool, clinic.id, patient_a, inquiry_a.id, 4, None)
        .await
        .unwrap();
    reviews::create(&pool, clinic.id, patient_b, inquiry_b.id, 2, None)
        .await
        .unwrap();

    let avg = reviews::average_rating(&pool, clinic.id).await.unwrap();
    assert!((avg.unwrap() - 3.0).abs() < f64::EPSILON);
}

#[sqlx::test]
async fn duplicate_review_for_same_inquiry_is_conflict(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let patient = patient_id(&pool, "patient@example.com").await;
    let owner = provider_id(&pool, "owner@example.com").await;
    let clinic = make_clinic(&pool, owner, "dup-clinic").await;
    let inquiry = make_converted_inquiry(&pool, patient, clinic.id).await;

    reviews::create(&pool, clinic.id, patient, inquiry.id, 5, None)
        .await
        .unwrap();

    let err = reviews::create(&pool, clinic.id, patient, inquiry.id, 4, None)
        .await
        .unwrap_err();
    assert!(matches!(err, ApiError::Conflict));
}
