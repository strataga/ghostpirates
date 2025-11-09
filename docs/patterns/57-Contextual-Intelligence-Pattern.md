# Pattern 57: Contextual Intelligence Pattern (Smart Alerts & Insights)

**Category:** Artificial Intelligence Pattern
**Complexity:** Medium-High
**Related Patterns:** Strategy Pattern, Observer Pattern, Cache-Aside Pattern, Smart Suggestions Pattern (#56)

---

## Intent

Proactively surface actionable business intelligence to users **exactly when and where they need it**, through context-aware insights that appear conditionally based on real-time data analysis.

---

## Problem

**The Challenge:**

Traditional PSA platforms dump data dashboards on users and expect them to:

1. Remember to check dashboards regularly
2. Interpret metrics and identify problems
3. Know what action to take when issues arise
4. Context-switch between pages to understand full picture

**Real-World Consequences:**

- Budget overruns discovered too late (already 20% over)
- Overdue invoices ignored for weeks (cash flow problems)
- Team burnout unnoticed until someone quits
- Profitability issues caught after project completion
- Compliance problems (missing timesheets) discovered at month-end

### Traditional Approach Problems

```typescript
// ❌ Traditional PSA Approach
// User must actively navigate to dashboard
// User must interpret charts and metrics
// User must remember to check regularly
// No proactive warnings or guidance
// Issues discovered retroactively, not preventatively

// Example: Budget overrun
// Day 1: Project at 75% budget (dashboard shows yellow)
// Day 5: Project at 95% budget (dashboard shows orange)
// Day 8: Project at 110% budget (dashboard shows red) ← TOO LATE!
// Manager finally notices: "Why didn't anyone tell me?"
```

---

## Solution

The **Contextual Intelligence Pattern** combines rule-based analysis with context-aware presentation to deliver actionable insights **at the point of need**:

1. **Conditional Rendering**: Insights only appear when business conditions warrant attention
2. **Context-Aware**: Different insights based on current page and user role
3. **Action-Oriented**: Every insight includes specific recommended action
4. **Just-in-Time**: Appear exactly when user is doing related work
5. **Progressive Enhancement**: Start with rules, enhance with AI later (Sprint 8)

### Key Principles

1. **Show, Don't Tell**: Present insight where user can act, not in a separate dashboard
2. **Signal vs Noise**: Only show insights that matter (high signal-to-noise ratio)
3. **Dismissible but Persistent**: User can hide, but critical alerts re-appear
4. **Explain Why**: Always provide context (not just "budget exceeded")
5. **Suggest What**: Recommend specific action (not just "review project")

---

## Architecture

### Two-Phase Implementation

#### Phase 1: Rule-Based Insights (Sprint 7)

```
User Action (e.g., open time entry page)
    ↓
Insight Generators (Strategy Pattern)
    ↓
Rule-Based Analysis (deterministic logic)
    ↓
Conditional Insights (only if thresholds met)
    ↓
ContextualInsightsBanner Component (UI)
    ↓
User Takes Action (or dismisses)
```

#### Phase 2: AI-Enhanced Insights (Sprint 8)

```
Rule-Based Insight Generated
    ↓
Severity = CRITICAL or Complex Pattern?
    ↓ Yes
Claude AI Enhancement (natural language + prediction)
    ↓
Enhanced Insight with Reasoning
    ↓
Cache for 4 hours (avoid repeat API calls)
    ↓
ContextualInsightsBanner Component
```

---

## Implementation

### Phase 1: Rule-Based Insights (Sprint 7 - No AI Required)

**Domain Layer - Value Objects:**

```typescript
// apps/api/src/domain/insights/contextual-insight.vo.ts

export type InsightSeverity = 'INFO' | 'WARNING' | 'CRITICAL';

export interface InsightAction {
  label: string;
  type: 'button' | 'link';
  variant: 'primary' | 'secondary' | 'ghost';
  href?: string;
}

export interface InsightContext {
  page: string;
  entityType?: 'project' | 'invoice' | 'client' | 'timeEntry';
  entityId?: string;
}

export class ContextualInsight {
  constructor(
    public readonly id: string,
    public readonly severity: InsightSeverity,
    public readonly title: string,
    public readonly message: string,
    public readonly icon: string, // emoji
    public readonly actions: InsightAction[],
    public readonly dismissible: boolean,
    public readonly context: InsightContext,
  ) {}

  static createBudgetWarning(project: Project, utilization: number): ContextualInsight {
    const severity = utilization >= 1.0 ? 'CRITICAL' : 'WARNING';
    const title = utilization >= 1.0 ? 'Budget Exceeded' : 'Approaching Budget Limit';
    const message =
      `You're at ${(utilization * 100).toFixed(0)}% budget on ${project.name}. ` +
      (utilization >= 1.0
        ? 'This project has exceeded its budget.'
        : 'Logging 2+ more hours will exceed budget.');

    return new ContextualInsight(
      `budget-${project.id}-${Date.now()}`,
      severity,
      title,
      message,
      utilization >= 1.0 ? '🚨' : '⚠️',
      [
        {
          label: 'Review Budget',
          type: 'link',
          variant: 'primary',
          href: `/projects/${project.id}`,
        },
        {
          label: 'Continue Anyway',
          type: 'button',
          variant: 'ghost',
        },
      ],
      utilization < 1.0, // Only dismissible if warning, not critical
      {
        page: 'time-entry',
        entityType: 'project',
        entityId: project.id,
      },
    );
  }
}
```

**Domain Layer - Insight Generator Interface (Strategy Pattern):**

```typescript
// apps/api/src/domain/insights/insight-generator.interface.ts

export interface IInsightGenerator {
  /**
   * Generate insights for given context
   */
  generate(
    context: InsightContext,
    userId: string,
    organizationId: string,
  ): Promise<ContextualInsight[]>;

  /**
   * Does this generator support this context?
   */
  supports(context: InsightContext): boolean;
}
```

**Domain Layer - Budget Insight Generator:**

```typescript
// apps/api/src/domain/insights/budget-insight-generator.ts

export class BudgetInsightGenerator implements IInsightGenerator {
  constructor(
    private readonly projectRepo: IProjectRepository,
    private readonly timeEntryRepo: ITimeEntryRepository,
  ) {}

  supports(context: InsightContext): boolean {
    return (
      (context.page === 'time-entry' || context.page === 'project-detail') && !!context.projectId
    );
  }

  async generate(
    context: InsightContext,
    userId: string,
    organizationId: string,
  ): Promise<ContextualInsight[]> {
    const insights: ContextualInsight[] = [];

    if (!context.projectId) return insights;

    // Load project
    const project = await this.projectRepo.findById(context.projectId);
    if (!project || !project.budget) return insights;

    // Calculate current utilization
    const actualCost = await this.timeEntryRepo.getTotalCostByProject(project.id, organizationId, {
      status: 'APPROVED',
    });

    const utilization = actualCost.totalCost / project.budget.amount;

    // Only generate insight if over threshold
    if (utilization >= 0.9) {
      insights.push(ContextualInsight.createBudgetWarning(project, utilization));
    }

    return insights;
  }
}
```

**Domain Layer - Compliance Insight Generator:**

```typescript
// apps/api/src/domain/insights/compliance-insight-generator.ts

export class ComplianceInsightGenerator implements IInsightGenerator {
  constructor(private readonly timeEntryRepo: ITimeEntryRepository) {}

  supports(context: InsightContext): boolean {
    return context.page === 'time-entry' || context.page === 'dashboard';
  }

  async generate(
    context: InsightContext,
    userId: string,
    organizationId: string,
  ): Promise<ContextualInsight[]> {
    const insights: ContextualInsight[] = [];

    // Check for old draft entries (>7 days)
    const draftCount = await this.timeEntryRepo.countDraftEntriesBefore(
      userId,
      organizationId,
      new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
    );

    if (draftCount > 0) {
      insights.push(
        new ContextualInsight(
          `draft-entries-${userId}-${Date.now()}`,
          'INFO',
          'Draft Entries from Last Week',
          `You have ${draftCount} draft entries from last week. Submit them for approval?`,
          '⏰',
          [
            {
              label: 'View Drafts',
              type: 'link',
              variant: 'primary',
              href: '/time-entries?status=DRAFT',
            },
          ],
          true, // Dismissible
          {
            page: context.page,
          },
        ),
      );
    }

    // Check for missing timesheet (if today is Friday and <40h logged this week)
    const today = new Date();
    if (today.getDay() === 5) {
      // Friday
      const weekStart = startOfWeek(today);
      const weekHours = await this.timeEntryRepo.getTotalHoursByUserInRange(
        userId,
        organizationId,
        weekStart,
        today,
      );

      if (weekHours < 40) {
        insights.push(
          new ContextualInsight(
            `incomplete-week-${userId}-${Date.now()}`,
            'WARNING',
            'Timesheet Incomplete',
            `You've only logged ${weekHours}h this week. Don't forget to submit your full timesheet!`,
            '📋',
            [
              {
                label: 'Log Time',
                type: 'link',
                variant: 'primary',
                href: '/time-entries/new',
              },
            ],
            true,
            { page: context.page },
          ),
        );
      }
    }

    return insights;
  }
}
```

**Application Layer - Query Handler:**

```typescript
// apps/api/src/application/insights/queries/get-contextual-insights.handler.ts

export class GetContextualInsightsQuery {
  constructor(
    public readonly userId: string,
    public readonly organizationId: string,
    public readonly context: InsightContext,
  ) {}
}

@QueryHandler(GetContextualInsightsQuery)
export class GetContextualInsightsHandler {
  constructor(
    private readonly generators: IInsightGenerator[], // Injected array of all generators
    private readonly cacheService: ICacheService,
  ) {}

  async execute(query: GetContextualInsightsQuery): Promise<ContextualInsight[]> {
    const { userId, organizationId, context } = query;

    // Check cache first (5-minute TTL)
    const cacheKey = `insights:${userId}:${context.page}:${context.entityId}`;
    const cached = await this.cacheService.get<ContextualInsight[]>(cacheKey);
    if (cached) return cached;

    // Generate insights using all applicable generators
    const insightPromises = this.generators
      .filter((gen) => gen.supports(context))
      .map((gen) => gen.generate(context, userId, organizationId));

    const nestedInsights = await Promise.all(insightPromises);
    const insights = nestedInsights.flat();

    // Cache for 5 minutes
    await this.cacheService.set(cacheKey, insights, 300);

    return insights;
  }
}
```

**Infrastructure Layer - Module Configuration:**

```typescript
// apps/api/src/presentation/insights/insights.module.ts

@Module({
  imports: [CqrsModule, RepositoriesModule],
  providers: [
    GetContextualInsightsHandler,
    // Register all insight generators
    {
      provide: 'INSIGHT_GENERATORS',
      useFactory: (
        projectRepo: IProjectRepository,
        timeEntryRepo: ITimeEntryRepository,
        invoiceRepo: IInvoiceRepository,
      ) => [
        new BudgetInsightGenerator(projectRepo, timeEntryRepo),
        new ComplianceInsightGenerator(timeEntryRepo),
        new ProfitabilityInsightGenerator(projectRepo, timeEntryRepo),
        new InvoiceInsightGenerator(invoiceRepo),
      ],
      inject: ['IProjectRepository', 'ITimeEntryRepository', 'IInvoiceRepository'],
    },
  ],
  controllers: [InsightsController],
})
export class InsightsModule {}
```

**Presentation Layer - Controller:**

```typescript
// apps/api/src/presentation/insights/insights.controller.ts

@Controller('insights')
export class InsightsController {
  constructor(
    private readonly queryBus: QueryBus,
    private readonly currentUserService: CurrentUserService,
  ) {}

  @Get('contextual')
  @UseGuards(JwtAuthGuard)
  async getContextualInsights(
    @Query() dto: GetContextualInsightsDto,
  ): Promise<ContextualInsightDto[]> {
    const user = this.currentUserService.getUser();

    const query = new GetContextualInsightsQuery(user.id, user.organizationId, dto.context);

    const insights = await this.queryBus.execute(query);

    return insights.map((i) => i.toJSON());
  }
}
```

**Frontend - ContextualInsightsBanner Component:**

```tsx
// apps/web/components/insights/contextual-insights-banner.tsx

export function ContextualInsightsBanner() {
  const pathname = usePathname();
  const context = extractContextFromPath(pathname);

  const { data: insights, isLoading } = useContextualInsights(context);

  // Don't render if no insights
  if (!insights || insights.length === 0) return null;

  return (
    <div className="w-full space-y-2 mb-6">
      {insights.slice(0, 3).map((insight) => (
        <InsightCard key={insight.id} insight={insight} />
      ))}
      {insights.length > 3 && (
        <button onClick={() => showAllInsights()}>Show {insights.length - 3} more insights</button>
      )}
    </div>
  );
}

function InsightCard({ insight }: { insight: ContextualInsight }) {
  const [isDismissed, setIsDismissed] = useState(false);
  const dismissMutation = useDismissInsightMutation();

  if (isDismissed) return null;

  const severityStyles = {
    INFO: 'bg-blue-50 border-blue-200 text-blue-900',
    WARNING: 'bg-yellow-50 border-yellow-200 text-yellow-900',
    CRITICAL: 'bg-red-50 border-red-200 text-red-900',
  };

  return (
    <div className={`p-4 rounded-lg border-2 ${severityStyles[insight.severity]}`}>
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3 flex-1">
          <span className="text-2xl">{insight.icon}</span>
          <div className="flex-1">
            <h4 className="font-semibold">{insight.title}</h4>
            <p className="text-sm mt-1">{insight.message}</p>

            {insight.actions && insight.actions.length > 0 && (
              <div className="flex gap-2 mt-3">
                {insight.actions.map((action, idx) => (
                  <Button
                    key={idx}
                    variant={action.variant}
                    onClick={action.onClick}
                    href={action.href}
                  >
                    {action.label}
                  </Button>
                ))}
              </div>
            )}
          </div>
        </div>

        {insight.dismissible && (
          <button
            onClick={() => {
              dismissMutation.mutate(insight.id);
              setIsDismissed(true);
            }}
            className="text-gray-400 hover:text-gray-600"
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
}
```

---

### Phase 2: Claude AI Enhancement (Sprint 8)

**Enhance complex insights with natural language and predictions:**

```typescript
// apps/api/src/domain/insights/claude-insight-enhancer.ts

export class ClaudeInsightEnhancer {
  constructor(private readonly claudeClient: ClaudeClient) {}

  async enhance(
    baseInsight: ContextualInsight,
    context: EnhancementContext,
  ): Promise<ContextualInsight> {
    // Only enhance CRITICAL insights or specific types (save API costs)
    if (!this.shouldEnhance(baseInsight)) {
      return baseInsight;
    }

    const prompt = this.buildPrompt(baseInsight, context);

    const response = await this.claudeClient.generateText({
      prompt,
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 200,
      temperature: 0.3, // Low temp for factual analysis
    });

    return {
      ...baseInsight,
      message: response.text, // Replace with Claude's enhanced explanation
      metadata: {
        enhanced: true,
        enhancedAt: new Date().toISOString(),
      },
    };
  }

  private shouldEnhance(insight: ContextualInsight): boolean {
    // Only enhance complex insights
    return (
      insight.severity === 'CRITICAL' || insight.context.entityType === 'project' // Profitability insights
    );
  }

  private buildPrompt(insight: ContextualInsight, context: EnhancementContext): string {
    return `
You are an AI business analyst for WellOS PSA platform.

Context:
- User: ${context.userName}, Role: ${context.userRole}
- Organization: ${context.orgName}
- Current Page: ${context.page}

Base Insight:
${insight.title}: ${insight.message}

Project Data:
- Budget: ${context.project?.budget?.toJSON()}
- Spent: ${context.project?.actualCost?.toJSON()}
- Burn Rate: ${context.project?.burnRate} per day
- Remaining Days: ${context.project?.estimatedDaysToCompletion}

Historical Context:
${JSON.stringify(context.historicalData, null, 2)}

Task:
Enhance this insight with:
1. Natural language explanation of WHY this is happening
2. Predictive analysis (when will budget be exceeded?)
3. Specific, actionable recommendation
4. Impact if no action taken

Keep response concise (2-3 sentences max).
Format: "[Explanation]. [Prediction]. [Recommendation]."
`;
  }
}
```

**Example Enhancement:**

```typescript
// Sprint 7 (Rule-Based):
"⚠️ You're at 92% budget on Project Alpha. Logging 2+ more hours will exceed your $50,000 budget.";

// Sprint 8 (Claude-Enhanced):
"⚠️ Project Alpha is at 92% budget ($46K of $50K spent). At your current burn rate of $2K/day, you'll exceed budget in 2 days (Jan 15th). Consider discussing scope reduction with the client or requesting a budget increase of at least $5K to complete remaining work.";
```

---

## Benefits

### User Benefits

1. **Proactive Awareness**: Problems surfaced before they escalate
2. **Context-Sensitive**: Insights appear exactly when relevant
3. **Action-Oriented**: Clear guidance on what to do
4. **Time Savings**: No need to manually check dashboards
5. **Reduced Stress**: System watches for problems, not user

### Business Benefits

1. **Prevent Budget Overruns**: Warn before exceeding budget
2. **Improve Cash Flow**: Surface overdue invoices immediately
3. **Increase Compliance**: Remind about missing timesheets
4. **Better Decisions**: Real-time intelligence at point of decision
5. **Competitive Advantage**: "AI-powered insights" differentiator

---

## Metrics

### Target Metrics

- **60%+ of users** see at least 1 insight per week
- **40%+ action rate** (user clicks insight button)
- **<20% dismissal rate** without action (high relevance)
- **<5% false positives** (insights that aren't relevant)
- **<100ms load time** (cached insights)

### How to Measure

```typescript
// Track insight impressions and actions
export class TrackInsightActionCommand {
  constructor(
    public readonly insightId: string,
    public readonly action: 'viewed' | 'clicked' | 'dismissed',
  ) {}
}

// Calculate insight effectiveness
export class CalculateInsightEffectivenessQuery {
  async execute(): Promise<InsightMetrics> {
    const insights = await this.getRecentInsights();

    return {
      totalShown: insights.length,
      actionRate: insights.filter((i) => i.action === 'clicked').length / insights.length,
      dismissalRate: insights.filter((i) => i.action === 'dismissed').length / insights.length,
      avgTimeToAction: this.calculateAvgTime(insights),
    };
  }
}
```

---

## When to Use

Use this pattern when:

- ✅ Users need to be alerted about business conditions
- ✅ Context matters (different insights for different pages)
- ✅ Actions can be taken immediately
- ✅ Data analysis can be automated (rules or AI)
- ✅ Users benefit from proactive guidance

Don't use this pattern when:

- ❌ Insights would be noise (alert fatigue)
- ❌ No clear action for user to take
- ❌ Context doesn't matter (same insight everywhere)
- ❌ Users already monitoring dashboards actively

---

## Related Patterns

- **Pattern #56: Smart Suggestions Pattern** (time entry predictions)
- **Strategy Pattern** (different insight generators)
- **Observer Pattern** (insights react to data changes)
- **Cache-Aside Pattern** (Redis caching for performance)
- **Factory Pattern** (create appropriate insights)

---

## Anti-Patterns

### ❌ Alert Fatigue

```typescript
// Showing too many insights (10+ per page)
// Showing low-importance insights
// Showing same insight repeatedly
// Result: User ignores all insights, banners become wallpaper
```

### ❌ Actionless Insights

```typescript
// "Your profitability is low" (OK, so what do I do?)
// "3 projects are at risk" (which ones? what action?)
// No clear next step provided
// Result: User frustrated, insights feel useless
```

### ❌ Context-Blind Insights

```typescript
// Showing invoice insights on time entry page
// Showing project budget on client detail page
// Insights not relevant to current task
// Result: User dismisses without reading (noise)
```

---

## Implementation Checklist

### Sprint 7: Rule-Based Foundation

- [x] Create `ContextualInsight` value object
- [x] Create `IInsightGenerator` interface
- [x] Implement `BudgetInsightGenerator`
- [x] Implement `ComplianceInsightGenerator`
- [x] Implement `ProfitabilityInsightGenerator`
- [x] Create `GetContextualInsightsQuery` + handler
- [x] Add `GET /api/insights/contextual` endpoint
- [x] Implement Redis caching (5-min TTL)
- [x] Build `ContextualInsightsBanner` component
- [x] Build `InsightCard` component
- [x] Add dismissal logic (localStorage)
- [x] Integrate into page layouts
- [x] Unit tests for all generators
- [x] E2E tests for insights display

### Sprint 8: Claude AI Enhancement

- [ ] Create `ClaudeInsightEnhancer` service
- [ ] Integrate with existing Claude client from US-613
- [ ] Enhance CRITICAL insights with Claude
- [ ] Add 4-hour cache for enhanced insights
- [ ] Track AI enhancement costs
- [ ] A/B test: rule-based vs AI-enhanced acceptance rates
- [ ] Optimize prompts for cost efficiency

---

## Code Locations

- Insight Value Objects: `apps/api/src/domain/insights/`
- Insight Generators: `apps/api/src/domain/insights/*-generator.ts`
- Query Handler: `apps/api/src/application/insights/queries/get-contextual-insights.handler.ts`
- Banner Component: `apps/web/components/insights/contextual-insights-banner.tsx`
- Hooks: `apps/web/hooks/insights/use-contextual-insights.ts`

---

## References

### Related User Stories

- Sprint 7 - US-711 Enhancement: Contextual Insights Component
- Sprint 8 - US-613: AI Chat Assistant (Claude integration)

### Documentation

- Sprint 7 Enhancement Plan: `docs/sprints/sprint-07-us-711-contextual-insights-enhancement.md`
- Pattern #56: Smart Suggestions Pattern (time entry predictions)

---

**Pattern Status:** ✅ Planned for Sprint 7 (Rule-Based) + Sprint 8 (AI-Enhanced)

**Last Updated:** January 2025

---

## Conclusion

The Contextual Intelligence Pattern transforms passive dashboards into proactive guidance by delivering insights **exactly when and where users need them**. By combining rule-based analysis (Sprint 7) with AI enhancement (Sprint 8), we create a system that feels intelligent without requiring complex machine learning infrastructure.

**Key Takeaway:** Context matters. Showing a budget warning when the user is logging time to that project is 10x more valuable than showing it on a dashboard they check once a week. Progressive enhancement (rules → AI) allows us to deliver value immediately while building toward more sophisticated intelligence.
