// Prompt templates for LLM interactions
//
// This module contains all prompt templates used by the agent system.
// Prompts are versioned for reproducibility and A/B testing.

/// Prompt template structure
pub struct PromptTemplate {
    pub name: String,
    pub version: String,
    pub system: String,
    pub user_template: String,
}

impl PromptTemplate {
    /// Render the user template with variables
    pub fn render(&self, _variables: &std::collections::HashMap<String, String>) -> String {
        // TODO: Implement variable substitution (US-303.12)
        self.user_template.clone()
    }
}

// Prompt library will be populated in US-303.14
pub mod library {
    use super::PromptTemplate;

    pub fn goal_analysis() -> PromptTemplate {
        PromptTemplate {
            name: "goal_analysis".to_string(),
            version: "1.0.0".to_string(),
            system: "You are a highly skilled project manager analyzing project goals. \
                     Analyze the following goal and provide structured output in JSON format."
                .to_string(),
            user_template: "Goal: {{goal}}\n\n\
                            Provide:\n\
                            1. Core objective (one sentence)\n\
                            2. Key subtasks (ordered list)\n\
                            3. Required specializations (types of workers needed)\n\
                            4. Estimated timeline (hours)\n\
                            5. Potential blockers\n\
                            6. Success criteria"
                .to_string(),
        }
    }

    pub fn team_formation() -> PromptTemplate {
        PromptTemplate {
            name: "team_formation".to_string(),
            version: "1.0.0".to_string(),
            system: "You are forming a team of specialized AI agents. \
                     Create 3-5 worker specifications in JSON format."
                .to_string(),
            user_template: "Goal: {{goal}}\n\
                            Subtasks: {{subtasks}}\n\n\
                            Create 3-5 specialized workers. For each:\n\
                            - Role name and specialization\n\
                            - Key skills required\n\
                            - Primary responsibilities\n\
                            - Tools they'll need"
                .to_string(),
        }
    }

    pub fn task_decomposition() -> PromptTemplate {
        PromptTemplate {
            name: "task_decomposition".to_string(),
            version: "1.0.0".to_string(),
            system: "You are breaking down a goal into concrete, actionable tasks."
                .to_string(),
            user_template: "Goal: {{goal}}\n\n\
                            For each task provide:\n\
                            - Title\n\
                            - Detailed description\n\
                            - Acceptance criteria (3-5 checkable items)\n\
                            - Required skills\n\
                            - Estimated tokens/complexity"
                .to_string(),
        }
    }
}
