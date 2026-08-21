CREATE TYPE clinic_status AS ENUM ('draft', 'pending', 'approved', 'suspended');

CREATE TABLE clinics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    country_code CHAR(2) NOT NULL,
    city TEXT NOT NULL,
    accreditations TEXT[] NOT NULL DEFAULT '{}',
    description TEXT,
    status clinic_status NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_clinics_status ON clinics(status);
CREATE INDEX idx_clinics_country_city ON clinics(country_code, city);
