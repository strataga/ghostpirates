# Phase 5: Frontend Basics

**Duration**: Weeks 9-10 (14 days)
**Goal**: Next.js App → Team Creation Wizard → Dashboard Layout → Real-time Updates
**Dependencies**: Phase 4 complete (Backend API fully operational)

---

## Epic 1: Next.js App Setup with App Router

### Task 1.1: Create Next.js Project Structure

**Type**: Frontend
**Dependencies**: Node.js 18+, npm/pnpm

**Subtasks**:

- [ ] 1.1.1: Initialize Next.js 14 project with App Router

```bash
cd /Users/jason/projects/ghostpirates
npx create-next-app@latest apps/web --typescript --tailwind --app --src-dir --import-alias "@/*"
cd apps/web
```

- [ ] 1.1.2: Install required dependencies

```bash
npm install @tanstack/react-query zustand
npm install axios
npm install react-hook-form zod @hookform/resolvers
npm install date-fns
npm install lucide-react
npm install @radix-ui/react-dialog @radix-ui/react-dropdown-menu @radix-ui/react-tabs
npm install @radix-ui/react-progress @radix-ui/react-tooltip @radix-ui/react-select
npm install clsx tailwind-merge
npm install --save-dev @types/node
```

- [ ] 1.1.3: Configure project structure

```bash
mkdir -p src/app/(dashboard)
mkdir -p src/components/{ui,teams,tasks,layout}
mkdir -p src/lib/{api,hooks,store,utils}
mkdir -p src/types
touch src/lib/utils.ts
```

- [ ] 1.1.4: Create utility functions

```typescript
// src/lib/utils.ts
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 4,
  }).format(amount);
}

export function formatDate(date: Date | string): string {
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(date));
}

export function formatRelativeTime(date: Date | string): string {
  const now = new Date();
  const then = new Date(date);
  const diffMs = now.getTime() - then.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 7) return `${diffDays}d ago`;

  return formatDate(date);
}
```

- [ ] 1.1.5: Create TypeScript types

```typescript
// src/types/team.ts
export enum TeamStatus {
  Pending = "pending",
  Planning = "planning",
  Active = "active",
  Paused = "paused",
  Completed = "completed",
  Failed = "failed",
  Archived = "archived",
}

export interface Team {
  id: string;
  company_id: string;
  goal: string;
  status: TeamStatus;
  manager_agent_id: string | null;
  created_by: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  paused_at: string | null;
  budget_limit: number | null;
  actual_cost: number;
  metadata: Record<string, any>;
}

export enum MemberRole {
  Manager = "manager",
  Worker = "worker",
}

export enum MemberStatus {
  Active = "active",
  Idle = "idle",
  Busy = "busy",
  Offline = "offline",
  Failed = "failed",
}

export interface TeamMember {
  id: string;
  team_id: string;
  agent_id: string;
  role: MemberRole;
  specialization: string | null;
  status: MemberStatus;
  current_workload: number;
  max_concurrent_tasks: number;
  tasks_completed: number;
  tasks_failed: number;
  total_tokens_used: number;
  total_cost: number;
  joined_at: string;
  last_active_at: string | null;
}
```

```typescript
// src/types/task.ts
export enum TaskStatus {
  Pending = "pending",
  Assigned = "assigned",
  InProgress = "in_progress",
  Review = "review",
  Completed = "completed",
  Failed = "failed",
  RevisionRequested = "revision_requested",
  Blocked = "blocked",
}

export interface Task {
  id: string;
  team_id: string;
  parent_task_id: string | null;
  title: string;
  description: string;
  acceptance_criteria: string[];
  assigned_to: string | null;
  assigned_by: string | null;
  status: TaskStatus;
  priority: number;
  start_time: string | null;
  completion_time: string | null;
  revision_count: number;
  max_revisions: number;
  input_data: Record<string, any> | null;
  output_data: Record<string, any> | null;
  error_message: string | null;
  required_skills: string[];
  estimated_tokens: number | null;
  actual_tokens: number | null;
  created_at: string;
  updated_at: string;
}
```

- [ ] 1.1.6: Configure Tailwind CSS theme

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        success: {
          DEFAULT: "hsl(142, 76%, 36%)",
          foreground: "hsl(0, 0%, 100%)",
        },
        warning: {
          DEFAULT: "hsl(38, 92%, 50%)",
          foreground: "hsl(0, 0%, 100%)",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      fontFamily: {
        sans: ["var(--font-inter)"],
        mono: ["var(--font-mono)"],
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
```

- [ ] 1.1.7: Add CSS variables

```css
/* src/app/globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 222.2 84% 4.9%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 84% 4.9%;
    --popover-foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
```

**Acceptance Criteria**:

- [ ] Next.js 14 project created
- [ ] All dependencies installed
- [ ] Project structure created
- [ ] TypeScript types defined
- [ ] Tailwind CSS configured
- [ ] Utility functions working
- [ ] Dev server starts without errors

---

## Epic 2: Team Creation Wizard (Multi-step Form)

### Task 2.1: Create Team Creation Form

**Type**: Frontend
**Dependencies**: React Hook Form, Zod

**Subtasks**:

- [ ] 2.1.1: Create form validation schema

```typescript
// src/lib/validations/team.ts
import { z } from "zod";

export const createTeamSchema = z.object({
  goal: z
    .string()
    .min(10, "Goal must be at least 10 characters")
    .max(2000, "Goal must be less than 2000 characters"),
  budget_limit: z
    .number()
    .positive("Budget must be positive")
    .max(10000, "Budget cannot exceed $10,000")
    .optional()
    .nullable(),
  max_depth: z
    .number()
    .int()
    .min(1)
    .max(5)
    .default(3)
    .optional(),
});

export type CreateTeamFormData = z.infer<typeof createTeamSchema>;
```

- [ ] 2.1.2: Create team creation form component

```typescript
// src/components/teams/TeamCreationWizard.tsx
"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useRouter } from "next/navigation";
import { Loader2, Sparkles } from "lucide-react";
import { createTeamSchema, type CreateTeamFormData } from "@/lib/validations/team";
import { useCreateTeam } from "@/lib/hooks/useTeams";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { Label } from "@/components/ui/Label";

export function TeamCreationWizard() {
  const router = useRouter();
  const [step, setStep] = useState(1);
  const createTeam = useCreateTeam();

  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    watch,
  } = useForm<CreateTeamFormData>({
    resolver: zodResolver(createTeamSchema),
    defaultValues: {
      goal: "",
      budget_limit: null,
      max_depth: 3,
    },
  });

  const goal = watch("goal");

  const onSubmit = async (data: CreateTeamFormData) => {
    try {
      const team = await createTeam.mutateAsync(data);
      router.push(`/dashboard/teams/${team.id}`);
    } catch (error) {
      console.error("Failed to create team:", error);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-2 mb-2">
          <Sparkles className="w-6 h-6 text-primary" />
          <h1 className="text-3xl font-bold">Create AI Team</h1>
        </div>
        <p className="text-muted-foreground">
          Describe your goal and Ghost Pirates will assemble a specialized team to complete it.
        </p>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Step 1: Goal */}
        <div className="space-y-4">
          <div>
            <Label htmlFor="goal">What would you like to accomplish?</Label>
            <Textarea
              id="goal"
              placeholder="Example: Create a comprehensive market analysis report for the electric vehicle industry in North America, including competitor analysis, market trends, and growth projections for the next 5 years."
              rows={6}
              className="mt-1"
              {...register("goal")}
            />
            {errors.goal && (
              <p className="text-sm text-destructive mt-1">{errors.goal.message}</p>
            )}
            <p className="text-sm text-muted-foreground mt-2">
              {goal.length} / 2000 characters
            </p>
          </div>

          {/* Step 2: Optional Settings */}
          {step >= 2 && (
            <>
              <div>
                <Label htmlFor="budget_limit">Budget Limit (optional)</Label>
                <div className="flex items-center gap-2 mt-1">
                  <span className="text-muted-foreground">$</span>
                  <Input
                    id="budget_limit"
                    type="number"
                    step="0.01"
                    placeholder="100.00"
                    {...register("budget_limit", { valueAsNumber: true })}
                  />
                </div>
                {errors.budget_limit && (
                  <p className="text-sm text-destructive mt-1">
                    {errors.budget_limit.message}
                  </p>
                )}
                <p className="text-sm text-muted-foreground mt-2">
                  Team will pause if budget is exceeded. Leave empty for no limit.
                </p>
              </div>

              <div>
                <Label htmlFor="max_depth">Task Decomposition Depth</Label>
                <Input
                  id="max_depth"
                  type="number"
                  min="1"
                  max="5"
                  className="mt-1"
                  {...register("max_depth", { valueAsNumber: true })}
                />
                {errors.max_depth && (
                  <p className="text-sm text-destructive mt-1">
                    {errors.max_depth.message}
                  </p>
                )}
                <p className="text-sm text-muted-foreground mt-2">
                  How many levels of subtasks to create (1-5). Higher = more granular tasks.
                </p>
              </div>
            </>
          )}
        </div>

        {/* Navigation Buttons */}
        <div className="flex justify-between pt-4">
          {step === 1 ? (
            <div></div>
          ) : (
            <Button
              type="button"
              variant="outline"
              onClick={() => setStep(step - 1)}
              disabled={isSubmitting}
            >
              Back
            </Button>
          )}

          {step === 1 ? (
            <Button
              type="button"
              onClick={() => setStep(2)}
              disabled={!goal || goal.length < 10}
            >
              Continue
            </Button>
          ) : (
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating Team...
                </>
              ) : (
                "Create Team"
              )}
            </Button>
          )}
        </div>
      </form>

      {/* Progress Indicator */}
      <div className="flex justify-center gap-2 mt-8">
        {[1, 2].map((s) => (
          <div
            key={s}
            className={`h-2 w-16 rounded-full transition-colors ${
              s <= step ? "bg-primary" : "bg-muted"
            }`}
          />
        ))}
      </div>
    </div>
  );
}
```

- [ ] 2.1.3: Create reusable UI components

```typescript
// src/components/ui/Button.tsx
import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline: "border border-input hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "underline-offset-4 hover:underline text-primary",
      },
      size: {
        default: "h-10 py-2 px-4",
        sm: "h-9 px-3 rounded-md",
        lg: "h-11 px-8 rounded-md",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
```

```typescript
// src/components/ui/Input.tsx
import * as React from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";

export { Input };
```

```typescript
// src/components/ui/Textarea.tsx
import * as React from "react";
import { cn } from "@/lib/utils";

export interface TextareaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {}

const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        className={cn(
          "flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
Textarea.displayName = "Textarea";

export { Textarea };
```

```typescript
// src/components/ui/Label.tsx
import * as React from "react";
import { cn } from "@/lib/utils";

export interface LabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {}

const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  ({ className, ...props }, ref) => {
    return (
      <label
        ref={ref}
        className={cn(
          "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
          className
        )}
        {...props}
      />
    );
  }
);
Label.displayName = "Label";

export { Label };
```

**Acceptance Criteria**:

- [ ] Form validation with Zod working
- [ ] Multi-step wizard functional
- [ ] Character counter displayed
- [ ] Budget input accepts decimals
- [ ] Progress indicator updates
- [ ] Form submission triggers API call
- [ ] Error messages displayed correctly

---

## Epic 3: Dashboard Layout with Sidebar

### Task 3.1: Create Dashboard Layout

**Type**: Frontend
**Dependencies**: Epic 1 complete

**Subtasks**:

- [ ] 3.1.1: Create dashboard layout component

```typescript
// src/app/(dashboard)/layout.tsx
import { Sidebar } from "@/components/layout/Sidebar";
import { Header } from "@/components/layout/Header";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-y-auto p-6">{children}</main>
      </div>
    </div>
  );
}
```

- [ ] 3.1.2: Create sidebar component

```typescript
// src/components/layout/Sidebar.tsx
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Users,
  ListTodo,
  Settings,
  Sparkles,
  DollarSign,
} from "lucide-react";
import { cn } from "@/lib/utils";

const navigation = [
  { name: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
  { name: "Teams", href: "/dashboard/teams", icon: Users },
  { name: "Tasks", href: "/dashboard/tasks", icon: ListTodo },
  { name: "Billing", href: "/dashboard/billing", icon: DollarSign },
  { name: "Settings", href: "/dashboard/settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <div className="w-64 bg-card border-r border-border flex flex-col">
      {/* Logo */}
      <div className="h-16 flex items-center px-6 border-b border-border">
        <Sparkles className="w-6 h-6 text-primary mr-2" />
        <span className="text-xl font-bold">Ghost Pirates</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-1">
        {navigation.map((item) => {
          const isActive = pathname === item.href || pathname.startsWith(`${item.href}/`);
          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              <item.icon className="w-5 h-5" />
              {item.name}
            </Link>
          );
        })}
      </nav>

      {/* Create Team Button */}
      <div className="p-4 border-t border-border">
        <Link
          href="/dashboard/teams/new"
          className="flex items-center justify-center gap-2 w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
        >
          <Sparkles className="w-4 h-4" />
          Create Team
        </Link>
      </div>
    </div>
  );
}
```

- [ ] 3.1.3: Create header component

```typescript
// src/components/layout/Header.tsx
"use client";

import { Bell, Search, User } from "lucide-react";
import { Button } from "@/components/ui/Button";

export function Header() {
  return (
    <header className="h-16 border-b border-border bg-card px-6 flex items-center justify-between">
      {/* Search */}
      <div className="flex-1 max-w-xl">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search teams, tasks..."
            className="w-full pl-10 pr-4 py-2 bg-background border border-input rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
      </div>

      {/* Right Section */}
      <div className="flex items-center gap-4">
        {/* Notifications */}
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="w-5 h-5" />
          <span className="absolute top-1 right-1 w-2 h-2 bg-destructive rounded-full" />
        </Button>

        {/* User Menu */}
        <Button variant="ghost" size="icon">
          <User className="w-5 h-5" />
        </Button>
      </div>
    </header>
  );
}
```

- [ ] 3.1.4: Create dashboard home page

```typescript
// src/app/(dashboard)/dashboard/page.tsx
import { StatsCards } from "@/components/dashboard/StatsCards";
import { RecentTeams } from "@/components/dashboard/RecentTeams";
import { ActivityFeed } from "@/components/dashboard/ActivityFeed";

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Overview of your AI teams and their progress
        </p>
      </div>

      <StatsCards />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <RecentTeams />
        <ActivityFeed />
      </div>
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] Sidebar renders correctly
- [ ] Active nav item highlighted
- [ ] Header with search bar functional
- [ ] Dashboard layout responsive
- [ ] Create Team button accessible
- [ ] Navigation working between pages

---

## Epic 4: Team List Component with React Query

### Task 4.1: Implement API Client and React Query Hooks

**Type**: Frontend
**Dependencies**: React Query, Axios

**Subtasks**:

- [ ] 4.1.1: Create API client

```typescript
// src/lib/api/client.ts
import axios from "axios";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000";

export const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

// Add auth token to requests
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem("auth_token");
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle errors
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Redirect to login
      window.location.href = "/auth/login";
    }
    return Promise.reject(error);
  }
);
```

- [ ] 4.1.2: Create teams API functions

```typescript
// src/lib/api/teams.ts
import { apiClient } from "./client";
import type { Team, TeamMember } from "@/types/team";
import type { Task } from "@/types/task";
import type { CreateTeamFormData } from "@/lib/validations/team";

export const teamsApi = {
  getTeams: async (): Promise<Team[]> => {
    const { data } = await apiClient.get("/api/teams");
    return data;
  },

  getTeam: async (id: string): Promise<Team> => {
    const { data } = await apiClient.get(`/api/teams/${id}`);
    return data;
  },

  createTeam: async (teamData: CreateTeamFormData): Promise<Team> => {
    const { data } = await apiClient.post("/api/teams", teamData);
    return data;
  },

  updateTeam: async (id: string, updates: Partial<Team>): Promise<Team> => {
    const { data } = await apiClient.patch(`/api/teams/${id}`, updates);
    return data;
  },

  pauseTeam: async (id: string): Promise<Team> => {
    const { data } = await apiClient.post(`/api/teams/${id}/pause`);
    return data;
  },

  resumeTeam: async (id: string): Promise<Team> => {
    const { data } = await apiClient.post(`/api/teams/${id}/resume`);
    return data;
  },

  getTeamMembers: async (id: string): Promise<TeamMember[]> => {
    const { data } = await apiClient.get(`/api/teams/${id}/members`);
    return data;
  },

  getTeamTasks: async (id: string): Promise<Task[]> => {
    const { data } = await apiClient.get(`/api/teams/${id}/tasks`);
    return data;
  },
};
```

- [ ] 4.1.3: Create React Query hooks

```typescript
// src/lib/hooks/useTeams.ts
import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";
import { teamsApi } from "@/lib/api/teams";
import type { Team, TeamMember } from "@/types/team";
import type { Task } from "@/types/task";
import type { CreateTeamFormData } from "@/lib/validations/team";

// Query keys
export const teamKeys = {
  all: ["teams"] as const,
  lists: () => [...teamKeys.all, "list"] as const,
  list: (filters: string) => [...teamKeys.lists(), { filters }] as const,
  details: () => [...teamKeys.all, "detail"] as const,
  detail: (id: string) => [...teamKeys.details(), id] as const,
  members: (id: string) => [...teamKeys.detail(id), "members"] as const,
  tasks: (id: string) => [...teamKeys.detail(id), "tasks"] as const,
};

// Get all teams
export function useTeams() {
  return useQuery({
    queryKey: teamKeys.lists(),
    queryFn: teamsApi.getTeams,
  });
}

// Get single team
export function useTeam(id: string, options?: UseQueryOptions<Team>) {
  return useQuery({
    queryKey: teamKeys.detail(id),
    queryFn: () => teamsApi.getTeam(id),
    enabled: !!id,
    ...options,
  });
}

// Get team members
export function useTeamMembers(id: string) {
  return useQuery({
    queryKey: teamKeys.members(id),
    queryFn: () => teamsApi.getTeamMembers(id),
    enabled: !!id,
  });
}

// Get team tasks
export function useTeamTasks(id: string) {
  return useQuery({
    queryKey: teamKeys.tasks(id),
    queryFn: () => teamsApi.getTeamTasks(id),
    enabled: !!id,
  });
}

// Create team mutation
export function useCreateTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: teamsApi.createTeam,
    onSuccess: (newTeam) => {
      queryClient.invalidateQueries({ queryKey: teamKeys.lists() });
      queryClient.setQueryData(teamKeys.detail(newTeam.id), newTeam);
    },
  });
}

// Pause team mutation
export function usePauseTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: teamsApi.pauseTeam,
    onSuccess: (updatedTeam) => {
      queryClient.invalidateQueries({ queryKey: teamKeys.lists() });
      queryClient.setQueryData(teamKeys.detail(updatedTeam.id), updatedTeam);
    },
  });
}

// Resume team mutation
export function useResumeTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: teamsApi.resumeTeam,
    onSuccess: (updatedTeam) => {
      queryClient.invalidateQueries({ queryKey: teamKeys.lists() });
      queryClient.setQueryData(teamKeys.detail(updatedTeam.id), updatedTeam);
    },
  });
}
```

- [ ] 4.1.4: Create React Query provider

```typescript
// src/components/providers/QueryProvider.tsx
"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { useState } from "react";

export function QueryProvider({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 60 * 1000, // 1 minute
            refetchOnWindowFocus: false,
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      {children}
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
```

- [ ] 4.1.5: Add provider to layout

```typescript
// src/app/layout.tsx
import { QueryProvider } from "@/components/providers/QueryProvider";
import "./globals.css";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <QueryProvider>{children}</QueryProvider>
      </body>
    </html>
  );
}
```

- [ ] 4.1.6: Create team list component

```typescript
// src/components/teams/TeamList.tsx
"use client";

import Link from "next/link";
import { Loader2, Users, DollarSign } from "lucide-react";
import { useTeams } from "@/lib/hooks/useTeams";
import { TeamStatusBadge } from "@/components/teams/TeamStatusBadge";
import { formatCurrency, formatRelativeTime } from "@/lib/utils";

export function TeamList() {
  const { data: teams, isLoading, error } = useTeams();

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center text-destructive p-8">
        Failed to load teams. Please try again.
      </div>
    );
  }

  if (!teams || teams.length === 0) {
    return (
      <div className="text-center p-12 border-2 border-dashed border-border rounded-lg">
        <Users className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
        <h3 className="text-lg font-semibold mb-2">No teams yet</h3>
        <p className="text-muted-foreground mb-4">
          Create your first AI team to get started
        </p>
        <Link
          href="/dashboard/teams/new"
          className="inline-flex items-center justify-center px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
        >
          Create Team
        </Link>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {teams.map((team) => (
        <Link
          key={team.id}
          href={`/dashboard/teams/${team.id}`}
          className="block p-6 bg-card border border-border rounded-lg hover:border-primary transition-colors"
        >
          <div className="flex items-start justify-between mb-3">
            <TeamStatusBadge status={team.status} />
            <span className="text-sm text-muted-foreground">
              {formatRelativeTime(team.created_at)}
            </span>
          </div>

          <h3 className="font-semibold mb-2 line-clamp-2">{team.goal}</h3>

          <div className="flex items-center justify-between text-sm text-muted-foreground">
            <div className="flex items-center gap-1">
              <DollarSign className="w-4 h-4" />
              <span>{formatCurrency(team.actual_cost)}</span>
            </div>
            {team.budget_limit && (
              <span className="text-xs">
                / {formatCurrency(team.budget_limit)}
              </span>
            )}
          </div>
        </Link>
      ))}
    </div>
  );
}
```

- [ ] 4.1.7: Create team status badge component

```typescript
// src/components/teams/TeamStatusBadge.tsx
import { TeamStatus } from "@/types/team";
import { cn } from "@/lib/utils";

interface TeamStatusBadgeProps {
  status: TeamStatus;
}

export function TeamStatusBadge({ status }: TeamStatusBadgeProps) {
  const variants = {
    [TeamStatus.Pending]: "bg-gray-100 text-gray-800",
    [TeamStatus.Planning]: "bg-blue-100 text-blue-800",
    [TeamStatus.Active]: "bg-green-100 text-green-800",
    [TeamStatus.Paused]: "bg-yellow-100 text-yellow-800",
    [TeamStatus.Completed]: "bg-purple-100 text-purple-800",
    [TeamStatus.Failed]: "bg-red-100 text-red-800",
    [TeamStatus.Archived]: "bg-gray-100 text-gray-600",
  };

  return (
    <span
      className={cn(
        "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
        variants[status]
      )}
    >
      {status}
    </span>
  );
}
```

**Acceptance Criteria**:

- [ ] API client configured with auth
- [ ] React Query provider setup
- [ ] useTeams hook returns data
- [ ] Team list renders teams
- [ ] Loading state displays spinner
- [ ] Error state displays message
- [ ] Empty state displays CTA
- [ ] Status badges color-coded

---

## Epic 5: Task Display Components

### Task 5.1: Create Task Components

**Type**: Frontend
**Dependencies**: Epic 4 complete

**Subtasks**:

- [ ] 5.1.1: Create task list component

```typescript
// src/components/tasks/TaskList.tsx
"use client";

import { Task, TaskStatus } from "@/types/task";
import { TaskCard } from "./TaskCard";
import { Loader2 } from "lucide-react";

interface TaskListProps {
  tasks: Task[];
  isLoading?: boolean;
}

export function TaskList({ tasks, isLoading }: TaskListProps) {
  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-32">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!tasks || tasks.length === 0) {
    return (
      <div className="text-center text-muted-foreground py-8">
        No tasks yet
      </div>
    );
  }

  // Group tasks by status
  const grouped = tasks.reduce((acc, task) => {
    if (!acc[task.status]) {
      acc[task.status] = [];
    }
    acc[task.status].push(task);
    return acc;
  }, {} as Record<TaskStatus, Task[]>);

  const statusOrder: TaskStatus[] = [
    TaskStatus.Pending,
    TaskStatus.Assigned,
    TaskStatus.InProgress,
    TaskStatus.Review,
    TaskStatus.RevisionRequested,
    TaskStatus.Blocked,
    TaskStatus.Completed,
    TaskStatus.Failed,
  ];

  return (
    <div className="space-y-6">
      {statusOrder.map((status) => {
        const statusTasks = grouped[status];
        if (!statusTasks || statusTasks.length === 0) return null;

        return (
          <div key={status}>
            <h3 className="text-sm font-semibold uppercase text-muted-foreground mb-3">
              {status.replace("_", " ")} ({statusTasks.length})
            </h3>
            <div className="space-y-2">
              {statusTasks.map((task) => (
                <TaskCard key={task.id} task={task} />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
```

- [ ] 5.1.2: Create task card component

```typescript
// src/components/tasks/TaskCard.tsx
import { Task } from "@/types/task";
import { TaskStatusBadge } from "./TaskStatusBadge";
import { Clock, CheckCircle2 } from "lucide-react";
import { formatRelativeTime } from "@/lib/utils";

interface TaskCardProps {
  task: Task;
}

export function TaskCard({ task }: TaskCardProps) {
  const progress =
    (task.acceptance_criteria.length - task.revision_count) /
    task.acceptance_criteria.length;

  return (
    <div className="p-4 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors">
      <div className="flex items-start justify-between mb-2">
        <h4 className="font-medium flex-1 mr-4">{task.title}</h4>
        <TaskStatusBadge status={task.status} />
      </div>

      <p className="text-sm text-muted-foreground mb-3 line-clamp-2">
        {task.description}
      </p>

      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1">
            <CheckCircle2 className="w-3 h-3" />
            <span>
              {task.acceptance_criteria.length} criteria
            </span>
          </div>
          {task.start_time && (
            <div className="flex items-center gap-1">
              <Clock className="w-3 h-3" />
              <span>{formatRelativeTime(task.start_time)}</span>
            </div>
          )}
        </div>

        {task.priority && (
          <span className="px-2 py-0.5 rounded bg-muted">
            P{task.priority}
          </span>
        )}
      </div>

      {/* Progress bar for in-progress tasks */}
      {task.status === "in_progress" && (
        <div className="mt-3">
          <div className="h-1 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${progress * 100}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] 5.1.3: Create task status badge

```typescript
// src/components/tasks/TaskStatusBadge.tsx
import { TaskStatus } from "@/types/task";
import { cn } from "@/lib/utils";

interface TaskStatusBadgeProps {
  status: TaskStatus;
}

export function TaskStatusBadge({ status }: TaskStatusBadgeProps) {
  const variants = {
    [TaskStatus.Pending]: "bg-gray-100 text-gray-800",
    [TaskStatus.Assigned]: "bg-blue-100 text-blue-800",
    [TaskStatus.InProgress]: "bg-indigo-100 text-indigo-800",
    [TaskStatus.Review]: "bg-purple-100 text-purple-800",
    [TaskStatus.Completed]: "bg-green-100 text-green-800",
    [TaskStatus.Failed]: "bg-red-100 text-red-800",
    [TaskStatus.RevisionRequested]: "bg-orange-100 text-orange-800",
    [TaskStatus.Blocked]: "bg-yellow-100 text-yellow-800",
  };

  return (
    <span
      className={cn(
        "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium",
        variants[status]
      )}
    >
      {status.replace("_", " ")}
    </span>
  );
}
```

**Acceptance Criteria**:

- [ ] Task list renders tasks grouped by status
- [ ] Task cards display all relevant info
- [ ] Status badges color-coded
- [ ] Progress bar shows for in-progress tasks
- [ ] Loading state functional
- [ ] Empty state displays message

---

## Epic 6: Real-time Status Updates

### Task 6.1: Implement WebSocket Connection

**Type**: Frontend
**Dependencies**: Backend WebSocket from Phase 3

**Subtasks**:

- [ ] 6.1.1: Create WebSocket hook

```typescript
// src/lib/hooks/useWebSocket.ts
import { useEffect, useRef, useState } from "react";

interface UseWebSocketOptions {
  onMessage?: (data: any) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  reconnect?: boolean;
  reconnectDelay?: number;
}

export function useWebSocket(url: string, options: UseWebSocketOptions = {}) {
  const {
    onMessage,
    onOpen,
    onClose,
    onError,
    reconnect = true,
    reconnectDelay = 3000,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  const connect = () => {
    const token = localStorage.getItem("auth_token");
    const wsUrl = `${url}?token=${token}`;

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log("WebSocket connected");
      setIsConnected(true);
      onOpen?.();
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onMessage?.(data);
      } catch (error) {
        console.error("Failed to parse WebSocket message:", error);
      }
    };

    ws.onclose = () => {
      console.log("WebSocket disconnected");
      setIsConnected(false);
      wsRef.current = null;
      onClose?.();

      // Attempt reconnect
      if (reconnect) {
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log("Attempting to reconnect...");
          connect();
        }, reconnectDelay);
      }
    };

    ws.onerror = (error) => {
      console.error("WebSocket error:", error);
      onError?.(error);
    };

    wsRef.current = ws;
  };

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [url]);

  const send = (data: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    } else {
      console.warn("WebSocket is not connected");
    }
  };

  return { isConnected, send };
}
```

- [ ] 6.1.2: Create team updates hook

```typescript
// src/lib/hooks/useTeamUpdates.ts
import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useWebSocket } from "./useWebSocket";
import { teamKeys } from "./useTeams";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:4000/ws";

interface TeamUpdate {
  type: "task_assigned" | "task_completed" | "task_failed" | "cost_updated" | "status_changed";
  team_id: string;
  data: any;
}

export function useTeamUpdates(teamId: string) {
  const queryClient = useQueryClient();

  const { isConnected } = useWebSocket(`${WS_URL}/teams/${teamId}`, {
    onMessage: (update: TeamUpdate) => {
      console.log("Team update received:", update);

      switch (update.type) {
        case "task_assigned":
        case "task_completed":
        case "task_failed":
          // Invalidate tasks query
          queryClient.invalidateQueries({
            queryKey: teamKeys.tasks(teamId),
          });
          break;

        case "cost_updated":
          // Invalidate team query to get updated cost
          queryClient.invalidateQueries({
            queryKey: teamKeys.detail(teamId),
          });
          break;

        case "status_changed":
          // Invalidate both team detail and list
          queryClient.invalidateQueries({
            queryKey: teamKeys.detail(teamId),
          });
          queryClient.invalidateQueries({
            queryKey: teamKeys.lists(),
          });
          break;
      }
    },
    onOpen: () => {
      console.log(`Connected to team ${teamId} updates`);
    },
    onClose: () => {
      console.log(`Disconnected from team ${teamId} updates`);
    },
  });

  return { isConnected };
}
```

- [ ] 6.1.3: Add real-time updates to team detail page

```typescript
// src/app/(dashboard)/dashboard/teams/[id]/page.tsx
"use client";

import { useParams } from "next/navigation";
import { Loader2, Wifi, WifiOff } from "lucide-react";
import { useTeam, useTeamMembers, useTeamTasks } from "@/lib/hooks/useTeams";
import { useTeamUpdates } from "@/lib/hooks/useTeamUpdates";
import { TeamHeader } from "@/components/teams/TeamHeader";
import { TeamMembersPanel } from "@/components/teams/TeamMembersPanel";
import { TaskList } from "@/components/tasks/TaskList";

export default function TeamDetailPage() {
  const params = useParams();
  const teamId = params.id as string;

  const { data: team, isLoading: teamLoading } = useTeam(teamId);
  const { data: members, isLoading: membersLoading } = useTeamMembers(teamId);
  const { data: tasks, isLoading: tasksLoading } = useTeamTasks(teamId);
  const { isConnected } = useTeamUpdates(teamId);

  if (teamLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!team) {
    return <div className="text-center text-destructive">Team not found</div>;
  }

  return (
    <div className="space-y-6">
      {/* Real-time indicator */}
      <div className="flex items-center justify-between">
        <div></div>
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          {isConnected ? (
            <>
              <Wifi className="w-4 h-4 text-green-500" />
              <span>Live updates</span>
            </>
          ) : (
            <>
              <WifiOff className="w-4 h-4 text-red-500" />
              <span>Reconnecting...</span>
            </>
          )}
        </div>
      </div>

      <TeamHeader team={team} />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <h2 className="text-xl font-semibold mb-4">Tasks</h2>
          <TaskList tasks={tasks || []} isLoading={tasksLoading} />
        </div>

        <div>
          <h2 className="text-xl font-semibold mb-4">Team Members</h2>
          <TeamMembersPanel members={members || []} isLoading={membersLoading} />
        </div>
      </div>
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] WebSocket connects on page load
- [ ] Real-time indicator shows connection status
- [ ] Task updates trigger UI refresh
- [ ] Cost updates trigger UI refresh
- [ ] Status changes trigger UI refresh
- [ ] Automatic reconnection working
- [ ] No memory leaks from WebSocket

---

## Success Criteria - Phase 5 Complete

- [ ] Next.js app running with App Router
- [ ] Team creation wizard functional
- [ ] Dashboard layout with sidebar
- [ ] Team list displays all teams
- [ ] Task list grouped by status
- [ ] Real-time updates working via WebSocket
- [ ] React Query caching optimal
- [ ] All components responsive
- [ ] Loading and error states handled
- [ ] No console errors

---

## Next Steps

Proceed to [09-phase-6-realtime-audit.md](./09-phase-6-realtime-audit.md) for audit trail and advanced real-time features.

---

**Phase 5: Modern React Frontend Online**
