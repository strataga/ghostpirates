# Phase 2: Agent System Implementation

**Duration**: Weeks 3-4 (14 days)
**Goal**: Manager Agent → Worker Agents → LLM Integration → Team Formation
**Dependencies**: Phase 1 complete (Database + API Foundation)

---

## Epic 1: Manager Agent Implementation

### Task 1.1: Manager Agent Core Logic

**Type**: Backend
**Dependencies**: Database schema complete

**Subtasks**:

- [ ] 1.1.1: Create Manager Agent struct

```rust
// apps/api/src/agents/manager.rs
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::infrastructure::llm::ClaudeClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerAgent {
    pub id: Uuid,
    pub team_id: Uuid,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl ManagerAgent {
    pub fn new(team_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
    
    pub async fn analyze_goal(&self, goal: &str) -> Result<GoalAnalysis, AgentError> {
        // Implement goal analysis with Claude
    }
    
    pub async fn form_team(&self, analysis: &GoalAnalysis) -> Result<Vec<WorkerSpec>, AgentError> {
        // Create worker specifications
    }
    
    pub async fn decompose_goal(&self, goal: &str) -> Result<Vec<Task>, AgentError> {
        // Break goal into tasks
    }
    
    pub async fn review_task(&self, task: &Task, output: &TaskOutput) -> Result<ReviewDecision, AgentError> {
        // Review worker output
    }
}
```

- [ ] 1.1.2: Implement goal analysis

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GoalAnalysis {
    pub core_objective: String,
    pub subtasks: Vec<String>,
    pub required_specializations: Vec<String>,
    pub estimated_timeline_hours: f32,
    pub potential_blockers: Vec<String>,
    pub success_criteria: Vec<String>,
}

impl ManagerAgent {
    pub async fn analyze_goal(&self, goal: &str) -> Result<GoalAnalysis, AgentError> {
        let system_prompt = format!(
            "You are a highly skilled project manager analyzing project goals. \
            Analyze the following goal and provide structured output in JSON format."
        );
        
        let user_prompt = format!(
            "Goal: {}\n\n\
            Provide:\n\
            1. Core objective (one sentence)\n\
            2. Key subtasks (ordered list)\n\
            3. Required specializations (types of workers needed)\n\
            4. Estimated timeline (hours)\n\
            5. Potential blockers\n\
            6. Success criteria",
            goal
        );

        let response = self.llm_client.complete(&system_prompt, &user_prompt).await?;
        let analysis: GoalAnalysis = serde_json::from_str(&response)?;
        
        Ok(analysis)
    }
}
```

- [ ] 1.1.3: Implement team formation logic

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerSpec {
    pub specialization: String,
    pub skills: Vec<String>,
    pub responsibilities: Vec<String>,
    pub required_tools: Vec<String>,
}

impl ManagerAgent {
    pub async fn form_team(&self, analysis: &GoalAnalysis) -> Result<Vec<WorkerSpec>, AgentError> {
        let prompt = format!(
            "Create 3-5 specialized worker types for this goal:\n{}\n\n\
            Subtasks: {:?}\n\n\
            For each worker, provide:\n\
            - Role name and specialization\n\
            - Key skills required\n\
            - Primary responsibilities\n\
            - Tools they'll need",
            analysis.core_objective,
            analysis.subtasks
        );

        let response = self.llm_client.complete(&TEAM_FORMATION_PROMPT, &prompt).await?;
        let workers: Vec<WorkerSpec> = serde_json::from_str(&response)?;
        
        // Validate 3-5 workers
        if workers.len() < 3 || workers.len() > 5 {
            return Err(AgentError::InvalidTeamSize(workers.len()));
        }
        
        Ok(workers)
    }
}
```

- [ ] 1.1.4: Implement task decomposition

```rust
impl ManagerAgent {
    pub async fn decompose_goal(&self, goal: &str) -> Result<Vec<Task>, AgentError> {
        let prompt = format!(
            "Decompose this goal into concrete tasks:\n{}\n\n\
            For each task provide:\n\
            - Title\n\
            - Detailed description\n\
            - Acceptance criteria (3-5 checkable items)\n\
            - Required skills\n\
            - Estimated tokens/complexity",
            goal
        );

        let response = self.llm_client.complete(&TASK_DECOMPOSITION_PROMPT, &prompt).await?;
        let task_data: Vec<TaskData> = serde_json::from_str(&response)?;
        
        let mut tasks = Vec::new();
        for data in task_data {
            tasks.push(Task {
                id: Uuid::new_v4(),
                team_id: self.team_id,
                title: data.title,
                description: data.description,
                acceptance_criteria: data.acceptance_criteria,
                status: TaskStatus::Pending,
                ..Default::default()
            });
        }
        
        Ok(tasks)
    }
}
```

- [ ] 1.1.5: Implement review and revision logic

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approved,
    RevisionRequested { feedback: String },
    Rejected { reason: String },
}

impl ManagerAgent {
    pub async fn review_task(&self, task: &Task, output: &TaskOutput) -> Result<ReviewDecision, AgentError> {
        let prompt = format!(
            "Review this task output:\n\n\
            Task: {}\n\
            Description: {}\n\
            Acceptance Criteria:\n{}\n\n\
            Output: {}\n\n\
            Decide: Approve, Request Revision, or Reject",
            task.title,
            task.description,
            task.acceptance_criteria.join("\n"),
            serde_json::to_string_pretty(&output.result)?
        );

        let response = self.llm_client.complete(&REVIEW_PROMPT, &prompt).await?;
        let decision: ReviewDecision = serde_json::from_str(&response)?;
        
        Ok(decision)
    }
}
```

**Acceptance Criteria**:

- [ ] Manager agent can analyze goals
- [ ] Team formation creates 3-5 workers
- [ ] Task decomposition generates actionable tasks
- [ ] Review logic provides specific feedback
- [ ] All LLM calls have error handling

---

[Continue with detailed Worker Agent implementation, LLM client wrappers, prompt templates, etc.]

## Success Criteria - Phase 2 Complete

- [ ] Manager agent operational
- [ ] Worker agents created dynamically
- [ ] LLM integration functional
- [ ] Team formation working
- [ ] Task decomposition generating tasks
- [ ] Review and revision loop operational

---

## Next Steps

Proceed to [06-phase-3-task-orchestration.md](./06-phase-3-task-orchestration.md) for task assignment and execution.

---

**Phase 2: Autonomous Agents Online 🤖**
