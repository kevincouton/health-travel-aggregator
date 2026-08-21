use sqlx::PgPool;

#[sqlx::test]
async fn locations_seed_and_list(pool: PgPool) {
    chassis::locations::seed(&pool).await.unwrap();
    let locations = chassis::locations::list(&pool).await.unwrap();

    assert!(locations.len() >= 20);
    assert!(locations
        .iter()
        .any(|l| l.country_code == "US" && l.city == "New York"));
    assert!(locations
        .iter()
        .any(|l| l.country_code == "TH" && l.city == "Bangkok"));

    // Idempotent: second seed should not duplicate rows.
    chassis::locations::seed(&pool).await.unwrap();
    let locations_after = chassis::locations::list(&pool).await.unwrap();
    assert_eq!(locations.len(), locations_after.len());
}

#[sqlx::test]
async fn treatments_seed_and_list(pool: PgPool) {
    chassis::treatments::seed(&pool).await.unwrap();
    let treatments = chassis::treatments::list(&pool).await.unwrap();

    assert!(treatments.len() >= 10);
    assert!(treatments
        .iter()
        .any(|t| t.slug == "dental-implants" && t.category == "Dental"));

    // Idempotent: second seed should not duplicate rows.
    chassis::treatments::seed(&pool).await.unwrap();
    let treatments_after = chassis::treatments::list(&pool).await.unwrap();
    assert_eq!(treatments.len(), treatments_after.len());
}

#[sqlx::test]
async fn treatments_by_slug_lookup(pool: PgPool) {
    chassis::treatments::seed(&pool).await.unwrap();

    let found = chassis::treatments::by_slug(&pool, "ivf")
        .await
        .unwrap()
        .expect("ivf treatment should exist");
    assert_eq!(found.slug, "ivf");
    assert_eq!(found.name, "IVF");
    assert_eq!(found.category, "Fertility");

    let missing = chassis::treatments::by_slug(&pool, "not-a-treatment")
        .await
        .unwrap();
    assert!(missing.is_none());
}
