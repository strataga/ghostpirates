use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::types::{WorkerSpec, WorkerStatus, TaskOutput, Specialization};
use super::errors::{AgentError, AgentResult};

/// Worker Agent that executes specific tasks based on specialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAgent {
    pub id: Uuid,
    pub team_id: Uuid,
    pub specialization: Specialization,
    pub skills: Vec<String>,
    pub responsibilities: Vec<String>,
    pub required_tools: Vec<String>,
    pub status: WorkerStatus,
    pub assigned_task_id: Option<Uuid>,
}

impl WorkerAgent {
    /// Create a Worker Agent from a WorkerSpec
    pub fn from_spec(team_id: Uuid, spec: &WorkerSpec) -> Self {
        let specialization = match spec.specialization.as_str() {
            "Researcher" => Specialization::Researcher,
            "Coder" => Specialization::Coder,
            "Reviewer" => Specialization::Reviewer,
            "Tester" => Specialization::Tester,
            "Writer" => Specialization::Writer,
            _ => Specialization::Researcher, // Default
        };

        Self {
            id: Uuid::new_v4(),
            team_id,
            specialization,
            skills: spec.skills.clone(),
            responsibilities: spec.responsibilities.clone(),
            required_tools: spec.required_tools.clone(),
            status: WorkerStatus::Idle,
            assigned_task_id: None,
        }
    }

    /// Assign a task to this worker
    pub fn assign_task(&mut self, task_id: Uuid) -> AgentResult<()> {
        if self.status != WorkerStatus::Idle {
            return Err(AgentError::TaskExecutionFailed(
                format!("Worker {} is not idle (status: {:?})", self.id, self.status)
            ));
        }

        self.assigned_task_id = Some(task_id);
        self.status = WorkerStatus::Working;
        Ok(())
    }

    /// Get the current status of this worker
    pub fn get_status(&self) -> WorkerStatus {
        self.status
    }

    /// Check if this worker can handle a task based on required skills
    pub fn can_handle_task(&self, required_skills: &[String]) -> bool {
        required_skills.iter().any(|req_skill| {
            self.skills.iter().any(|skill| skill.to_lowercase().contains(&req_skill.to_lowercase()))
        })
    }

    /// Execute a task (stub implementation - will be fleshed out in Sprint 4)
    pub async fn execute_task(
        &mut self,
        task_id: Uuid,
    ) -> AgentResult<TaskOutput> {
        // TODO: Implement actual task execution in Sprint 4
        if self.assigned_task_id.is_none() {
            return Err(AgentError::TaskExecutionFailed(
                "No task assigned to worker".to_string()
            ));
        }

        // Mock output for now
        Ok(TaskOutput {
            task_id,
            worker_id: self.id,
            result: serde_json::json!({"status": "completed"}),
            artifacts: vec![],
            logs: vec!["Task executed successfully".to_string()],
            metadata: serde_json::json!({}),
        })
    }

    /// Report progress to the Manager
    pub async fn report_progress(&self) -> AgentResult<String> {
        Ok(format!(
            "Worker {} ({}) - Status: {:?}, Task: {:?}",
            self.id, self.specialization, self.status, self.assigned_task_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation_from_spec() {
        let team_id = Uuid::new_v4();
        let spec = WorkerSpec {
            specialization: "Coder".to_string(),
            skills: vec!["Rust".to_string(), "API".to_string()],
            responsibilities: vec!["Code implementation".to_string()],
            required_tools: vec!["cargo".to_string()],
        };

        let worker = WorkerAgent::from_spec(team_id, &spec);

        assert_eq!(worker.team_id, team_id);
        assert_eq!(worker.specialization, Specialization::Coder);
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert_eq!(worker.skills.len(), 2);
    }

    #[test]
    fn test_assign_task() {
        let team_id = Uuid::new_v4();
        let spec = WorkerSpec {
            specialization: "Tester".to_string(),
            skills: vec![],
            responsibilities: vec![],
            required_tools: vec![],
        };

        let mut worker = WorkerAgent::from_spec(team_id, &spec);
        let task_id = Uuid::new_v4();

        let result = worker.assign_task(task_id);
        assert!(result.is_ok());
        assert_eq!(worker.assigned_task_id, Some(task_id));
        assert_eq!(worker.status, WorkerStatus::Working);
    }

    #[test]
    fn test_can_handle_task() {
        let team_id = Uuid::new_v4();
        let spec = WorkerSpec {
            specialization: "Coder".to_string(),
            skills: vec!["Rust".to_string(), "Python".to_string()],
            responsibilities: vec![],
            required_tools: vec![],
        };

        let worker = WorkerAgent::from_spec(team_id, &spec);

        assert!(worker.can_handle_task(&vec!["Rust".to_string()]));
        assert!(worker.can_handle_task(&vec!["Python".to_string()]));
        assert!(!worker.can_handle_task(&vec!["JavaScript".to_string()]));
    }
}
