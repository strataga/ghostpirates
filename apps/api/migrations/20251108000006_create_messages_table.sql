-- Create messages table
CREATE TYPE message_type AS ENUM (
    'task_assignment',
    'task_completion',
    'revision_request',
    'approval',
    'rejection',
    'agent_communication',
    'system_event'
);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    from_agent_id UUID NOT NULL,
    to_agent_id UUID,
    message_type message_type NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_team_id ON messages(team_id);
CREATE INDEX idx_messages_type ON messages(message_type);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX idx_messages_from_agent ON messages(from_agent_id);
