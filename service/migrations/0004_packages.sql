CREATE TABLE packages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    treatment_id UUID NOT NULL REFERENCES treatments(id),
    name TEXT NOT NULL,
    price_min INTEGER,
    price_max INTEGER,
    duration_days INTEGER,
    inclusions TEXT[] NOT NULL DEFAULT '{}',
    exclusions TEXT[] NOT NULL DEFAULT '{}',
    is_published BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_packages_clinic ON packages(clinic_id);
CREATE INDEX idx_packages_treatment ON packages(treatment_id);
