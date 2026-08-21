use chassis::{
    clinics::{self, ClinicFilters, ClinicStatus},
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

async fn make_clinic(
    pool: &PgPool,
    owner: Uuid,
    slug: &str,
    name: &str,
    country: &str,
    city: &str,
    accreditations: &[String],
) -> clinics::Clinic {
    clinics::create(pool, owner, name, slug, country, city, accreditations, Some("description"))
        .await
        .unwrap()
}

async fn approve(pool: &PgPool, id: Uuid) {
    sqlx::query("UPDATE clinics SET status = 'approved' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

fn filters() -> ClinicFilters {
    ClinicFilters::default()
}

#[sqlx::test]
async fn text_search_matches_name_or_city(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "search@example.com").await;

    let a = make_clinic(&pool, owner, "alpha-clinic", "Alpha Dental", "US", "New York", &[]).await;
    let b = make_clinic(&pool, owner, "beta-clinic", "Beta Care", "US", "Boston", &[]).await;
    approve(&pool, a.id).await;
    approve(&pool, b.id).await;

    let (found, total) = clinics::search(
        &pool,
        ClinicFilters {
            q: Some("alpha".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, a.id);

    let (city, total) = clinics::search(
        &pool,
        ClinicFilters {
            q: Some("boston".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(city.len(), 1);
    assert_eq!(city[0].id, b.id);
}

#[sqlx::test]
async fn country_and_city_filter(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "location@example.com").await;

    let us_miami = make_clinic(&pool, owner, "us-miami", "Miami Clinic", "US", "Miami", &[]).await;
    let ca_toronto = make_clinic(&pool, owner, "ca-toronto", "Toronto Clinic", "CA", "Toronto", &[]).await;
    approve(&pool, us_miami.id).await;
    approve(&pool, ca_toronto.id).await;

    let (us, total) = clinics::search(
        &pool,
        ClinicFilters {
            country: Some("US".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(us[0].id, us_miami.id);

    let (miami, total) = clinics::search(
        &pool,
        ClinicFilters {
            country: Some("US".into()),
            city: Some("Miami".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(miami[0].id, us_miami.id);

    let (empty, total) = clinics::search(
        &pool,
        ClinicFilters {
            country: Some("US".into()),
            city: Some("Toronto".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 0);
    assert!(empty.is_empty());
}

#[sqlx::test]
async fn only_approved_clinics_are_returned(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "status@example.com").await;

    let approved = make_clinic(&pool, owner, "approved", "Approved", "US", "Austin", &[]).await;
    let _pending = make_clinic(&pool, owner, "pending", "Pending", "US", "Austin", &[]).await;
    approve(&pool, approved.id).await;

    let (found, total) = clinics::search(&pool, filters(), 1, 20).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].status, ClinicStatus::Approved);
}

#[sqlx::test]
async fn treatment_filter_via_published_packages(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    treatments::seed(&pool).await.unwrap();
    let owner = provider(&pool, "treatment@example.com").await;

    let dental_clinic = make_clinic(&pool, owner, "dental-clinic", "Dental Plus", "US", "Austin", &[]).await;
    let hair_clinic = make_clinic(&pool, owner, "hair-clinic", "Hair Restore", "US", "Austin", &[]).await;
    approve(&pool, dental_clinic.id).await;
    approve(&pool, hair_clinic.id).await;

    let dental = treatments::by_slug(&pool, "dental-implants").await.unwrap().unwrap();
    let hair = treatments::by_slug(&pool, "hair-transplant").await.unwrap().unwrap();

    let dental_pkg = packages::create(
        &pool,
        dental_clinic.id,
        dental.id,
        "Dental Package",
        Some(1_000),
        Some(2_000),
        None,
        &[],
        &[],
    )
    .await
    .unwrap();
    packages::update(
        &pool,
        dental_pkg.id,
        "Dental Package",
        Some(1_000),
        Some(2_000),
        None,
        &[],
        &[],
        true,
    )
    .await
    .unwrap();

    let hair_pkg = packages::create(
        &pool,
        hair_clinic.id,
        hair.id,
        "Hair Package",
        Some(500),
        Some(1_000),
        None,
        &[],
        &[],
    )
    .await
    .unwrap();
    packages::update(
        &pool,
        hair_pkg.id,
        "Hair Package",
        Some(500),
        Some(1_000),
        None,
        &[],
        &[],
        true,
    )
    .await
    .unwrap();

    let (found, total) = clinics::search(
        &pool,
        ClinicFilters {
            treatment: Some("dental-implants".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, dental_clinic.id);
}

#[sqlx::test]
async fn price_range_filter(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    treatments::seed(&pool).await.unwrap();
    let owner = provider(&pool, "price@example.com").await;

    let cheap_clinic = make_clinic(&pool, owner, "cheap", "Cheap Clinic", "US", "Austin", &[]).await;
    let pricey_clinic = make_clinic(&pool, owner, "pricey", "Pricey Clinic", "US", "Austin", &[]).await;
    approve(&pool, cheap_clinic.id).await;
    approve(&pool, pricey_clinic.id).await;

    let dental = treatments::by_slug(&pool, "dental-implants").await.unwrap().unwrap();

    let cheap_pkg = packages::create(
        &pool,
        cheap_clinic.id,
        dental.id,
        "Cheap",
        Some(100),
        Some(500),
        None,
        &[],
        &[],
    )
    .await
    .unwrap();
    packages::update(
        &pool, cheap_pkg.id, "Cheap", Some(100), Some(500), None, &[], &[], true,
    )
    .await
    .unwrap();

    let pricey_pkg = packages::create(
        &pool,
        pricey_clinic.id,
        dental.id,
        "Pricey",
        Some(5_000),
        Some(10_000),
        None,
        &[],
        &[],
    )
    .await
    .unwrap();
    packages::update(
        &pool, pricey_pkg.id, "Pricey", Some(5_000), Some(10_000), None, &[], &[], true,
    )
    .await
    .unwrap();

    let (under_1k, total) = clinics::search(
        &pool,
        ClinicFilters {
            max_price: Some(1_000),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(under_1k[0].id, cheap_clinic.id);

    let (over_1k, total) = clinics::search(
        &pool,
        ClinicFilters {
            min_price: Some(1_000),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(over_1k[0].id, pricey_clinic.id);
}

#[sqlx::test]
async fn accreditation_filter(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "accreditation@example.com").await;

    let jci = make_clinic(&pool, owner, "jci", "JCI Clinic", "US", "Austin", &["JCI".into()]).await;
    let iso = make_clinic(
        &pool,
        owner,
        "iso",
        "ISO Clinic",
        "US",
        "Boston",
        &["ISO".into()],
    )
    .await;
    approve(&pool, jci.id).await;
    approve(&pool, iso.id).await;

    let (found, total) = clinics::search(
        &pool,
        ClinicFilters {
            accreditation: Some("JCI".into()),
            ..filters()
        },
        1,
        20,
    )
    .await
    .unwrap();
    assert_eq!(total, 1);
    assert_eq!(found[0].id, jci.id);
}

#[sqlx::test]
async fn pagination_and_total_count(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "pagination@example.com").await;

    for i in 0..5 {
        let clinic = make_clinic(
            &pool,
            owner,
            &format!("page-clinic-{i}"),
            &format!("Page Clinic {i}"),
            "US",
            "Austin",
            &[],
        )
        .await;
        approve(&pool, clinic.id).await;
    }

    let (page1, total) = clinics::search(&pool, filters(), 1, 2).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(page1.len(), 2);

    let (page2, total) = clinics::search(&pool, filters(), 2, 2).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(page2.len(), 2);

    let (page3, total) = clinics::search(&pool, filters(), 3, 2).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(page3.len(), 1);

    let (out_of_range, total) = clinics::search(&pool, filters(), 10, 2).await.unwrap();
    assert_eq!(total, 5);
    assert!(out_of_range.is_empty());
}

#[sqlx::test]
async fn per_page_is_clamped(pool: PgPool) {
    chassis::db::migrate(&pool).await.unwrap();
    let owner = provider(&pool, "clamp@example.com").await;

    for i in 0..3 {
        let clinic = make_clinic(
            &pool,
            owner,
            &format!("clamp-clinic-{i}"),
            &format!("Clamp Clinic {i}"),
            "US",
            "Austin",
            &[],
        )
        .await;
        approve(&pool, clinic.id).await;
    }

    let (found, _) = clinics::search(&pool, filters(), 1, 0).await.unwrap();
    assert_eq!(found.len(), 1);

    let (found, _) = clinics::search(&pool, filters(), 1, 100).await.unwrap();
    assert_eq!(found.len(), 3);
}
