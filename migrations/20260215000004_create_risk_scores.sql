-- Create risk_scores table for tracking compliance risk assessments
CREATE TABLE IF NOT EXISTS risk_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    compliance_item_id UUID NOT NULL REFERENCES compliance_items(id) ON DELETE CASCADE,
    document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Risk assessment details
    risk_category VARCHAR(100) NOT NULL,
    risk_score INTEGER NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    risk_level VARCHAR(20) NOT NULL CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
    
    -- Assessment metadata
    assessment_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assessed_by VARCHAR(500),
    notes TEXT,
    
    -- AI analysis context
    ai_confidence FLOAT CHECK (ai_confidence >= 0 AND ai_confidence <= 1),
    ai_reasoning TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_risk_scores_compliance_item_id ON risk_scores(compliance_item_id);
CREATE INDEX IF NOT EXISTS idx_risk_scores_document_id ON risk_scores(document_id);
CREATE INDEX IF NOT EXISTS idx_risk_scores_user_id ON risk_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_risk_scores_risk_level ON risk_scores(risk_level);
CREATE INDEX IF NOT EXISTS idx_risk_scores_assessment_date ON risk_scores(assessment_date);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_risk_scores_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER risk_scores_updated_at
    BEFORE UPDATE ON risk_scores
    FOR EACH ROW
    EXECUTE FUNCTION update_risk_scores_updated_at();
