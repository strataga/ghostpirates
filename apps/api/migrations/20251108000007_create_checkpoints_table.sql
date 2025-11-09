-- Create checkpoints table
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_number INT NOT NULL,
    context_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, step_number)
);

CREATE INDEX idx_checkpoints_task_id ON checkpoints(task_id);
CREATE INDEX idx_checkpoints_created_at ON checkpoints(created_at DESC);
