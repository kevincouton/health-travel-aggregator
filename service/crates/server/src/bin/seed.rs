//! Idempotent database seed helper for end-to-end testing.
//!
//! Run from the `service/` directory with a `DATABASE_URL` and
//! `SESSION_SIGNING_KEY` in the environment:
//!
//! ```bash
//! DATABASE_URL=postgres://health_travel:health_travel@localhost:5432/health_travel \
//!   SESSION_SIGNING_KEY=... \
//!   cargo run --bin seed
//! ```

use anyhow::Context;
use chassis::{auth, db, locations, treatments, users::UserRole};
use sqlx::{query, query_as};
use uuid::Uuid;

const PROVIDER_EMAIL: &str = "provider@example.com";
const ADMIN_EMAIL: &str = "admin@example.com";
const PASSWORD: &str = "Password123!";
const CLINIC_SLUG: &str = "istanbul-smile-clinic";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;

    let pool = db::connect(&database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    db::migrate(&pool)
        .await
        .context("failed to run database migrations")?;

    locations::seed(&pool)
        .await
        .context("failed to seed locations")?;

    treatments::seed(&pool)
        .await
        .context("failed to seed treatments")?;

    let password_hash = auth::hash_password(PASSWORD).context("failed to hash seed password")?;

    // Upsert the provider user used by the smoke tests.
    let provider_id: Uuid = query_as::<_, (Uuid,)>(
        "INSERT INTO users (email, role, password_hash, email_verified_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (email) DO UPDATE SET
           role = EXCLUDED.role,
           password_hash = EXCLUDED.password_hash,
           email_verified_at = COALESCE(users.email_verified_at, NOW())
         RETURNING id",
    )
    .bind(PROVIDER_EMAIL)
    .bind(UserRole::ProviderAdmin)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .context("failed to upsert provider user")?
    .0;

    // Upsert the platform admin user.
    let admin_id: Uuid = query_as::<_, (Uuid,)>(
        "INSERT INTO users (email, role, password_hash, email_verified_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (email) DO UPDATE SET
           role = EXCLUDED.role,
           password_hash = EXCLUDED.password_hash,
           email_verified_at = COALESCE(users.email_verified_at, NOW())
         RETURNING id",
    )
    .bind(ADMIN_EMAIL)
    .bind(UserRole::PlatformAdmin)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .context("failed to upsert admin user")?
    .0;

    // Upsert the approved demo clinic owned by the provider.
    let clinic_id: Uuid = query_as::<_, (Uuid,)>(
        "INSERT INTO clinics
           (owner_user_id, name, slug, country_code, city, accreditations, description, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'approved')
         ON CONFLICT (slug) DO UPDATE SET
           owner_user_id = EXCLUDED.owner_user_id,
           name = EXCLUDED.name,
           country_code = EXCLUDED.country_code,
           city = EXCLUDED.city,
           accreditations = EXCLUDED.accreditations,
           description = EXCLUDED.description,
           status = EXCLUDED.status
         RETURNING id",
    )
    .bind(provider_id)
    .bind("Istanbul Smile Clinic")
    .bind(CLINIC_SLUG)
    .bind("TR")
    .bind("Istanbul")
    .bind(vec!["JCI".to_string(), "ISO 9001".to_string()])
    .bind(Some(
        "Premium dental and hair transplant clinic in Istanbul.",
    ))
    .fetch_one(&pool)
    .await
    .context("failed to upsert demo clinic")?
    .0;

    // Optionally seed a published package for the demo clinic.
    if let Some((treatment_id,)) =
        query_as::<_, (Uuid,)>("SELECT id FROM treatments WHERE slug = $1")
            .bind("dental-implants")
            .fetch_optional(&pool)
            .await
            .context("failed to lookup dental-implants treatment")?
    {
        let existing = query_as::<_, (Uuid,)>(
            "SELECT id FROM packages WHERE clinic_id = $1 AND treatment_id = $2",
        )
        .bind(clinic_id)
        .bind(treatment_id)
        .fetch_optional(&pool)
        .await
        .context("failed to check for existing demo package")?;

        if existing.is_none() {
            query(
                "INSERT INTO packages
                   (clinic_id, treatment_id, name, price_min, price_max, duration_days, inclusions, exclusions, is_published)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)
                 ON CONFLICT DO NOTHING",
            )
            .bind(clinic_id)
            .bind(treatment_id)
            .bind("All-on-4 Dental Implants Package")
            .bind(2_500i32)
            .bind(4_500i32)
            .bind(7i32)
            .bind(vec!["Consultation".to_string(), "X-ray".to_string(), "Implants".to_string(), "Hotel transfer".to_string()])
            .bind(vec!["Flights".to_string(), "Personal expenses".to_string()])
            .execute(&pool)
            .await
            .context("failed to insert demo package")?;
        }
    }

    tracing::info!(
        provider_id = %provider_id,
        admin_id = %admin_id,
        clinic_id = %clinic_id,
        "database seeded successfully"
    );

    Ok(())
}
