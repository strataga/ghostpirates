-- Create tasks table
CREATE TYPE task_status AS ENUM (
    'pending',
    'assigned',
    'in_progress',
    'review',
    'completed',
    'failed',
    'revision_requested'
);

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    parent_task_id UUID REFERENCES tasks(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    acceptance_criteria JSONB NOT NULL DEFAULT '[]'::jsonb,
    assigned_to UUID REFERENCES team_members(id),
    assigned_by UUID REFERENCES team_members(id),
    status task_status NOT NULL DEFAULT 'pending',
    start_time TIMESTAMPTZ,
    completion_time TIMESTAMPTZ,
    revision_count INT NOT NULL DEFAULT 0,
    max_revisions INT NOT NULL DEFAULT 3,
    input_data JSONB,
    output_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_revisions CHECK (revision_count >= 0 AND revision_count <= max_revisions)
);

CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_parent_task_id ON tasks(parent_task_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
