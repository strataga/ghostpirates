# Second-Order Architecture Solutions Part 2: Isolated Team-Based Agent Systems

## 7. CAPABILITY BUNDLING & SKILL ACQUISITION STRATEGY

### Problem

Workers get static skill profiles. But there's no strategy for *when* to acquire new skills, *how* to sequence learning, or *when* to specialize deeply vs maintain breadth. An agent that's excellent at research might waste time learning design. Conversely, neglecting skill growth leaves agents plateaued.

### Solution: Intelligent Skill Acquisition Sequencing

```rust
// src/agent_learning/skill_acquisition.rs

#[derive(Debug, Clone)]
pub struct SkillProficiency {
    pub skill_id: Uuid,
    pub agent_id: Uuid,
    pub skill_name: String,
    pub proficiency_level: f32,      // 0.0-1.0
    pub tasks_performed: u32,
    pub average_success_rate: f32,
    pub improvement_rate: f32,       // Per task, 0.001-0.1
    pub last_practiced: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SkillAcquisitionPlan {
    pub plan_id: Uuid,
    pub agent_id: Uuid,
    pub current_skills: Vec<SkillProficiency>,
    pub recommended_next_skill: Option<SkillRecommendation>,
    pub specialization_trajectory: SpecializationPath,
    pub learning_strategy: LearningStrategy,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SkillRecommendation {
    pub skill_name: String,
    pub rationale: String,
    pub estimated_tasks_to_master: u32,
    pub estimated_time_to_competence: Duration,
    pub expected_performance_lift: f32,
    pub synergy_with_existing: f32,  // 0-1, how much it enhances existing skills
    pub recommended_sequencing: SkillSequenceStep,
}

#[derive(Debug, Clone)]
pub struct SkillSequenceStep {
    pub order: u32,
    pub skill: String,
    pub phase: LearningPhase,
    pub prerequisite_skills: Vec<String>,
    pub estimated_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum LearningPhase {
    Foundation,        // Learn fundamentals
    Integration,       // Apply in concert with existing skills
    Mastery,          // Deep expertise
    Maintenance,      // Prevent skill decay
}

#[derive(Debug, Clone)]
pub enum SpecializationPath {
    DeepSpecialization,  // Master 2-3 skills deeply
    BroadGeneralist,     // Competent across many skills
    Hybrid {
        core_skills: Vec<String>,
        supporting_skills: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum LearningStrategy {
    Intensive,      // Master one skill quickly
    Distributed,    // Balance multiple skill development
    OpportunityBased, // Learn when tasks require it
}

#[derive(Debug, Clone)]
pub struct SkillDecayTracker {
    pub decay_id: Uuid,
    pub agent_id: Uuid,
    pub skill_name: String,
    pub last_practiced: DateTime<Utc>,
    pub proficiency_at_last_practice: f32,
    pub current_estimated_proficiency: f32,
    pub days_since_practice: u32,
    pub decay_rate: f32,  // Per day, 0.01-0.1
}

pub struct SkillAcquisitionPlanner {
    db: Arc<Database>,
    llm_client: Arc<LlmClient>,
}

impl SkillAcquisitionPlanner {
    /// Create acquisition plan for an agent
    pub async fn create_skill_acquisition_plan(
        &self,
        agent_id: Uuid,
    ) -> Result<SkillAcquisitionPlan, SkillError> {
        let agent = self.db.get_agent(agent_id).await?;
        let current_skills = self.db.get_agent_skills(agent_id).await?;
        let historical_performance = self.db.get_agent_performance_history(agent_id).await?;
        
        // Analyze current specialization
        let spec_path = self.analyze_specialization_path(&current_skills, &historical_performance)?;
        
        // Determine next skill to learn
        let next_skill = self.recommend_next_skill(agent_id, &current_skills, &spec_path).await?;
        
        // Choose learning strategy based on current load
        let strategy = self.select_learning_strategy(&current_skills, &next_skill).await?;
        
        let plan = SkillAcquisitionPlan {
            plan_id: Uuid::new_v4(),
            agent_id,
            current_skills,
            recommended_next_skill: next_skill,
            specialization_trajectory: spec_path,
            learning_strategy: strategy,
            created_at: Utc::now(),
        };
        
        self.db.store_skill_acquisition_plan(&plan).await?;
        
        Ok(plan)
    }
    
    /// Assign learning task to agent
    pub async fn assign_learning_task(
        &self,
        agent_id: Uuid,
        skill_to_learn: &str,
        difficulty: TaskDifficulty,
    ) -> Result<Uuid, SkillError> {
        // Create a task designed to teach the skill
        let task = Task {
            id: Uuid::new_v4(),
            title: format!("Learning task: {}", skill_to_learn),
            description: format!("Practice task for skill development: {}", skill_to_learn),
            category: skill_to_learn.to_string(),
            required_skills: vec![skill_to_learn.to_string()],
            acceptance_criteria: vec![
                format!("Demonstrate {} proficiency", skill_to_learn),
                "Complete within learning time budget".to_string(),
            ],
            assigned_to: vec![agent_id],
            learning_task: true,
            learning_skill: Some(skill_to_learn.to_string()),
            difficulty,
            created_at: Utc::now(),
            ..Default::default()
        };
        
        self.db.store_task(&task).await?;
        
        Ok(task.id)
    }
    
    /// Update proficiency based on task performance
    pub async fn update_skill_proficiency(
        &self,
        agent_id: Uuid,
        skill_name: &str,
        task_success: bool,
        quality_score: f32,
    ) -> Result<SkillProficiency, SkillError> {
        let mut skill = self.db.get_skill_proficiency(agent_id, skill_name).await?;
        
        // Update metrics
        skill.tasks_performed += 1;
        
        let new_success_rate = (skill.average_success_rate * (skill.tasks_performed - 1) as f32
            + if task_success { 1.0 } else { 0.0 })
            / skill.tasks_performed as f32;
        skill.average_success_rate = new_success_rate;
        
        // Apply proficiency improvement
        let improvement_delta = if task_success {
            quality_score * 0.1  // Successful tasks improve proficiency
        } else {
            -0.02  // Failed tasks slightly degrade
        };
        
        skill.proficiency_level = (skill.proficiency_level + improvement_delta).clamp(0.0, 1.0);
        skill.improvement_rate = improvement_delta;
        skill.last_practiced = Utc::now();
        
        self.db.store_skill_proficiency(&skill).await?;
        
        // Check if skill is now "mastered"
        if skill.proficiency_level > 0.85 {
            self.db.mark_skill_mastered(agent_id, skill_name).await?;
        }
        
        Ok(skill)
    }
    
    /// Monitor skill decay over time
    pub async fn monitor_skill_decay(&self, agent_id: Uuid) -> Result<Vec<SkillDecayTracker>, SkillError> {
        let skills = self.db.get_agent_skills(agent_id).await?;
        let mut decay_records = Vec::new();
        
        for skill in skills {
            let days_since = (Utc::now() - skill.last_practiced).num_days() as u32;
            
            // Decay formula: proficiency_now = proficiency_then * (1 - decay_rate)^days
            let decay_rate = self.estimate_decay_rate(&skill).await?;
            let current_estimated = skill.proficiency_level * (1.0 - decay_rate).powf(days_since as f32);
            
            if current_estimated < skill.proficiency_level - 0.1 {
                // Skill has decayed significantly
                let decay = SkillDecayTracker {
                    decay_id: Uuid::new_v4(),
                    agent_id,
                    skill_name: skill.skill_name.clone(),
                    last_practiced: skill.last_practiced,
                    proficiency_at_last_practice: skill.proficiency_level,
                    current_estimated_proficiency: current_estimated,
                    days_since_practice: days_since,
                    decay_rate,
                };
                
                decay_records.push(decay);
            }
        }
        
        Ok(decay_records)
    }
    
    /// Recommend refresher tasks for decaying skills
    pub async fn recommend_refresher_tasks(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<RefresherTaskRecommendation>, SkillError> {
        let decay_records = self.monitor_skill_decay(agent_id).await?;
        let mut recommendations = Vec::new();
        
        for decay in decay_records {
            if decay.days_since_practice > 14 {  // Haven't practiced in 2 weeks
                recommendations.push(RefresherTaskRecommendation {
                    skill_name: decay.skill_name.clone(),
                    urgency: if decay.current_estimated_proficiency < 0.5 {
                        RefresherUrgency::Critical
                    } else {
                        RefresherUrgency::Moderate
                    },
                    estimated_time_to_refresh: Duration::minutes(30),
                    task_type: RefresherTaskType::QuickReview,
                    priority: (decay.proficiency_at_last_practice * decay.days_since_practice as f32 * 10.0) as u32,
                });
            }
        }
        
        // Sort by priority
        recommendations.sort_by_key(|r| std::cmp::Reverse(r.priority));
        
        Ok(recommendations)
    }
    
    fn analyze_specialization_path(
        &self,
        skills: &[SkillProficiency],
        _historical_performance: &[PerformanceRecord],
    ) -> Result<SpecializationPath, SkillError> {
        // Count skills by proficiency level
        let deep_skills: Vec<_> = skills.iter()
            .filter(|s| s.proficiency_level > 0.8)
            .collect();
        
        let broad_skills: Vec<_> = skills.iter()
            .filter(|s| s.proficiency_level > 0.5 && s.proficiency_level <= 0.8)
            .collect();
        
        if deep_skills.len() >= 2 {
            Ok(SpecializationPath::DeepSpecialization)
        } else if broad_skills.len() >= 4 {
            Ok(SpecializationPath::BroadGeneralist)
        } else {
            Ok(SpecializationPath::Hybrid {
                core_skills: deep_skills.iter().map(|s| s.skill_name.clone()).collect(),
                supporting_skills: broad_skills.iter().map(|s| s.skill_name.clone()).collect(),
            })
        }
    }
    
    async fn recommend_next_skill(
        &self,
        agent_id: Uuid,
        current_skills: &[SkillProficiency],
        spec_path: &SpecializationPath,
    ) -> Result<Option<SkillRecommendation>, SkillError> {
        // Get all tasks this agent is working on
        let upcoming_tasks = self.db.get_agent_upcoming_tasks(agent_id).await?;
        
        // Identify skills needed by upcoming tasks
        let mut needed_skills: std::collections::HashMap<String, u32> = Default::default();
        
        for task in upcoming_tasks {
            for req_skill in &task.required_skills {
                let has_skill = current_skills.iter()
                    .any(|s| s.skill_name == *req_skill && s.proficiency_level > 0.5);
                
                if !has_skill {
                    *needed_skills.entry(req_skill.clone()).or_insert(0) += 1;
                }
            }
        }
        
        if needed_skills.is_empty() {
            return Ok(None);
        }
        
        // Pick highest-demand needed skill
        let (next_skill, _demand) = needed_skills.into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap();
        
        // Estimate learning cost
        let tasks_to_master = self.estimate_tasks_to_master(&next_skill).await?;
        let synergy = self.calculate_skill_synergy(&next_skill, current_skills).await?;
        
        Ok(Some(SkillRecommendation {
            skill_name: next_skill.clone(),
            rationale: format!("Required by {} upcoming tasks", needed_skills.len()),
            estimated_tasks_to_master: tasks_to_master,
            estimated_time_to_competence: Duration::hours((tasks_to_master * 2) as i64),
            expected_performance_lift: 0.15,  // Would calculate more accurately
            synergy_with_existing: synergy,
            recommended_sequencing: SkillSequenceStep {
                order: 1,
                skill: next_skill,
                phase: LearningPhase::Foundation,
                prerequisite_skills: vec![],
                estimated_duration: Duration::hours(8),
            },
        }))
    }
    
    async fn select_learning_strategy(
        &self,
        current_skills: &[SkillProficiency],
        _recommended: &Option<SkillRecommendation>,
    ) -> Result<LearningStrategy, SkillError> {
        // If agent has heavy workload, use opportunistic learning
        let avg_utilization = current_skills.iter()
            .map(|s| s.tasks_performed as f32)
            .sum::<f32>() / current_skills.len().max(1) as f32;
        
        if avg_utilization > 10.0 {
            Ok(LearningStrategy::OpportunityBased)
        } else {
            Ok(LearningStrategy::Intensive)
        }
    }
    
    async fn estimate_decay_rate(&self, skill: &SkillProficiency) -> Result<f32, SkillError> {
        // Expertise-dependent: harder skills decay faster without practice
        // Rough formula: base 0.05 per day + (1 - proficiency) * 0.05
        Ok(0.05 + (1.0 - skill.proficiency_level) * 0.05)
    }
    
    async fn estimate_tasks_to_master(&self, _skill: &str) -> Result<u32, SkillError> {
        Ok(10)  // Would vary by skill complexity
    }
    
    async fn calculate_skill_synergy(
        &self,
        new_skill: &str,
        current_skills: &[SkillProficiency],
    ) -> Result<f32, SkillError> {
        // High synergy if new skill complements existing ones
        if current_skills.iter().any(|s| s.skill_name == "research" && new_skill == "analysis") {
            Ok(0.9)
        } else if current_skills.iter().any(|s| s.skill_name == "writing" && new_skill == "editing") {
            Ok(0.85)
        } else {
            Ok(0.5)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RefresherTaskRecommendation {
    pub skill_name: String,
    pub urgency: RefresherUrgency,
    pub estimated_time_to_refresh: Duration,
    pub task_type: RefresherTaskType,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub enum RefresherUrgency {
    Critical,
    Moderate,
    Low,
}

#[derive(Debug, Clone)]
pub enum RefresherTaskType {
    QuickReview,
    PracticeProblem,
    MiniProject,
}

#[derive(Debug, Clone)]
pub enum TaskDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug)]
pub enum SkillError {
    DatabaseError(String),
    AgentNotFound,
    SkillNotFound,
}
```

### Database Schema for Skill Acquisition

```sql
CREATE TABLE skill_proficiencies (
    proficiency_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    skill_name VARCHAR(100) NOT NULL,
    
    proficiency_level DECIMAL(3,2),     -- 0.0 to 1.0
    tasks_performed INT DEFAULT 0,
    average_success_rate DECIMAL(3,2),
    improvement_rate DECIMAL(4,3),
    last_practiced TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    mastered BOOLEAN DEFAULT false,
    mastered_at TIMESTAMP,
    
    UNIQUE(agent_id, skill_name),
    CREATED_AT TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE skill_acquisition_plans (
    plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    
    current_skills JSONB,
    recommended_next_skill JSONB,
    specialization_path VARCHAR(50),
    learning_strategy VARCHAR(50),
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE skill_decay_tracking (
    decay_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    skill_name VARCHAR(100),
    
    last_practiced TIMESTAMP,
    proficiency_at_last_practice DECIMAL(3,2),
    current_estimated_proficiency DECIMAL(3,2),
    days_since_practice INT,
    decay_rate DECIMAL(4,3),
    
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE learning_tasks (
    task_id UUID PRIMARY KEY REFERENCES tasks(id),
    skill_to_learn VARCHAR(100) NOT NULL,
    difficulty VARCHAR(50),
    target_proficiency DECIMAL(3,2),
    learning_deadline TIMESTAMP,
    
    proficiency_gained DECIMAL(3,2),
    completed_at TIMESTAMP
);

CREATE INDEX idx_proficiency_agent ON skill_proficiencies(agent_id);
CREATE INDEX idx_proficiency_mastered ON skill_proficiencies(agent_id) WHERE mastered = true;
CREATE INDEX idx_decay_agent ON skill_decay_tracking(agent_id, recorded_at DESC);
CREATE INDEX idx_learning_tasks_skill ON learning_tasks(skill_to_learn);
```

---

## 8. FAILURE MODE CATEGORIZATION

### Problem

When tasks fail, you escalate to humans. But different failures need different interventions. Ambiguity (needs clarification) vs capability gap (needs different agent) vs coordination failure (needs process change) vs external failure (tool down) all route to the same "escalate to human" pile.

### Solution: Structured Failure Classification & Targeted Escalation

```rust
// src/error_handling/failure_classification.rs

#[derive(Debug, Clone)]
pub struct FailureAnalysis {
    pub failure_id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    
    pub failure_category: FailureCategory,
    pub root_cause: RootCause,
    pub confidence: f32,  // 0-1
    pub evidence: Vec<String>,
    
    pub recommended_action: RecommendedAction,
    pub escalation_priority: EscalationPriority,
    pub human_intervention_type: Option<HumanInterventionType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureCategory {
    Ambiguity,           // Task description unclear
    CapabilityGap,       // Agent lacks required skill
    CoordinationFailure, // Multi-agent coordination broke down
    ToolFailure,         // External tool/API failed
    ContextLimitation,   // Task exceeds context window
    BoundaryViolation,   // Agent violated constraints
    LogicalImpossibility, // Task contradicts itself
    ResourceExhaustion,  // Out of budget/tokens
    TemporaryOutage,     // Transient error (retry-able)
}

#[derive(Debug, Clone)]
pub enum RootCause {
    AmbiguousRequirements,
    MissingSkill(String),
    InsufficientContextWindow,
    ToolIntegrationDown(String),
    ConflictingRequirements,
    ExternalAPIRateLimit,
    InsufficientBudget,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Clarify,                          // Ask user to clarify
    Reassign,                         // Assign to different agent
    Decompose,                        // Break into smaller tasks
    RetryWithBackoff,                 // Transient, retry later
    ReplaceToolIntegration,           // Update tool/API
    ExpandBudget,                     // Allocate more budget
    ReduceScope,                      // Simplify requirements
    IncrementContextWindow,           // Use more context
    Abort,                            // Stop, not worth pursuing
}

#[derive(Debug, Clone)]
pub enum EscalationPriority {
    Critical,    // Needs immediate human intervention
    High,        // Escalate within 1 hour
    Medium,      // Escalate within 4 hours
    Low,         // Can wait, non-blocking
    NoEscalation, // Self-recoverable
}

#[derive(Debug, Clone)]
pub enum HumanInterventionType {
    Clarification,
    SkillGapResolution,
    ToolIntegrationFix,
    PolicyDecision,
    CreativeInput,
}

pub struct FailureClassifier {
    db: Arc<Database>,
    llm_client: Arc<LlmClient>,
}

impl FailureClassifier {
    /// Analyze task failure and classify
    pub async fn classify_failure(
        &self,
        task_id: Uuid,
        agent_id: Uuid,
        error_message: &str,
        execution_log: &ExecutionLog,
    ) -> Result<FailureAnalysis, ClassificationError> {
        // Collect evidence
        let mut evidence = vec![error_message.to_string()];
        
        // Check for tool failures
        if let Some(tool_failure) = self.detect_tool_failure(error_message, execution_log).await? {
            evidence.push(format!("Tool failure detected: {}", tool_failure));
        }
        
        // Use LLM to analyze context for ambiguity
        let ambiguity_score = self.assess_requirement_ambiguity(&execution_log.task_description).await?;
        if ambiguity_score > 0.7 {
            evidence.push("High ambiguity in task description detected".to_string());
        }
        
        // Check for capability gap
        let capability_gap = self.detect_capability_gap(agent_id, task_id).await?;
        
        // Check for coordination issues
        let coordination_issues = self.detect_coordination_failure(execution_log).await?;
        
        // Check resource limits
        let resource_issue = self.check_resource_constraints(execution_log).await?;
        
        // Determine primary failure category
        let (category, root_cause, confidence) = if let Some(tool) = self.extract_tool_from_error(error_message) {
            (FailureCategory::ToolFailure, RootCause::ToolIntegrationDown(tool), 0.9)
        } else if let Some(skill_gap) = capability_gap {
            (FailureCategory::CapabilityGap, RootCause::MissingSkill(skill_gap), 0.8)
        } else if !coordination_issues.is_empty() {
            (FailureCategory::CoordinationFailure, 
             RootCause::Other(coordination_issues.join("; ")), 0.7)
        } else if let Some(resource) = resource_issue {
            (FailureCategory::ResourceExhaustion, 
             RootCause::Other(resource), 0.85)
        } else if ambiguity_score > 0.7 {
            (FailureCategory::Ambiguity, 
             RootCause::AmbiguousRequirements, 
             ambiguity_score)
        } else {
            // Check if transient
            if self.is_transient_error(error_message) {
                (FailureCategory::TemporaryOutage, 
                 RootCause::Other("Transient error".to_string()), 0.7)
            } else {
                (FailureCategory::LogicalImpossibility,
                 RootCause::Other("Unknown failure".to_string()), 0.5)
            }
        };
        
        let recommended_action = self.recommend_action(&category, &root_cause);
        let escalation_priority = self.determine_escalation_priority(&category);
        let human_type = self.determine_human_intervention_type(&category);
        
        let analysis = FailureAnalysis {
            failure_id: Uuid::new_v4(),
            task_id,
            agent_id,
            timestamp: Utc::now(),
            failure_category: category,
            root_cause,
            confidence,
            evidence,
            recommended_action,
            escalation_priority,
            human_intervention_type: human_type,
        };
        
        self.db.store_failure_analysis(&analysis).await?;
        
        Ok(analysis)
    }
    
    /// Auto-recover based on failure category
    pub async fn attempt_auto_recovery(
        &self,
        analysis: &FailureAnalysis,
    ) -> Result<AutoRecoveryAttempt, ClassificationError> {
        let mut result = AutoRecoveryAttempt {
            failure_id: analysis.failure_id,
            attempted_actions: Vec::new(),
            success: false,
            recovery_timestamp: Utc::now(),
        };
        
        match &analysis.recommended_action {
            RecommendedAction::RetryWithBackoff => {
                // Wait and retry
                result.attempted_actions.push("Scheduled retry with exponential backoff".to_string());
                result.success = true;
            }
            RecommendedAction::Decompose => {
                // Try breaking task into smaller parts
                result.attempted_actions.push("Decomposing task into smaller subtasks".to_string());
                result.success = true;
            }
            RecommendedAction::Reassign => {
                // Find alternative agent
                if let Ok(alt_agent) = self.find_alternative_agent(analysis.agent_id, analysis.task_id).await {
                    result.attempted_actions.push(format!("Reassigned to agent {:?}", alt_agent));
                    result.success = true;
                }
            }
            _ => {
                // These require human intervention
            }
        }
        
        self.db.store_auto_recovery_attempt(&result).await?;
        
        Ok(result)
    }
    
    async fn detect_tool_failure(
        &self,
        error_message: &str,
        execution_log: &ExecutionLog,
    ) -> Result<Option<String>, ClassificationError> {
        for tool in &execution_log.tools_used {
            if error_message.to_lowercase().contains(&tool.to_lowercase()) {
                return Ok(Some(tool.clone()));
            }
        }
        
        // Check for common error patterns
        if error_message.contains("503") || error_message.contains("429") {
            return Ok(Some("External API".to_string()));
        }
        
        Ok(None)
    }
    
    async fn assess_requirement_ambiguity(&self, task_description: &str) -> Result<f32, ClassificationError> {
        let prompt = format!(
            "Rate the ambiguity of this task description from 0 (crystal clear) to 1 (extremely vague):\n{}\n\nRespond with JSON: {{\"ambiguity_score\": 0.3}}",
            task_description
        );
        
        let response = self.llm_client.generate_json(&prompt).await?;
        Ok(response["ambiguity_score"].as_f64().unwrap_or(0.5) as f32)
    }
    
    async fn detect_capability_gap(
        &self,
        agent_id: Uuid,
        task_id: Uuid,
    ) -> Result<Option<String>, ClassificationError> {
        let agent = self.db.get_agent(agent_id).await?;
        let task = self.db.get_task(task_id).await?;
        
        for required_skill in &task.required_skills {
            let has_skill = agent.profile.skills.iter()
                .any(|s| s.name.to_lowercase().contains(&required_skill.to_lowercase()) 
                    && s.proficiency > 0.5);
            
            if !has_skill {
                return Ok(Some(required_skill.clone()));
            }
        }
        
        Ok(None)
    }
    
    async fn detect_coordination_failure(
        &self,
        execution_log: &ExecutionLog,
    ) -> Result<Vec<String>, ClassificationError> {
        let mut issues = Vec::new();
        
        // Check for long gaps between agent messages
        for window in execution_log.messages.windows(2) {
            let gap_ms = window[1].timestamp.timestamp_millis() 
                - window[0].timestamp.timestamp_millis();
            
            if gap_ms > 300000 {  // 5 minute gap
                issues.push("Long coordination gap detected".to_string());
            }
        }
        
        // Check for conflicting outputs
        if execution_log.messages.len() > 2 {
            for i in 0..execution_log.messages.len() - 1 {
                let msg1 = &execution_log.messages[i];
                let msg2 = &execution_log.messages[i + 1];
                
                if msg1.agent_id != msg2.agent_id {
                    // Different agents - check for contradiction
                    if self.detect_output_contradiction(&msg1.content, &msg2.content).await? {
                        issues.push("Output contradiction between agents".to_string());
                    }
                }
            }
        }
        
        Ok(issues)
    }
    
    async fn check_resource_constraints(
        &self,
        execution_log: &ExecutionLog,
    ) -> Result<Option<String>, ClassificationError> {
        let total_cost: Decimal = execution_log.messages.iter()
            .filter_map(|m| m.cost_usd)
            .sum();
        
        if let Some(budget) = execution_log.budget_limit {
            if total_cost > budget {
                return Ok(Some(format!("Budget exceeded: spent ${} of ${}", total_cost, budget)));
            }
        }
        
        let total_tokens: u32 = execution_log.messages.iter()
            .filter_map(|m| m.token_count)
            .sum();
        
        if total_tokens > 128000 {
            return Ok(Some("Context window approaching limit".to_string()));
        }
        
        Ok(None)
    }
    
    async fn detect_output_contradiction(
        &self,
        output_a: &str,
        output_b: &str,
    ) -> Result<bool, ClassificationError> {
        let prompt = format!(
            "Do these outputs contradict each other?\nOutput A: {}\nOutput B: {}\n\nRespond: true/false",
            output_a, output_b
        );
        
        let response = self.llm_client.generate_text(&prompt).await?;
        Ok(response.to_lowercase().contains("true"))
    }
    
    fn extract_tool_from_error(&self, error_message: &str) -> Option<String> {
        if error_message.contains("API") {
            Some("API".to_string())
        } else if error_message.contains("database") {
            Some("database".to_string())
        } else if error_message.contains("network") {
            Some("network".to_string())
        } else {
            None
        }
    }
    
    fn is_transient_error(&self, error_message: &str) -> bool {
        error_message.contains("timeout")
            || error_message.contains("connection reset")
            || error_message.contains("temporarily unavailable")
    }
    
    fn recommend_action(
        &self,
        category: &FailureCategory,
        root_cause: &RootCause,
    ) -> RecommendedAction {
        match category {
            FailureCategory::Ambiguity => RecommendedAction::Clarify,
            FailureCategory::CapabilityGap => RecommendedAction::Reassign,
            FailureCategory::CoordinationFailure => RecommendedAction::Decompose,
            FailureCategory::ToolFailure => RecommendedAction::ReplaceToolIntegration,
            FailureCategory::ContextLimitation => RecommendedAction::ReduceScope,
            FailureCategory::ResourceExhaustion => RecommendedAction::ExpandBudget,
            FailureCategory::TemporaryOutage => RecommendedAction::RetryWithBackoff,
            FailureCategory::LogicalImpossibility => RecommendedAction::Abort,
            _ => RecommendedAction::Abort,
        }
    }
    
    fn determine_escalation_priority(&self, category: &FailureCategory) -> EscalationPriority {
        match category {
            FailureCategory::LogicalImpossibility => EscalationPriority::Critical,
            FailureCategory::BoundaryViolation => EscalationPriority::Critical,
            FailureCategory::Ambiguity => EscalationPriority::High,
            FailureCategory::ToolFailure => EscalationPriority::High,
            FailureCategory::TemporaryOutage => EscalationPriority::NoEscalation,
            FailureCategory::ResourceExhaustion => EscalationPriority::Medium,
            _ => EscalationPriority::Medium,
        }
    }
    
    fn determine_human_intervention_type(
        &self,
        category: &FailureCategory,
    ) -> Option<HumanInterventionType> {
        match category {
            FailureCategory::Ambiguity => Some(HumanInterventionType::Clarification),
            FailureCategory::CapabilityGap => Some(HumanInterventionType::SkillGapResolution),
            FailureCategory::ToolFailure => Some(HumanInterventionType::ToolIntegrationFix),
            FailureCategory::LogicalImpossibility => Some(HumanInterventionType::PolicyDecision),
            _ => None,
        }
    }
    
    async fn find_alternative_agent(
        &self,
        _failed_agent_id: Uuid,
        task_id: Uuid,
    ) -> Result<Uuid, ClassificationError> {
        // Find agent with better skills for this task
        let task = self.db.get_task(task_id).await?;
        let candidates = self.db.find_agents_with_skills(&task.required_skills).await?;
        
        if let Some(best) = candidates.first() {
            Ok(best.id)
        } else {
            Err(ClassificationError::NoAlternativeAgentFound)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutoRecoveryAttempt {
    pub failure_id: Uuid,
    pub attempted_actions: Vec<String>,
    pub success: bool,
    pub recovery_timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub enum ClassificationError {
    DatabaseError(String),
    LlmError(String),
    NoAlternativeAgentFound,
}
```

### Database Schema for Failure Classification

```sql
CREATE TABLE failure_analyses (
    failure_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    agent_id UUID NOT NULL REFERENCES agents(id),
    
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    failure_category VARCHAR(50),
    root_cause VARCHAR(100),
    confidence DECIMAL(3,2),
    
    evidence TEXT[],
    recommended_action VARCHAR(50),
    escalation_priority VARCHAR(50),
    human_intervention_type VARCHAR(50),
    
    escalated BOOLEAN DEFAULT false,
    escalated_to_user_id UUID,
    escalated_at TIMESTAMP
);

CREATE TABLE auto_recovery_attempts (
    recovery_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    failure_id UUID NOT NULL REFERENCES failure_analyses(failure_id),
    
    attempted_actions TEXT[],
    success BOOLEAN,
    recovery_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    if_success_rerun_task BOOLEAN DEFAULT true
);

CREATE TABLE failure_statistics (
    stat_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id),
    
    failure_category VARCHAR(50),
    occurrence_count INT DEFAULT 1,
    auto_recovery_rate DECIMAL(3,2),
    avg_time_to_resolution_minutes INT,
    
    last_occurrence TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_failures_category ON failure_analyses(failure_category);
CREATE INDEX idx_failures_priority ON failure_analyses(escalation_priority);
CREATE INDEX idx_recovery_success ON auto_recovery_attempts(success);
```

---

## 9. MANAGER AGENT BURNOUT PREVENTION

### Problem

Manager agents coordinate and review work across many workers. But there's no load balancing—a manager with a large team becomes a bottleneck. High-performing managers get more assignments, worsening the problem. Eventually the manager quality degrades from overload.

### Solution: Manager Load Monitoring & Dynamic Team Splitting

```rust
// src/manager_coordination/load_balancing.rs

#[derive(Debug, Clone)]
pub struct ManagerWorkloadMetrics {
    pub metrics_id: Uuid,
    pub manager_id: Uuid,
    pub measured_at: DateTime<Utc>,
    
    pub team_size: u32,
    pub active_tasks: u32,
    pub pending_reviews: u32,
    pub revision_requests: u32,
    
    pub avg_decision_latency_ms: u32,  // Time to make assignment decision
    pub avg_review_time_minutes: f32,
    pub quality_score: f32,            // Manager's review quality
    
    pub workload_utilization: f32,     // 0-1, percentage of capacity
    pub burnout_risk: BurnoutRiskLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BurnoutRiskLevel {
    Low,      // < 60% utilization
    Moderate, // 60-80%
    High,     // 80-95%
    Critical, // > 95%
}

#[derive(Debug, Clone)]
pub struct ManagerPerformanceTrend {
    pub trend_id: Uuid,
    pub manager_id: Uuid,
    pub measurement_period: Duration,
    
    pub quality_trend: f32,          // Positive = improving, negative = degrading
    pub decision_speed_trend: f32,   // Positive = faster
    pub workload_trend: f32,         // Positive = increasing
    
    pub correlation_quality_workload: f32,  // How strongly does workload affect quality?
    pub degradation_rate_per_addition: f32, // Quality drop per new worker added
    pub estimated_optimal_team_size: u32,
}

#[derive(Debug, Clone)]
pub struct TeamSplitRecommendation {
    pub recommendation_id: Uuid,
    pub original_team_id: Uuid,
    pub original_manager_id: Uuid,
    
    pub action: SplitAction,
    pub urgency: SplitUrgency,
    pub expected_outcome: SplitOutcome,
}

#[derive(Debug, Clone)]
pub enum SplitAction {
    SplitTeam {
        new_team_count: u32,
        worker_distribution: Vec<u32>,
        new_manager_candidate: Option<Uuid>,
    },
    HireCoordinator {
        role: String,
        responsibilities: Vec<String>,
    },
    ReduceScope,
}

#[derive(Debug, Clone)]
pub enum SplitUrgency {
    Immediate,
    Within24Hours,
    Within1Week,
    Preventive,
}

#[derive(Debug, Clone)]
pub struct SplitOutcome {
    pub expected_quality_improvement: f32,
    pub expected_latency_reduction: f32,
    pub estimated_implementation_overhead: Duration,
}

pub struct ManagerLoadBalancer {
    db: Arc<Database>,
}

impl ManagerLoadBalancer {
    /// Measure manager workload
    pub async fn measure_manager_workload(
        &self,
        manager_id: Uuid,
    ) -> Result<ManagerWorkloadMetrics, LoadError> {
        let manager = self.db.get_agent(manager_id).await?;
        let team = self.db.get_team_by_manager(manager_id).await?;
        let tasks = self.db.get_tasks_for_team(team.id).await?;
        
        let active_tasks = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count() as u32;
        let pending_reviews = tasks.iter().filter(|t| t.status == TaskStatus::PendingReview).count() as u32;
        let revision_requests = tasks.iter()
            .filter(|t| t.status == TaskStatus::InRevision)
            .count() as u32;
        
        // Calculate decision latency (time between task completion and next action)
        let messages = self.db.get_manager_messages(manager_id).await?;
        let avg_latency = self.calculate_decision_latency(&messages)?;
        
        // Get manager's review quality
        let review_quality = self.calculate_manager_review_quality(manager_id).await?;
        
        // Estimate workload utilization
        let workload_util = self.estimate_utilization(
            team.members.len() as u32,
            active_tasks,
            pending_reviews,
            avg_latency,
        );
        
        let burnout_risk = match workload_util {
            u if u < 0.6 => BurnoutRiskLevel::Low,
            u if u < 0.8 => BurnoutRiskLevel::Moderate,
            u if u < 0.95 => BurnoutRiskLevel::High,
            _ => BurnoutRiskLevel::Critical,
        };
        
        let metrics = ManagerWorkloadMetrics {
            metrics_id: Uuid::new_v4(),
            manager_id,
            measured_at: Utc::now(),
            team_size: team.members.len() as u32,
            active_tasks,
            pending_reviews,
            revision_requests,
            avg_decision_latency_ms: avg_latency,
            avg_review_time_minutes: 15.0,  // Would calculate
            quality_score: review_quality,
            workload_utilization: workload_util,
            burnout_risk,
        };
        
        self.db.store_manager_metrics(&metrics).await?;
        
        Ok(metrics)
    }
    
    /// Analyze manager performance trend over time
    pub async fn analyze_manager_trend(
        &self,
        manager_id: Uuid,
        period: Duration,
    ) -> Result<ManagerPerformanceTrend, LoadError> {
        let recent_metrics = self.db.get_manager_metrics_in_period(manager_id, period).await?;
        
        if recent_metrics.len() < 2 {
            return Err(LoadError::InsufficientData);
        }
        
        // Calculate trends
        let quality_delta = recent_metrics.last().unwrap().quality_score 
            - recent_metrics.first().unwrap().quality_score;
        
        let speed_delta = (recent_metrics.first().unwrap().avg_decision_latency_ms as f32)
            - (recent_metrics.last().unwrap().avg_decision_latency_ms as f32);
        
        let workload_delta = recent_metrics.last().unwrap().workload_utilization 
            - recent_metrics.first().unwrap().workload_utilization;
        
        // Calculate correlation between workload and quality
        let correlation = self.calculate_correlation(
            &recent_metrics.iter().map(|m| m.workload_utilization).collect::<Vec<_>>(),
            &recent_metrics.iter().map(|m| m.quality_score).collect::<Vec<_>>(),
        );
        
        // Estimate degradation
        let degradation_per_worker = if workload_delta > 0.0 {
            quality_delta / workload_delta
        } else {
            0.0
        };
        
        let optimal_size = self.estimate_optimal_team_size(manager_id, &recent_metrics).await?;
        
        let trend = ManagerPerformanceTrend {
            trend_id: Uuid::new_v4(),
            manager_id,
            measurement_period: period,
            quality_trend: quality_delta,
            decision_speed_trend: speed_delta,
            workload_trend: workload_delta,
            correlation_quality_workload: correlation,
            degradation_rate_per_addition: degradation_per_worker,
            estimated_optimal_team_size: optimal_size,
        };
        
        self.db.store_manager_trend(&trend).await?;
        
        Ok(trend)
    }
    
    /// Recommend team splitting or reorganization
    pub async fn recommend_load_rebalancing(
        &self,
        manager_id: Uuid,
    ) -> Result<Option<TeamSplitRecommendation>, LoadError> {
        let metrics = self.measure_manager_workload(manager_id).await?;
        
        if metrics.burnout_risk == BurnoutRiskLevel::Low 
            || metrics.burnout_risk == BurnoutRiskLevel::Moderate {
            return Ok(None);
        }
        
        let trend = self.analyze_manager_trend(manager_id, Duration::days(7)).await.ok();
        
        let (action, urgency) = if metrics.burnout_risk == BurnoutRiskLevel::Critical {
            // Immediate action needed
            let split = SplitAction::SplitTeam {
                new_team_count: (metrics.team_size as f32 / 3.0).ceil() as u32,
                worker_distribution: vec![],  // Would calculate
                new_manager_candidate: self.find_promotion_candidate(manager_id).await.ok(),
            };
            (split, SplitUrgency::Immediate)
        } else if let Some(ref t) = trend {
            if t.quality_trend < -0.1 && t.workload_trend > 0.1 {
                // Quality degrading with increased load
                let split = SplitAction::SplitTeam {
                    new_team_count: 2,
                    worker_distribution: vec![],
                    new_manager_candidate: self.find_promotion_candidate(manager_id).await.ok(),
                };
                (split, SplitUrgency::Within24Hours)
            } else {
                // Preventive: hire coordinator
                let coordinator = SplitAction::HireCoordinator {
                    role: "Coordination Assistant".to_string(),
                    responsibilities: vec![
                        "Track task status".to_string(),
                        "Schedule reviews".to_string(),
                        "Aggregate metrics".to_string(),
                    ],
                };
                (coordinator, SplitUrgency::Within1Week)
            }
        } else {
            let split = SplitAction::SplitTeam {
                new_team_count: 2,
                worker_distribution: vec![],
                new_manager_candidate: None,
            };
            (split, SplitUrgency::Within24Hours)
        };
        
        let recommendation = TeamSplitRecommendation {
            recommendation_id: Uuid::new_v4(),
            original_team_id: self.db.get_team_by_manager(manager_id).await?.id,
            original_manager_id: manager_id,
            action,
            urgency,
            expected_outcome: SplitOutcome {
                expected_quality_improvement: 0.15,
                expected_latency_reduction: 0.30,
                estimated_implementation_overhead: Duration::hours(4),
            },
        };
        
        self.db.store_split_recommendation(&recommendation).await?;
        
        Ok(Some(recommendation))
    }
    
    /// Execute team split
    pub async fn execute_team_split(
        &self,
        recommendation: &TeamSplitRecommendation,
    ) -> Result<Vec<Uuid>, LoadError> {
        let original_team = self.db.get_team(recommendation.original_team_id).await?;
        let mut new_team_ids = Vec::new();
        
        match &recommendation.action {
            SplitAction::SplitTeam { new_team_count, new_manager_candidate, .. } => {
                let workers = original_team.members.clone();
                let workers_per_team = (workers.len() / *new_team_count as usize).max(1);
                
                for i in 0..*new_team_count {
                    let start = i as usize * workers_per_team;
                    let end = ((i as usize + 1) * workers_per_team).min(workers.len());
                    let team_workers = workers[start..end].to_vec();
                    
                    let manager_id = if i == 0 {
                        recommendation.original_manager_id
                    } else if let Some(candidate) = new_manager_candidate {
                        *candidate
                    } else {
                        // Promote top worker to manager role
                        self.promote_worker_to_manager(&team_workers).await?
                    };
                    
                    let new_team = Team {
                        id: Uuid::new_v4(),
                        manager_id,
                        members: team_workers,
                        goal: original_team.goal.clone(),
                        created_at: Utc::now(),
                        ..Default::default()
                    };
                    
                    self.db.store_team(&new_team).await?;
                    new_team_ids.push(new_team.id);
                }
                
                // Archive original team
                self.db.mark_team_archived(recommendation.original_team_id).await?;
            }
            _ => {}
        }
        
        Ok(new_team_ids)
    }
    
    fn calculate_decision_latency(&self, messages: &[TeamMessage]) -> Result<u32, LoadError> {
        if messages.len() < 2 {
            return Ok(0);
        }
        
        let mut latencies = Vec::new();
        let mut last_worker_msg: Option<&TeamMessage> = None;
        
        for msg in messages {
            if msg.is_manager_decision {
                if let Some(prev) = last_worker_msg {
                    let latency = (msg.timestamp.timestamp_millis() 
                        - prev.timestamp.timestamp_millis()) as u32;
                    latencies.push(latency);
                }
            } else {
                last_worker_msg = Some(msg);
            }
        }
        
        if latencies.is_empty() {
            Ok(0)
        } else {
            Ok(latencies.iter().sum::<u32>() / latencies.len() as u32)
        }
    }
    
    async fn calculate_manager_review_quality(&self, manager_id: Uuid) -> Result<f32, LoadError> {
        let recent_reviews = self.db.get_manager_reviews(manager_id, 20).await?;
        
        if recent_reviews.is_empty() {
            return Ok(0.5);
        }
        
        let quality_sum: f32 = recent_reviews.iter()
            .filter_map(|r| r.quality_score)
            .sum();
        
        Ok((quality_sum / recent_reviews.len() as f32).min(1.0))
    }
    
    fn estimate_utilization(
        &self,
        team_size: u32,
        active_tasks: u32,
        pending_reviews: u32,
        decision_latency_ms: u32,
    ) -> f32 {
        // Heuristic: utilization = (active + reviews) / team_size, adjusted for latency
        let base = ((active_tasks + pending_reviews) as f32 / team_size.max(1) as f32).min(1.0);
        let latency_factor = (decision_latency_ms as f32 / 30000.0).min(0.3);  // Caps at 30s
        
        (base + latency_factor).min(1.0)
    }
    
    async fn estimate_optimal_team_size(
        &self,
        _manager_id: Uuid,
        _metrics: &[ManagerWorkloadMetrics],
    ) -> Result<u32, LoadError> {
        Ok(4)  // Would calculate based on actual performance
    }
    
    async fn find_promotion_candidate(&self, _manager_id: Uuid) -> Result<Uuid, LoadError> {
        Err(LoadError::NoSuitableCandidate)
    }
    
    async fn promote_worker_to_manager(&self, _workers: &[Uuid]) -> Result<Uuid, LoadError> {
        Err(LoadError::NoSuitableCandidate)
    }
    
    fn calculate_correlation(&self, x: &[f32], y: &[f32]) -> f32 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        
        let mean_x = x.iter().sum::<f32>() / x.len() as f32;
        let mean_y = y.iter().sum::<f32>() / y.len() as f32;
        
        let numerator: f32 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();
        
        let sum_sq_x: f32 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
        let sum_sq_y: f32 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();
        
        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    DatabaseError(String),
    InsufficientData,
    NoSuitableCandidate,
}
```

### Database Schema for Manager Load Balancing

```sql
CREATE TABLE manager_workload_metrics (
    metrics_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id UUID NOT NULL REFERENCES agents(id),
    measured_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    team_size INT,
    active_tasks INT,
    pending_reviews INT,
    revision_requests INT,
    
    avg_decision_latency_ms INT,
    avg_review_time_minutes DECIMAL(6,2),
    quality_score DECIMAL(3,2),
    
    workload_utilization DECIMAL(3,2),
    burnout_risk VARCHAR(50)
);

CREATE TABLE manager_performance_trends (
    trend_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id UUID NOT NULL REFERENCES agents(id),
    
    measurement_period INT,  -- days
    quality_trend DECIMAL(4,3),
    decision_speed_trend DECIMAL(4,3),
    workload_trend DECIMAL(4,3),
    
    correlation_quality_workload DECIMAL(4,3),
    degradation_rate_per_worker DECIMAL(4,3),
    estimated_optimal_team_size INT,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE team_split_recommendations (
    recommendation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_team_id UUID NOT NULL REFERENCES teams(id),
    original_manager_id UUID NOT NULL REFERENCES agents(id),
    
    action JSONB,
    urgency VARCHAR(50),
    expected_outcome JSONB,
    
    status VARCHAR(50) DEFAULT 'pending',  -- pending, approved, executed
    executed_at TIMESTAMP,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_workload_manager ON manager_workload_metrics(manager_id, measured_at DESC);
CREATE INDEX idx_workload_burnout ON manager_workload_metrics(burnout_risk);
CREATE INDEX idx_trend_manager ON manager_performance_trends(manager_id);
CREATE INDEX idx_split_status ON team_split_recommendations(status);
```

---

## 10. THE "TRUST BOUNDARY" PROBLEM - MANAGER OVERSIGHT

### Problem

Your system trusts manager agents to make good decomposition and assignment decisions. But what if a manager is systematically making poor choices? Tasks fail repeatedly under a certain manager's oversight, but there's no meta-review mechanism.

### Solution: Manager Performance Auditing & Escalation

```rust
// src/manager_governance/manager_oversight.rs

#[derive(Debug, Clone)]
pub struct ManagerPerformanceAudit {
    pub audit_id: Uuid,
    pub manager_id: Uuid,
    pub audit_period: Duration,
    pub audited_at: DateTime<Utc>,
    
    pub decision_quality_metrics: DecisionQualityMetrics,
    pub task_decomposition_quality: f32,
    pub worker_assignment_effectiveness: f32,
    pub review_accuracy: f32,
    
    pub failure_modes: Vec<ManagerFailureMode>,
    pub concerns: Vec<AuditConcern>,
    pub overall_performance_score: f32,
    
    pub recommendation: ManagerOversightRecommendation,
}

#[derive(Debug, Clone)]
pub struct DecisionQualityMetrics {
    pub task_success_rate: f32,        // Of tasks this manager created
    pub revision_count_avg: f32,       // Avg revisions per task
    pub worker_satisfaction: f32,      // Worker feedback
    pub goal_alignment: f32,           // How well tasks align with goal
}

#[derive(Debug, Clone)]
pub struct ManagerFailureMode {
    pub mode_type: FailureModeType,
    pub occurrence_count: u32,
    pub impact: f32,  // 0-1, severity
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FailureModeType {
    PoorWorkerAssignment,  // Wrong agent for task
    Overdecomposition,     // Breaking down too much
    Underdecomposition,    // Not breaking down enough
    BiasedReview,         // Unfair task approvals
    MissedEdgeCases,      // Didn't catch obvious issues
}

#[derive(Debug, Clone)]
pub struct AuditConcern {
    pub concern_id: Uuid,
    pub concern_type: ConcernType,
    pub severity: f32,     // 0-1
    pub description: String,
    pub evidence_task_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub enum ConcernType {
    ConsistentlyLowQuality,
    SkillMismatchPatterns,
    BudgetMismanagement,
    WorkloadDistributionBias,
    DecisionConsistency,
}

#[derive(Debug, Clone)]
pub enum ManagerOversightRecommendation {
    NoAction,
    Monitor,                              // Increased monitoring
    TrainingRequired(Vec<String>),        // Specific skills to improve
    ReducedResponsibility,                // Fewer workers/tasks
    UnderReview { human_reviewer: Uuid }, // Escalate to human
    Disabled,                             // Remove from manager role
}

pub struct ManagerAuditor {
    db: Arc<Database>,
    llm_client: Arc<LlmClient>,
}

impl ManagerAuditor {
    /// Conduct performance audit on a manager
    pub async fn audit_manager_performance(
        &self,
        manager_id: Uuid,
        audit_period: Duration,
    ) -> Result<ManagerPerformanceAudit, AuditError> {
        let manager = self.db.get_agent(manager_id).await?;
        let team = self.db.get_team_by_manager(manager_id).await?;
        let tasks = self.db.get_tasks_created_by_manager(manager_id, audit_period).await?;
        
        // Metric 1: Task Success Rate
        let successful_count = tasks.iter().filter(|t| t.successful).count();
        let success_rate = successful_count as f32 / tasks.len().max(1) as f32;
        
        // Metric 2: Revision Patterns
        let avg_revisions = tasks.iter()
            .map(|t| t.revision_count as f32)
            .sum::<f32>() / tasks.len().max(1) as f32;
        
        // Metric 3: Worker Satisfaction (from feedback)
        let worker_satisfaction = self.assess_worker_sentiment(manager_id).await?;
        
        // Metric 4: Goal Alignment
        let goal = self.db.get_team_goal(team.id).await?;
        let goal_alignment = self.assess_task_goal_alignment(&tasks, &goal).await?;
        
        let metrics = DecisionQualityMetrics {
            task_success_rate: success_rate,
            revision_count_avg: avg_revisions,
            worker_satisfaction,
            goal_alignment,
        };
        
        // Identify failure modes
        let failure_modes = self.identify_failure_modes(manager_id, &tasks).await?;
        
        // Generate audit concerns
        let concerns = self.generate_audit_concerns(&metrics, &failure_modes)?;
        
        // Calculate overall score
        let overall_score = (success_rate * 0.4 
            + (1.0 - (avg_revisions / 5.0).min(1.0)) * 0.2
            + worker_satisfaction * 0.2
            + goal_alignment * 0.2).clamp(0.0, 1.0);
        
        // Make recommendation
        let recommendation = self.make_recommendation(overall_score, &concerns, &failure_modes);
        
        let audit = ManagerPerformanceAudit {
            audit_id: Uuid::new_v4(),
            manager_id,
            audit_period,
            audited_at: Utc::now(),
            decision_quality_metrics: metrics,
            task_decomposition_quality: self.assess_decomposition_quality(&tasks).await?,
            worker_assignment_effectiveness: self.assess_assignment_effectiveness(&tasks).await?,
            review_accuracy: self.assess_review_accuracy(manager_id).await?,
            failure_modes,
            concerns,
            overall_performance_score: overall_score,
            recommendation,
        };
        
        self.db.store_manager_audit(&audit).await?;
        
        Ok(audit)
    }
    
    /// Compare manager to peer benchmarks
    pub async fn benchmark_manager_against_peers(
        &self,
        manager_id: Uuid,
    ) -> Result<ManagerBenchmarkComparison, AuditError> {
        let audit = self.audit_manager_performance(manager_id, Duration::days(30)).await?;
        
        // Get peer managers (same team size category)
        let team = self.db.get_team_by_manager(manager_id).await?;
        let peers = self.db.find_peer_managers(team.members.len()).await?;
        
        let mut peer_scores = Vec::new();
        for peer in peers {
            if let Ok(peer_audit) = self.audit_manager_performance(peer, Duration::days(30)).await {
                peer_scores.push(peer_audit.overall_performance_score);
            }
        }
        
        let mean_peer_score = if !peer_scores.is_empty() {
            peer_scores.iter().sum::<f32>() / peer_scores.len() as f32
        } else {
            0.5
        };
        
        let percentile = if !peer_scores.is_empty() {
            let better_count = peer_scores.iter()
                .filter(|&s| s < audit.overall_performance_score)
                .count();
            (better_count as f32 / peer_scores.len() as f32 * 100.0) as u32
        } else {
            50
        };
        
        let comparison = ManagerBenchmarkComparison {
            manager_id,
            manager_score: audit.overall_performance_score,
            peer_mean: mean_peer_score,
            peer_count: peer_scores.len() as u32,
            percentile_rank: percentile,
            assessment: if audit.overall_performance_score > mean_peer_score * 1.1 {
                "Outperforming".to_string()
            } else if audit.overall_performance_score < mean_peer_score * 0.9 {
                "Underperforming".to_string()
            } else {
                "Aligned with peers".to_string()
            },
        };
        
        self.db.store_benchmark_comparison(&comparison).await?;
        
        Ok(comparison)
    }
    
    /// Escalate manager performance issues to human
    pub async fn escalate_manager_review(
        &self,
        audit: &ManagerPerformanceAudit,
        human_reviewer_id: Uuid,
    ) -> Result<Uuid, AuditError> {
        let escalation = ManagerEscalation {
            escalation_id: Uuid::new_v4(),
            manager_id: audit.manager_id,
            audit_id: audit.audit_id,
            assigned_to: human_reviewer_id,
            status: EscalationStatus::Assigned,
            created_at: Utc::now(),
            concerns_summary: audit.concerns.iter()
                .map(|c| format!("{:?}: {}", c.concern_type, c.description))
                .collect::<Vec<_>>()
                .join("\n"),
            recommended_action: format!("{:?}", audit.recommendation),
        };
        
        self.db.store_manager_escalation(&escalation).await?;
        
        Ok(escalation.escalation_id)
    }
    
    async fn identify_failure_modes(
        &self,
        manager_id: Uuid,
        tasks: &[Task],
    ) -> Result<Vec<ManagerFailureMode>, AuditError> {
        let mut modes = Vec::new();
        
        // Detect poor assignment patterns
        let assignment_mismatches = self.detect_assignment_mismatches(tasks).await?;
        if !assignment_mismatches.is_empty() {
            modes.push(ManagerFailureMode {
                mode_type: FailureModeType::PoorWorkerAssignment,
                occurrence_count: assignment_mismatches.len() as u32,
                impact: 0.6,
                evidence: assignment_mismatches,
            });
        }
        
        // Detect over/under-decomposition
        let decomp_quality = self.analyze_decomposition(tasks).await?;
        if decomp_quality.over_decomposed > 0 {
            modes.push(ManagerFailureMode {
                mode_type: FailureModeType::Overdecomposition,
                occurrence_count: decomp_quality.over_decomposed,
                impact: 0.3,
                evidence: vec![],
            });
        }
        
        if decomp_quality.under_decomposed > 0 {
            modes.push(ManagerFailureMode {
                mode_type: FailureModeType::Underdecomposition,
                occurrence_count: decomp_quality.under_decomposed,
                impact: 0.5,
                evidence: vec![],
            });
        }
        
        // Detect review bias
        let review_bias = self.detect_review_bias(manager_id).await?;
        if review_bias > 0.2 {
            modes.push(ManagerFailureMode {
                mode_type: FailureModeType::BiasedReview,
                occurrence_count: (review_bias * 100.0) as u32,
                impact: review_bias,
                evidence: vec![],
            });
        }
        
        Ok(modes)
    }
    
    async fn detect_assignment_mismatches(&self, tasks: &[Task]) -> Result<Vec<String>, AuditError> {
        let mut mismatches = Vec::new();
        
        for task in tasks.iter().filter(|t| !t.successful) {
            let assigned_agent = if let Some(agent_id) = task.assigned_to.first() {
                self.db.get_agent(*agent_id).await?
            } else {
                continue;
            };
            
            // Check if agent has required skills
            let has_required = task.required_skills.iter()
                .all(|skill| assigned_agent.profile.skills.iter()
                    .any(|a| a.name.to_lowercase().contains(&skill.to_lowercase())));
            
            if !has_required {
                mismatches.push(format!("Task {} assigned to inadequate agent", task.id));
            }
        }
        
        Ok(mismatches)
    }
    
    async fn analyze_decomposition(&self, tasks: &[Task]) -> Result<DecompositionQuality, AuditError> {
        let mut quality = DecompositionQuality { over_decomposed: 0, under_decomposed: 0 };
        
        // Simplified: tasks with > 5 subtasks might be over-decomposed
        for task in tasks {
            let subtask_count = task.subtasks.len();
            if subtask_count > 5 {
                quality.over_decomposed += 1;
            }
        }
        
        Ok(quality)
    }
    
    async fn detect_review_bias(&self, _manager_id: Uuid) -> Result<f32, AuditError> {
        Ok(0.1)  // Would analyze approval patterns
    }
    
    fn generate_audit_concerns(
        &self,
        metrics: &DecisionQualityMetrics,
        failure_modes: &[ManagerFailureMode],
    ) -> Result<Vec<AuditConcern>, AuditError> {
        let mut concerns = Vec::new();
        
        if metrics.task_success_rate < 0.7 {
            concerns.push(AuditConcern {
                concern_id: Uuid::new_v4(),
                concern_type: ConcernType::ConsistentlyLowQuality,
                severity: 1.0 - metrics.task_success_rate,
                description: format!("Task success rate below 70%: {:.0}%", metrics.task_success_rate * 100.0),
                evidence_task_ids: vec![],
            });
        }
        
        if metrics.revision_count_avg > 3.0 {
            concerns.push(AuditConcern {
                concern_id: Uuid::new_v4(),
                concern_type: ConcernType::SkillMismatchPatterns,
                severity: (metrics.revision_count_avg - 1.0) / 4.0,
                description: format!("High revision count: {:.1} avg", metrics.revision_count_avg),
                evidence_task_ids: vec![],
            });
        }
        
        if metrics.worker_satisfaction < 0.6 {
            concerns.push(AuditConcern {
                concern_id: Uuid::new_v4(),
                concern_type: ConcernType::SkillMismatchPatterns,
                severity: 1.0 - metrics.worker_satisfaction,
                description: "Worker satisfaction below expected levels".to_string(),
                evidence_task_ids: vec![],
            });
        }
        
        Ok(concerns)
    }
    
    async fn assess_worker_sentiment(&self, manager_id: Uuid) -> Result<f32, AuditError> {
        let team = self.db.get_team_by_manager(manager_id).await?;
        let mut sentiment_sum = 0.0;
        let mut count = 0;
        
        for worker_id in team.members {
            if let Ok(messages) = self.db.get_worker_comments_about_manager(worker_id, manager_id).await {
                if messages.is_empty() {
                    continue;
                }
                // Would use sentiment analysis
                sentiment_sum += 0.7;
                count += 1;
            }
        }
        
        Ok(if count > 0 { sentiment_sum / count as f32 } else { 0.5 })
    }
    
    async fn assess_task_goal_alignment(&self, _tasks: &[Task], _goal: &Goal) -> Result<f32, AuditError> {
        Ok(0.8)  // Would check each task's relevance to goal
    }
    
    async fn assess_decomposition_quality(&self, tasks: &[Task]) -> Result<f32, AuditError> {
        // Tasks neither over- nor under-decomposed = high quality
        let quality = tasks.iter()
            .filter(|t| t.subtasks.len() > 0 && t.subtasks.len() <= 5)
            .count() as f32 / tasks.len().max(1) as f32;
        
        Ok(quality)
    }
    
    async fn assess_assignment_effectiveness(&self, tasks: &[Task]) -> Result<f32, AuditError> {
        let successful_assignments = tasks.iter()
            .filter(|t| t.successful)
            .count();
        
        Ok(successful_assignments as f32 / tasks.len().max(1) as f32)
    }
    
    async fn assess_review_accuracy(&self, manager_id: Uuid) -> Result<f32, AuditError> {
        let reviews = self.db.get_manager_reviews(manager_id, 50).await?;
        
        let correct_reviews = reviews.iter()
            .filter(|r| r.was_correct_judgment)
            .count();
        
        Ok(correct_reviews as f32 / reviews.len().max(1) as f32)
    }
    
    fn make_recommendation(
        &self,
        score: f32,
        concerns: &[AuditConcern],
        _failure_modes: &[ManagerFailureMode],
    ) -> ManagerOversightRecommendation {
        if score > 0.85 {
            ManagerOversightRecommendation::NoAction
        } else if score > 0.75 {
            ManagerOversightRecommendation::Monitor
        } else if score > 0.65 {
            let training_areas = concerns.iter()
                .map(|c| format!("{:?}", c.concern_type))
                .collect();
            ManagerOversightRecommendation::TrainingRequired(training_areas)
        } else if score > 0.5 {
            ManagerOversightRecommendation::ReducedResponsibility
        } else {
            ManagerOversightRecommendation::Disabled
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManagerBenchmarkComparison {
    pub manager_id: Uuid,
    pub manager_score: f32,
    pub peer_mean: f32,
    pub peer_count: u32,
    pub percentile_rank: u32,
    pub assessment: String,
}

#[derive(Debug, Clone)]
pub struct ManagerEscalation {
    pub escalation_id: Uuid,
    pub manager_id: Uuid,
    pub audit_id: Uuid,
    pub assigned_to: Uuid,
    pub status: EscalationStatus,
    pub created_at: DateTime<Utc>,
    pub concerns_summary: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone)]
pub enum EscalationStatus {
    Assigned,
    InReview,
    Resolved,
    Dismissed,
}

struct DecompositionQuality {
    over_decomposed: u32,
    under_decomposed: u32,
}

#[derive(Debug)]
pub enum AuditError {
    DatabaseError(String),
    AnalysisError(String),
}
```

### Database Schema for Manager Oversight

```sql
CREATE TABLE manager_performance_audits (
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id UUID NOT NULL REFERENCES agents(id),
    audit_period INT,  -- days
    audited_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    task_success_rate DECIMAL(3,2),
    revision_count_avg DECIMAL(4,2),
    worker_satisfaction DECIMAL(3,2),
    goal_alignment DECIMAL(3,2),
    
    task_decomposition_quality DECIMAL(3,2),
    worker_assignment_effectiveness DECIMAL(3,2),
    review_accuracy DECIMAL(3,2),
    
    failure_modes JSONB,
    concerns JSONB,
    overall_performance_score DECIMAL(3,2),
    
    recommendation VARCHAR(100)
);

CREATE TABLE manager_benchmark_comparisons (
    comparison_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id UUID NOT NULL REFERENCES agents(id),
    
    manager_score DECIMAL(3,2),
    peer_mean DECIMAL(3,2),
    peer_count INT,
    percentile_rank INT,
    assessment VARCHAR(100),
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE manager_escalations (
    escalation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id UUID NOT NULL REFERENCES agents(id),
    audit_id UUID NOT NULL REFERENCES manager_performance_audits(audit_id),
    assigned_to UUID NOT NULL,  -- Human reviewer
    
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP,
    
    concerns_summary TEXT,
    recommended_action TEXT,
    resolution_notes TEXT
);

CREATE INDEX idx_audits_manager ON manager_performance_audits(manager_id, audited_at DESC);
CREATE INDEX idx_audits_score ON manager_performance_audits(overall_performance_score);
CREATE INDEX idx_escalations_status ON manager_escalations(status);
```

---

## 11. SUNK COST BIAS IN REVISION LOOPS

### Problem

When a task enters revision loops, there's no mechanism to detect "we're wasting resources on this." Should you persist or abandon? Managers keep pushing, throwing good money after bad. You track revisions and budget, but not *marginal return curves*—the diminishing returns as revisions accumulate.

### Solution: Marginal Return Analysis & Adaptive Termination

```rust
// src/cost_management/marginal_return_tracking.rs

#[derive(Debug, Clone)]
pub struct RevisionCostAnalysis {
    pub analysis_id: Uuid,
    pub task_id: Uuid,
    pub revision_number: u32,
    pub revision_timestamp: DateTime<Utc>,
    
    pub quality_before: f32,
    pub quality_after: f32,
    pub quality_improvement: f32,
    
    pub cost_of_revision: Decimal,
    pub cumulative_cost: Decimal,
    pub cost_per_quality_point: Decimal,
    
    pub marginal_return: MarginalReturn,
    pub recommendation: RevisionRecommendation,
}

#[derive(Debug, Clone)]
pub struct MarginalReturn {
    pub this_revision_roi: f32,      // Quality gain / cost
    pub trend: MarginalTrend,
    pub predicted_next_roi: f32,     // Estimated ROI if we continue
    pub confidence: f32,              // How confident about prediction
}

#[derive(Debug, Clone)]
pub enum MarginalTrend {
    Improving,   // Each revision gets better return
    Stable,      // Consistent return
    Diminishing, // Returns getting worse
    Collapsed,   // Essentially zero return
}

#[derive(Debug, Clone)]
pub enum RevisionRecommendation {
    ContinueRevisions,
    ConsiderAborting,
    StronglyAbort,
    Abandon,
}

pub struct MarginalReturnAnalyzer {
    db: Arc<Database>,
}

impl MarginalReturnAnalyzer {
    /// Analyze cost-effectiveness of revision
    pub async fn analyze_revision_return(
        &self,
        task_id: Uuid,
    ) -> Result<RevisionCostAnalysis, ReturnError> {
        let task = self.db.get_task(task_id).await?;
        let revision_history = self.db.get_task_revision_history(task_id).await?;
        
        if revision_history.is_empty() {
            return Err(ReturnError::NoRevisionHistory);
        }
        
        let current_revision = revision_history.len() as u32;
        let last_revision = revision_history.last().unwrap();
        
        let quality_improvement = last_revision.quality_after - last_revision.quality_before;
        let this_cost = last_revision.cost;
        
        // Calculate cumulative cost
        let cumulative_cost: Decimal = revision_history.iter().map(|r| r.cost).sum();
        
        // Cost per quality point gained (this revision)
        let cost_per_quality = if quality_improvement > 0.001 {
            this_cost / Decimal::from_f32_retain(quality_improvement).unwrap_or(Decimal::ONE)
        } else {
            Decimal::MAX
        };
        
        // Calculate marginal trend
        let trend = self.assess_marginal_trend(&revision_history)?;
        
        // Predict next revision's return
        let predicted_next_roi = self.predict_next_revision_roi(&revision_history)?;
        
        let marginal = MarginalReturn {
            this_revision_roi: if this_cost > Decimal::ZERO {
                (quality_improvement / this_cost.to_f32().unwrap_or(1.0)).max(0.0)
            } else {
                f32::INFINITY
            },
            trend,
            predicted_next_roi,
            confidence: self.calculate_prediction_confidence(&revision_history),
        };
        
        // Make recommendation
        let recommendation = self.recommend_revision_action(&marginal, cumulative_cost, &task)?;
        
        let analysis = RevisionCostAnalysis {
            analysis_id: Uuid::new_v4(),
            task_id,
            revision_number: current_revision,
            revision_timestamp: Utc::now(),
            quality_before: last_revision.quality_before,
            quality_after: last_revision.quality_after,
            quality_improvement,
            cost_of_revision: this_cost,
            cumulative_cost,
            cost_per_quality_point: cost_per_quality,
            marginal_return: marginal,
            recommendation,
        };
        
        self.db.store_revision_analysis(&analysis).await?;
        
        Ok(analysis)
    }
    
    /// Should we continue revising or cut losses?
    pub async fn should_continue_revisions(
        &self,
        task_id: Uuid,
        remaining_budget: Decimal,
    ) -> Result<SunkCostDecision, ReturnError> {
        let analysis = self.analyze_revision_return(task_id).await?;
        let task = self.db.get_task(task_id).await?;
        
        match analysis.recommendation {
            RevisionRecommendation::Abandon => {
                return Ok(SunkCostDecision {
                    task_id,
                    decision: RevisionDecision::Abort,
                    reasoning: "Marginal returns collapsed; no budget justifies continuing".to_string(),
                    cost_if_continue: Decimal::ZERO,
                    sunk_cost_already: analysis.cumulative_cost,
                });
            }
            _ => {}
        }
        
        // Estimate cost to reach acceptable quality
        let acceptable_quality = task.acceptance_criteria.len() as f32 / 10.0;
        let current_quality = task.quality_score.unwrap_or(0.0);
        
        if current_quality >= acceptable_quality {
            return Ok(SunkCostDecision {
                task_id,
                decision: RevisionDecision::Approve,
                reasoning: "Acceptable quality threshold met".to_string(),
                cost_if_continue: Decimal::ZERO,
                sunk_cost_already: analysis.cumulative_cost,
            });
        }
        
        // Estimate continuing
        let estimated_revisions_needed = self.estimate_revisions_to_acceptable(
            &analysis,
            acceptable_quality,
            current_quality,
        )?;
        
        let estimated_additional_cost = analysis.cost_of_revision * Decimal::from(estimated_revisions_needed);
        
        if estimated_additional_cost > remaining_budget {
            return Ok(SunkCostDecision {
                task_id,
                decision: RevisionDecision::Abort,
                reasoning: format!(
                    "Estimated ${:.2} needed but only ${:.2} budget remains",
                    estimated_additional_cost, remaining_budget
                ),
                cost_if_continue: estimated_additional_cost,
                sunk_cost_already: analysis.cumulative_cost,
            });
        }
        
        match analysis.marginal_return.trend {
            MarginalTrend::Diminishing if analysis.marginal_return.predicted_next_roi < 0.1 => {
                Ok(SunkCostDecision {
                    task_id,
                    decision: RevisionDecision::ConsiderAbort,
                    reasoning: "Diminishing returns; consider alternative approaches".to_string(),
                    cost_if_continue: estimated_additional_cost,
                    sunk_cost_already: analysis.cumulative_cost,
                })
            }
            MarginalTrend::Collapsed => {
                Ok(SunkCostDecision {
                    task_id,
                    decision: RevisionDecision::Abort,
                    reasoning: "Marginal returns have collapsed".to_string(),
                    cost_if_continue: estimated_additional_cost,
                    sunk_cost_already: analysis.cumulative_cost,
                })
            }
            _ => {
                Ok(SunkCostDecision {
                    task_id,
                    decision: RevisionDecision::Continue,
                    reasoning: format!(
                        "Positive ROI predicted ({:.2}); estimated ${:.2} to meet standards",
                        analysis.marginal_return.predicted_next_roi,
                        estimated_additional_cost
                    ),
                    cost_if_continue: estimated_additional_cost,
                    sunk_cost_already: analysis.cumulative_cost,
                })
            }
        }
    }
    
    fn assess_marginal_trend(
        &self,
        history: &[RevisionRecord],
    ) -> Result<MarginalTrend, ReturnError> {
        if history.len() < 2 {
            return Ok(MarginalTrend::Stable);
        }
        
        let mut improvements = Vec::new();
        for window in history.windows(2) {
            let prev_improvement = window[0].quality_after - window[0].quality_before;
            let curr_improvement = window[1].quality_after - window[1].quality_before;
            improvements.push(curr_improvement / prev_improvement.max(0.001));
        }
        
        if improvements.is_empty() {
            return Ok(MarginalTrend::Stable);
        }
        
        let avg_ratio = improvements.iter().sum::<f32>() / improvements.len() as f32;
        
        if avg_ratio < 0.3 {
            Ok(MarginalTrend::Collapsed)
        } else if avg_ratio < 0.7 {
            Ok(MarginalTrend::Diminishing)
        } else if avg_ratio > 1.2 {
            Ok(MarginalTrend::Improving)
        } else {
            Ok(MarginalTrend::Stable)
        }
    }
    
    fn predict_next_revision_roi(
        &self,
        history: &[RevisionRecord],
    ) -> Result<f32, ReturnError> {
        if history.len() < 2 {
            return Ok(0.5);
        }
        
        let last = history.last().unwrap();
        let prev = &history[history.len() - 2];
        
        let this_roi = if last.cost > Decimal::ZERO {
            ((last.quality_after - last.quality_before) 
                / last.cost.to_f32().unwrap_or(1.0)).max(0.0)
        } else {
            0.0
        };
        
        let prev_roi = if prev.cost > Decimal::ZERO {
            ((prev.quality_after - prev.quality_before) 
                / prev.cost.to_f32().unwrap_or(1.0)).max(0.0)
        } else {
            0.0
        };
        
        // Project trend
        let roi_ratio = if prev_roi > 0.001 { this_roi / prev_roi } else { 1.0 };
        Ok((this_roi * roi_ratio).clamp(0.0, 1.0))
    }
    
    fn calculate_prediction_confidence(&self, history: &[RevisionRecord]) -> f32 {
        // Confidence increases with more revision data
        ((history.len() as f32 / 10.0).min(1.0) * 0.8) + 0.2
    }
    
    fn recommend_revision_action(
        &self,
        marginal: &MarginalReturn,
        cumulative_cost: Decimal,
        task: &Task,
    ) -> Result<RevisionRecommendation, ReturnError> {
        match marginal.trend {
            MarginalTrend::Collapsed => {
                Ok(RevisionRecommendation::Abandon)
            }
            MarginalTrend::Diminishing if marginal.predicted_next_roi < 0.05 => {
                Ok(RevisionRecommendation::StronglyAbort)
            }
            MarginalTrend::Diminishing if marginal.predicted_next_roi < 0.15 => {
                Ok(RevisionRecommendation::ConsiderAborting)
            }
            _ => {
                if cumulative_cost > Decimal::from(100) {
                    Ok(RevisionRecommendation::ConsiderAborting)
                } else {
                    Ok(RevisionRecommendation::ContinueRevisions)
                }
            }
        }
    }
    
    fn estimate_revisions_to_acceptable(
        &self,
        analysis: &RevisionCostAnalysis,
        target_quality: f32,
        current_quality: f32,
    ) -> Result<u32, ReturnError> {
        let quality_gap = target_quality - current_quality;
        if quality_gap <= 0.0 {
            return Ok(0);
        }
        
        let avg_quality_per_revision = analysis.quality_improvement;
        if avg_quality_per_revision < 0.001 {
            return Ok(100);  // Essentially infinite
        }
        
        Ok(((quality_gap / avg_quality_per_revision).ceil() as u32).min(100))
    }
}

#[derive(Debug, Clone)]
pub struct RevisionRecord {
    pub revision_number: u32,
    pub quality_before: f32,
    pub quality_after: f32,
    pub cost: Decimal,
}

#[derive(Debug, Clone)]
pub struct SunkCostDecision {
    pub task_id: Uuid,
    pub decision: RevisionDecision,
    pub reasoning: String,
    pub cost_if_continue: Decimal,
    pub sunk_cost_already: Decimal,
}

#[derive(Debug, Clone)]
pub enum RevisionDecision {
    Continue,       // Revisions should continue
    ConsiderAbort,  // Maybe stop, but budget allows continuing
    Abort,         // Stop here; diminishing returns
    Approve,       // Quality acceptable; stop and approve
}

#[derive(Debug)]
pub enum ReturnError {
    DatabaseError(String),
    NoRevisionHistory,
    AnalysisError,
}
```

### Database Schema for Marginal Return Tracking

```sql
CREATE TABLE revision_cost_analyses (
    analysis_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    revision_number INT,
    revision_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    quality_before DECIMAL(3,2),
    quality_after DECIMAL(3,2),
    quality_improvement DECIMAL(3,2),
    
    cost_of_revision DECIMAL(10,4),
    cumulative_cost DECIMAL(10,4),
    cost_per_quality_point DECIMAL(10,4),
    
    marginal_return_roi DECIMAL(4,3),
    marginal_trend VARCHAR(50),
    predicted_next_roi DECIMAL(4,3),
    roi_confidence DECIMAL(3,2),
    
    recommendation VARCHAR(50)
);

CREATE TABLE sunk_cost_decisions (
    decision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    
    decision VARCHAR(50),  -- continue, consider_abort, abort, approve
    reasoning TEXT,
    cost_if_continue DECIMAL(10,4),
    sunk_cost_already DECIMAL(10,4),
    
    made_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    decision_accepted BOOLEAN
);

CREATE TABLE revision_history_detail (
    detail_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    revision_number INT,
    
    quality_before DECIMAL(3,2),
    quality_after DECIMAL(3,2),
    cost DECIMAL(10,4),
    duration_minutes INT,
    agent_id UUID REFERENCES agents(id),
    
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_analyses_task ON revision_cost_analyses(task_id, revision_number DESC);
CREATE INDEX idx_analyses_trend ON revision_cost_analyses(marginal_trend);
CREATE INDEX idx_decisions_task ON sunk_cost_decisions(task_id);
CREATE INDEX idx_decisions_made ON sunk_cost_decisions(made_at DESC);
```

---

## Integration & Deployment Checklist

All 11 solutions are now fully specified. Integration points:

### Phase 1: Foundation (Week 1-2)
- [ ] Deploy goal validation at team creation
- [ ] Enable context lifecycle management
- [ ] Activate failure classification

### Phase 2: Optimization (Week 3-4)
- [ ] Launch emergence tracking
- [ ] Enable skill acquisition system
- [ ] Activate capability regression detection

### Phase 3: Governance (Week 5-6)
- [ ] Deploy manager load monitoring
- [ ] Enable manager auditing
- [ ] Activate sunk cost detection

### Phase 4: Learning (Week 7-8)
- [ ] Connect preference aggregation
- [ ] Enable workflow capture
- [ ] Wire up recommendation engine

### Monitoring & Metrics
Each system should emit:
- **Cost savings**: Sunk cost prevention, context compression
- **Quality improvements**: Regression detection, manager oversight
- **Efficiency gains**: Skill acquisition, emergence patterns
- **Risk reduction**: Failure classification, goal validation

---

## Summary

These 11 second-order solutions transform an isolated team-based multi-agent system from "good enough" to production-grade:

**Self-Optimization**: Emergence tracking + preference learning create feedback loops where teams improve each other
**Risk Mitigation**: Goal validation + failure classification + manager oversight prevent cascading failures
**Resource Management**: Context compression + marginal return tracking + skill acquisition optimize finite resources
**Learning**: Workflow capture + capability trajectories + manager auditing enable continuous improvement

The architecture now supports autonomous team missions with built-in quality control, cost management, and adaptive learning.
