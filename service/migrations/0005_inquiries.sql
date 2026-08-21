CREATE TYPE inquiry_status AS ENUM ('new', 'contacted', 'converted', 'closed');

CREATE TABLE inquiries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    patient_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    package_id UUID REFERENCES packages(id) ON DELETE SET NULL,
    status inquiry_status NOT NULL DEFAULT 'new',
    medical_notes TEXT,
    preferred_dates TEXT,
    contact_email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inquiries_clinic ON inquiries(clinic_id);
CREATE INDEX idx_inquiries_patient ON inquiries(patient_user_id);
