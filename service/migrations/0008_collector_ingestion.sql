-- Collector ingestion support: source attribution on clinics, moderation flag
-- reasons, and the clinic-claim intent flow.
ALTER TABLE clinics
    ADD COLUMN source TEXT,
    ADD COLUMN external_ref TEXT,
    ADD COLUMN flag_reason TEXT;

-- Idempotent re-ingestion: one row per (source, external_ref). Provider-created
-- clinics have NULL source and are excluded from the index.
CREATE UNIQUE INDEX idx_clinics_source_external_ref
    ON clinics(source, external_ref)
    WHERE source IS NOT NULL;

CREATE TYPE claim_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE clinic_claims (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT,
    status claim_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_clinic_claims_status ON clinic_claims(status);
CREATE INDEX idx_clinic_claims_clinic ON clinic_claims(clinic_id);

-- A provider may only have one open claim per clinic.
CREATE UNIQUE INDEX idx_clinic_claims_open
    ON clinic_claims(clinic_id, user_id)
    WHERE status = 'pending';
