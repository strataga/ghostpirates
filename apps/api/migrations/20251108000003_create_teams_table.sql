-- Create teams table
CREATE TYPE team_status AS ENUM (
    'pending',
    'planning',
    'active',
    'completed',
    'failed',
    'archived'
);

CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    goal TEXT NOT NULL,
    status team_status NOT NULL DEFAULT 'pending',
    manager_agent_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    budget_limit DECIMAL(12,2),
    metadata JSONB DEFAULT '{}'::jsonb,
    CONSTRAINT positive_budget CHECK (budget_limit IS NULL OR budget_limit > 0)
);

CREATE INDEX idx_teams_company_id ON teams(company_id);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_teams_created_by ON teams(created_by);
CREATE INDEX idx_teams_created_at ON teams(created_at DESC);
