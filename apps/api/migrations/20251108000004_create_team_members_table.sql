-- Create team_members table
CREATE TYPE member_role AS ENUM ('manager', 'worker');
CREATE TYPE member_status AS ENUM ('active', 'idle', 'busy', 'offline');

CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    role member_role NOT NULL,
    specialization VARCHAR(100),
    status member_status NOT NULL DEFAULT 'active',
    current_workload INT NOT NULL DEFAULT 0,
    max_concurrent_tasks INT NOT NULL DEFAULT 3,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, agent_id),
    CONSTRAINT valid_workload CHECK (current_workload >= 0 AND current_workload <= max_concurrent_tasks)
);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);
CREATE INDEX idx_team_members_role ON team_members(role);
CREATE INDEX idx_team_members_status ON team_members(status);
