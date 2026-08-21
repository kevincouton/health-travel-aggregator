CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    patient_user_id UUID NOT NULL REFERENCES users(id),
    inquiry_id UUID NOT NULL REFERENCES inquiries(id),
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(clinic_id, inquiry_id)
);

CREATE INDEX idx_reviews_clinic ON reviews(clinic_id);
