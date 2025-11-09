---
description: Start working on a specific sprint of the Ghost Pirates implementation
tags: [planning, implementation]
---

# Start Sprint Implementation

You are about to start implementing a sprint from the Ghost Pirates development plan.

## Instructions

1. **Read the sprint file** from `/docs/sprints/` based on the sprint number provided by the user
2. **Review all user stories and tasks** in that sprint
3. **Create a todo list** using the TodoWrite tool with all major tasks from the sprint
4. **Ask the user** if they want to proceed with the first task or if they have any questions
5. **Follow the sprint plan** exactly as written, executing each task with proper domain-driven design patterns

## Sprint Files

- Sprint 1: `sprint-1-foundation.md` - ✅ COMPLETE (Foundation infrastructure)
- Sprint 2: `sprint-2-agent-system.md` - Agent architecture & LLM integration
- Sprint 3+: Future sprints (to be defined)

## Example Usage

User: `/start-sprint 2`
Claude: _Reads Sprint 2 file, creates todo list, starts implementing agent system_

## Important Notes

- Each sprint builds on the previous one
- Complete each task's acceptance criteria before moving to the next
- Follow hexagonal architecture (domain → repository → API)
- Run tests frequently (cargo test, cargo clippy, cargo fmt)
- Mark tasks as completed in the todo list as you go
- Sprint 1 is already complete - all tests passing

## Sprint 2 Focus

Agent System Architecture:
- Agent domain models (capabilities, roles, state)
- LLM integration (Claude API)
- Tool execution framework
- Agent memory and context management
- Autonomous task execution
- Inter-agent communication

See `/docs/sprints/sprint-2-agent-system.md` for full details.
