// Shared types for Ghost Pirates

export enum TeamStatus {
  Pending = "pending",
  Planning = "planning",
  Active = "active",
  Completed = "completed",
  Failed = "failed",
  Archived = "archived",
}

export enum TaskStatus {
  Pending = "pending",
  Assigned = "assigned",
  InProgress = "in_progress",
  Review = "review",
  Completed = "completed",
  Failed = "failed",
  RevisionRequested = "revision_requested",
}

export enum MemberRole {
  Manager = "manager",
  Worker = "worker",
}

export interface Team {
  id: string;
  companyId: string;
  goal: string;
  status: TeamStatus;
  managerAgentId?: string;
  createdBy: string;
  createdAt: Date;
  startedAt?: Date;
  completedAt?: Date;
  budgetLimit?: number;
  metadata: Record<string, unknown>;
}

export interface Task {
  id: string;
  teamId: string;
  parentTaskId?: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  assignedTo?: string;
  assignedBy?: string;
  status: TaskStatus;
  startTime?: Date;
  completionTime?: Date;
  revisionCount: number;
  maxRevisions: number;
  inputData?: Record<string, unknown>;
  outputData?: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export interface User {
  id: string;
  companyId: string;
  email: string;
  fullName: string;
  isActive: boolean;
  lastLogin?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface TeamMember {
  id: string;
  teamId: string;
  agentId: string;
  role: MemberRole;
  specialization?: string;
  status: string;
  currentWorkload: number;
  maxConcurrentTasks: number;
  joinedAt: Date;
}

export interface CostTracking {
  id: string;
  teamId: string;
  taskId?: string;
  modelName: string;
  inputTokens: number;
  outputTokens: number;
  totalCost: number;
  createdAt: Date;
}

// API Request/Response types
export interface CreateTeamRequest {
  goal: string;
  budgetLimit?: number;
}

export interface CreateTeamResponse {
  teamId: string;
  status: TeamStatus;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: Omit<User, "passwordHash">;
}

export interface RegisterRequest {
  email: string;
  password: string;
  fullName: string;
  companyId: string;
}

export interface RegisterResponse {
  userId: string;
}
