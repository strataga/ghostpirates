-- Create cost_tracking table
CREATE TABLE cost_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
    model_name VARCHAR(50) NOT NULL,
    input_tokens INT NOT NULL DEFAULT 0,
    output_tokens INT NOT NULL DEFAULT 0,
    total_cost DECIMAL(12,6) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_tokens CHECK (input_tokens >= 0 AND output_tokens >= 0),
    CONSTRAINT positive_cost CHECK (total_cost >= 0)
);

CREATE INDEX idx_cost_tracking_team_id ON cost_tracking(team_id);
CREATE INDEX idx_cost_tracking_task_id ON cost_tracking(task_id);
CREATE INDEX idx_cost_tracking_created_at ON cost_tracking(created_at DESC);
