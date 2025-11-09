# Pattern 56: Smart Suggestions Pattern (AI-Powered Predictions)

**Category:** Artificial Intelligence Pattern
**Complexity:** Medium
**Related Patterns:** Machine Learning Pattern, Pattern Recognition, Cache-Aside Pattern

---

## Intent

Reduce manual data entry through intelligent pattern recognition and predictive suggestions, addressing the #1 pain point in PSA platforms: "spending valuable billable time filling out timesheets."

---

## Problem

**Research Finding (January 2025):**

> "Manual timesheet entry plagues most platforms - users report 'spending valuable billable time filling out timesheets' and human error when reconstructing work weeks retrospectively."

> "True predictive analytics, intelligent resource matching, and automated time capture remain rare. Most platforms offer only basic AI capabilities—rules-based workflow automation marketed as 'artificial intelligence.'"

### Why This Matters

- **Time Waste:** Consultants spend 10-30 minutes/week filling out timesheets
- **Errors:** Reconstructing work retrospectively leads to inaccurate time tracking
- **Compliance:** Manual entry results in forgotten/missing entries
- **User Friction:** Repetitive data entry is frustrating and error-prone
- **Revenue Loss:** Unbilled hours due to forgotten entries

### Traditional Approach Problems

```typescript
// ❌ Traditional PSA Time Entry
// User opens time entry form
// User manually selects project from dropdown (scrolling through 50+ projects)
// User manually enters duration (trying to remember what they worked on 3 days ago)
// User types description
// User saves
// Repeat for every day, every project
// Result: 10-30 minutes/week wasted, frequent errors, unbilled hours
```

---

## Solution

The **Smart Suggestions Pattern** uses simple statistical pattern recognition to predict what the user will log based on their historical behavior. For MVP, we use frequency analysis and day-of-week patterns. In future phases, we can enhance with machine learning.

### Key Principles

1. **Learn from History:** Analyze user's past time entries to identify patterns
2. **Context-Aware:** Consider day of week, time of day, recent activity
3. **Confidence Scoring:** Only show suggestions we're confident about
4. **User Control:** User can accept, modify, or dismiss suggestions
5. **Progressive Enhancement:** Start simple (frequency), enhance with ML later

---

## Implementation

### Phase 1: Frequency-Based Pattern Recognition (Sprint 7 - MVP)

**Simple statistical analysis - no ML required.**

```typescript
// Domain Layer - Value Objects
export class TimeEntryPattern {
  constructor(
    public readonly userId: string,
    public readonly projectId: string,
    public readonly dayOfWeek: number, // 0-6
    public readonly averageDuration: number, // minutes
    public readonly frequency: number, // # of occurrences
    public readonly confidence: number, // 0-1
    public readonly lastOccurrence: Date,
  ) {}

  static fromHistory(entries: TimeEntry[]): TimeEntryPattern {
    const durations = entries.map((e) => e.duration);
    const averageDuration = durations.reduce((sum, d) => sum + d, 0) / durations.length;
    const frequency = entries.length;
    const confidence = Math.min(frequency / 4, 1); // Max confidence after 4 occurrences

    return new TimeEntryPattern(
      entries[0].userId,
      entries[0].projectId,
      entries[0].startTime.getDay(),
      averageDuration,
      frequency,
      confidence,
      entries[entries.length - 1].startTime,
    );
  }
}

export class TimeEntrySuggestion {
  constructor(
    public readonly projectId: string,
    public readonly projectName: string,
    public readonly suggestedDuration: number, // minutes
    public readonly reason: string,
    public readonly confidence: number, // 0-1
  ) {}

  static fromPattern(pattern: TimeEntryPattern, project: Project): TimeEntrySuggestion {
    const hours = Math.round(pattern.averageDuration / 60);
    const dayName = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][
      pattern.dayOfWeek
    ];

    return new TimeEntrySuggestion(
      pattern.projectId,
      project.name,
      pattern.averageDuration,
      `You usually log ${hours}h on ${project.name} on ${dayName}s`,
      pattern.confidence,
    );
  }
}
```

**Pattern Analysis Service:**

```typescript
// Domain Layer - Service
export class PatternAnalysisService {
  constructor(private readonly timeEntryRepo: ITimeEntryRepository) {}

  async analyzePatterns(userId: string): Promise<TimeEntryPattern[]> {
    // Get last 4 weeks of time entries
    const cutoffDate = subWeeks(new Date(), 4);
    const recentEntries = await this.timeEntryRepo.findByUserSince(userId, cutoffDate);

    // Group by (dayOfWeek, projectId)
    const groups = this.groupEntries(recentEntries);

    // Create patterns from groups (filter by frequency >= 2)
    const patterns = Object.values(groups)
      .filter((entries) => entries.length >= 2)
      .map((entries) => TimeEntryPattern.fromHistory(entries))
      .sort((a, b) => b.confidence - a.confidence); // Sort by confidence

    return patterns;
  }

  private groupEntries(entries: TimeEntry[]): Record<string, TimeEntry[]> {
    return entries.reduce((groups, entry) => {
      const key = `${entry.startTime.getDay()}_${entry.projectId}`;
      if (!groups[key]) {
        groups[key] = [];
      }
      groups[key].push(entry);
      return groups;
    }, {});
  }
}
```

**Application Layer - Query Handler:**

```typescript
// Application Layer
export class GetTimeEntrySuggestionsQuery {
  constructor(
    public readonly userId: string,
    public readonly date: Date, // Date user is logging time for
  ) {}
}

@QueryHandler(GetTimeEntrySuggestionsQuery)
export class GetTimeEntrySuggestionsHandler {
  constructor(
    private readonly patternAnalysisService: PatternAnalysisService,
    private readonly projectRepo: IProjectRepository,
  ) {}

  async execute(query: GetTimeEntrySuggestionsQuery): Promise<TimeEntrySuggestion[]> {
    const { userId, date } = query;
    const dayOfWeek = date.getDay();

    // Analyze user's patterns
    const patterns = await this.patternAnalysisService.analyzePatterns(userId);

    // Filter to patterns matching today's day of week
    const matchingPatterns = patterns.filter((p) => p.dayOfWeek === dayOfWeek);

    // Convert to suggestions (top 3)
    const suggestions: TimeEntrySuggestion[] = [];
    for (const pattern of matchingPatterns.slice(0, 3)) {
      const project = await this.projectRepo.findById(pattern.projectId);
      if (project) {
        suggestions.push(TimeEntrySuggestion.fromPattern(pattern, project));
      }
    }

    return suggestions;
  }
}
```

### Scheduled Pattern Analysis

**Keep patterns fresh without expensive real-time calculations:**

```typescript
// Application Layer - Command (scheduled daily)
@Cron('0 2 * * *') // Run at 2 AM daily
export class AnalyzeTimeEntryPatternsCommand {
  constructor(
    private readonly userRepo: IUserRepository,
    private readonly patternAnalysisService: PatternAnalysisService,
    private readonly cacheService: ICacheService,
  ) {}

  async execute(): Promise<void> {
    const users = await this.userRepo.findAllActive();

    for (const user of users) {
      const patterns = await this.patternAnalysisService.analyzePatterns(user.id);

      // Cache patterns for 24 hours
      await this.cacheService.set(
        `time-entry-patterns:${user.id}`,
        patterns,
        60 * 60 * 24, // 24 hours TTL
      );
    }
  }
}
```

### Frontend Integration

**Suggestions Component:**

```tsx
// apps/web/components/time-entry/smart-suggestions.tsx
export function SmartSuggestions({ date, onAccept }: SmartSuggestionsProps) {
  const { data: suggestions, isLoading } = useTimeEntrySuggestionsQuery(date);

  if (isLoading || !suggestions?.length) {
    return null;
  }

  return (
    <Card className="mb-4 border-blue-200 bg-blue-50">
      <CardHeader>
        <div className="flex items-center gap-2">
          <SparklesIcon className="h-5 w-5 text-blue-600" />
          <CardTitle className="text-sm font-medium">Smart Suggestions</CardTitle>
          <Badge variant="secondary">AI-Powered</Badge>
        </div>
        <CardDescription className="text-xs">
          Based on your work patterns, we think you might have worked on:
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {suggestions.map((suggestion, index) => (
            <SuggestionCard
              key={index}
              suggestion={suggestion}
              onAccept={() => onAccept(suggestion)}
            />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function SuggestionCard({ suggestion, onAccept }: SuggestionCardProps) {
  const hours = Math.round(suggestion.suggestedDuration / 60);

  return (
    <div className="flex items-center justify-between rounded-md border bg-white p-3">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <p className="font-medium text-sm">{suggestion.projectName}</p>
          <Badge variant="outline" className="text-xs">
            {hours}h
          </Badge>
          <ConfidenceBadge confidence={suggestion.confidence} />
        </div>
        <p className="text-xs text-gray-600 mt-1">{suggestion.reason}</p>
      </div>
      <Button size="sm" onClick={onAccept}>
        Accept
      </Button>
    </div>
  );
}

function ConfidenceBadge({ confidence }: { confidence: number }) {
  const percentage = Math.round(confidence * 100);
  const color =
    percentage >= 75 ? 'text-green-600' : percentage >= 50 ? 'text-yellow-600' : 'text-gray-600';

  return (
    <span className={`text-xs ${color}`} title={`${percentage}% confidence`}>
      {percentage >= 75 ? '●●●' : percentage >= 50 ? '●●○' : '●○○'}
    </span>
  );
}
```

**Accept Suggestion Flow:**

```tsx
// apps/web/components/time-entry/time-entry-form.tsx
export function TimeEntryForm({ date }: TimeEntryFormProps) {
  const [formData, setFormData] = useState<TimeEntryFormData>({
    projectId: '',
    duration: 0,
    description: '',
  });

  const handleAcceptSuggestion = (suggestion: TimeEntrySuggestion) => {
    // Pre-fill form with suggestion
    setFormData({
      projectId: suggestion.projectId,
      duration: suggestion.suggestedDuration,
      description: ``, // User still needs to add description
    });

    // Track acceptance for metrics
    trackEvent('time_entry_suggestion_accepted', {
      projectId: suggestion.projectId,
      confidence: suggestion.confidence,
    });

    // Focus description field (only thing left to fill)
    descriptionInputRef.current?.focus();
  };

  return (
    <div>
      <SmartSuggestions date={date} onAccept={handleAcceptSuggestion} />

      <form onSubmit={handleSubmit}>
        <Select
          label="Project"
          value={formData.projectId}
          onChange={(val) => setFormData({ ...formData, projectId: val })}
        />
        <Input
          label="Duration (hours)"
          type="number"
          value={formData.duration / 60}
          onChange={(e) => setFormData({ ...formData, duration: Number(e.target.value) * 60 })}
        />
        <Textarea
          ref={descriptionInputRef}
          label="Description"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
        />
        <Button type="submit">Save Time Entry</Button>
      </form>
    </div>
  );
}
```

---

## Benefits

### User Benefits

1. **Time Savings:** Reduce time entry from 5 minutes to 30 seconds (accept + description)
2. **Reduced Errors:** System remembers patterns user might forget
3. **Better Compliance:** Suggestions remind users to log all hours
4. **Less Friction:** Less typing, clicking, scrolling
5. **Improved Accuracy:** Pattern-based suggestions more accurate than memory

### Business Benefits

1. **Increased Billable Hours:** Capture hours that would otherwise be forgotten
2. **Competitive Advantage:** Replicon's "ZeroTime" costs $29/user/month extra - we include it
3. **User Satisfaction:** Frustration relief = better reviews and lower churn
4. **Data Quality:** More accurate time tracking = better profitability insights
5. **Marketing Differentiation:** "AI-powered time tracking" is compelling

### Marketing Messages

- "Stop filling out timesheets manually. WellOS learns your patterns and suggests time entries automatically."
- "Save 15+ minutes every week with AI-powered time entry suggestions."
- "Our smart suggestions capture hours you'd otherwise forget - increasing your billable revenue."

---

## Metrics

### Target Metrics (Sprint 7)

- **60%+ of users** receive suggestions after 2 weeks
- **40%+ acceptance rate** for suggestions
- **15 minutes saved** per user per week (measured by accepted suggestions)
- **5% increase** in logged hours (capturing previously forgotten work)

### How to Measure

```typescript
// Track suggestion acceptance
export class TrackSuggestionAcceptanceCommand {
  constructor(
    public readonly userId: string,
    public readonly suggestionId: string,
    public readonly projectId: string,
    public readonly confidence: number,
    public readonly accepted: boolean, // true = accepted, false = dismissed
  ) {}
}

// Calculate time saved
export class CalculateTimeSavedQuery {
  async execute(userId: string): Promise<TimeSavedMetrics> {
    const acceptedSuggestions = await this.getAcceptedSuggestions(userId);

    // Estimate: each accepted suggestion saves 3 minutes (vs manual entry)
    const minutesSaved = acceptedSuggestions.length * 3;

    // Also track: hours logged via suggestions (revenue impact)
    const hoursCaptured = acceptedSuggestions.reduce((sum, s) => sum + s.duration / 60, 0);

    return {
      suggestionCount: acceptedSuggestions.length,
      minutesSaved,
      hoursCaptured,
      estimatedRevenue: hoursCaptured * 150, // Assume $150/hr avg rate
    };
  }
}
```

**Dashboard Widget:**

```tsx
// Show "Time Saved" on user dashboard
export function TimeSavedWidget() {
  const { data } = useTimeSavedQuery();

  if (!data) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm flex items-center gap-2">
          <SparklesIcon className="h-4 w-4 text-purple-600" />
          Smart Suggestions
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-purple-600">{data.minutesSaved} min</div>
        <p className="text-xs text-gray-600 mt-1">saved this week</p>
        <p className="text-xs text-gray-500 mt-2">
          You've captured {data.hoursCaptured}h worth ${data.estimatedRevenue.toLocaleString()} in
          billable time
        </p>
      </CardContent>
    </Card>
  );
}
```

---

## Phase 2: Enhanced Pattern Recognition (Future - Sprint 16+)

After MVP proves the concept, enhance with:

### Calendar Integration

```typescript
// Analyze calendar events to predict time entries
export class CalendarPatternAnalyzer {
  async analyzeMeetings(userId: string, date: Date): Promise<TimeEntrySuggestion[]> {
    const meetings = await this.calendarService.getMeetings(userId, date);

    const suggestions: TimeEntrySuggestion[] = [];
    for (const meeting of meetings) {
      // Match meeting title/attendees to projects
      const project = await this.matchMeetingToProject(meeting);
      if (project) {
        suggestions.push({
          projectId: project.id,
          projectName: project.name,
          suggestedDuration: meeting.durationMinutes,
          reason: `You had a meeting: "${meeting.title}"`,
          confidence: 0.9,
        });
      }
    }

    return suggestions;
  }
}
```

### Email/Communication Analysis

```typescript
// Analyze emails, Slack messages to infer project work
export class CommunicationPatternAnalyzer {
  async analyzeEmails(userId: string, date: Date): Promise<TimeEntrySuggestion[]> {
    const emails = await this.emailService.getEmails(userId, date);

    // Use keyword matching to identify project mentions
    // "Working on Website Redesign project..." → suggest time entry
    // This requires email integration (Phase 2)
  }
}
```

### Machine Learning (Real AI)

```typescript
// Train ML model on historical data
export class MachineLearningPatternAnalyzer {
  async predictTimeEntries(userId: string, date: Date): Promise<TimeEntrySuggestion[]> {
    // Features: dayOfWeek, recentProjects, typicalDuration, seasonality, etc.
    // Model: Random Forest or Gradient Boosting
    // Train on 6+ months of historical data
    // Output: probability distribution over projects

    const features = await this.extractFeatures(userId, date);
    const predictions = await this.mlModel.predict(features);

    return predictions
      .filter((p) => p.confidence > 0.5)
      .slice(0, 3)
      .map((p) => this.convertToSuggestion(p));
  }
}
```

---

## When to Use

Use this pattern when:

- ✅ Users perform repetitive data entry tasks
- ✅ Historical data exists to identify patterns
- ✅ Suggestions can save significant time
- ✅ Users have agency to accept/reject suggestions
- ✅ Pattern recognition is valuable (not just novelty)

Don't use this pattern when:

- ❌ Data is too random (no patterns exist)
- ❌ Suggestions would be more annoying than helpful
- ❌ Insufficient historical data (<2 weeks)
- ❌ High cost of false suggestions (critical data)

---

## Related Patterns

- **Machine Learning Pattern** (if using real ML in Phase 2+)
- **Cache-Aside Pattern** (cache analyzed patterns)
- **Scheduled Job Pattern** (daily pattern analysis)
- **Observer Pattern** (track suggestion acceptance)

---

## Anti-Patterns

### ❌ Overly Aggressive Suggestions

```typescript
// Showing 10+ suggestions → overwhelming user
// Showing low-confidence suggestions → annoying
// Auto-accepting suggestions → user loses control
// Result: User disables feature
```

### ❌ Black Box Suggestions

```typescript
// Suggestion: "Work on Project X for 4 hours"
// User: "Why? I don't understand."
// No explanation provided
// Result: User doesn't trust suggestions
```

### ❌ Stale Patterns

```typescript
// Analyzing 6-month-old data
// User switched teams 2 months ago
// Suggestions irrelevant to current work
// Result: User ignores suggestions
```

---

## Implementation Checklist

Sprint 7 (Polish & Enhancements):

- [ ] Create `TimeEntryPattern` value object
- [ ] Create `TimeEntrySuggestion` value object
- [ ] Implement `PatternAnalysisService` (frequency-based)
- [ ] Create `GetTimeEntrySuggestionsQuery` + handler
- [ ] Implement scheduled pattern analysis (daily cron)
- [ ] Cache analyzed patterns (Redis, 24-hour TTL)
- [ ] Build `SmartSuggestions` component (frontend)
- [ ] Add "Accept" button to pre-fill form
- [ ] Track suggestion acceptance (analytics)
- [ ] Build "Time Saved" dashboard widget
- [ ] Add confidence scoring (3-dot indicator)
- [ ] Ensure suggestions work on mobile
- [ ] Write unit tests for pattern detection algorithm
- [ ] Write E2E test: user receives suggestions → accepts → form pre-filled
- [ ] Document pattern analysis algorithm

---

## References

### Research

- PSA Market Research (January 2025): "Manual timesheet entry is top pain point"
- Competitive Analysis: Replicon's "ZeroTime" costs $29/user/month extra

### Code Locations

- Pattern Analysis: `apps/api/src/domain/time-entry/pattern-analysis.service.ts`
- Suggestions Query: `apps/api/src/application/time-entry/queries/get-time-entry-suggestions.handler.ts`
- Suggestions Component: `apps/web/components/time-entry/smart-suggestions.tsx`
- Time Saved Widget: `apps/web/components/dashboard/time-saved-widget.tsx`

### Related User Stories

- Sprint 7 - US-711: Smart Time Entry Suggestions (AI-Powered)

---

**Pattern Status:** ✅ Planned for Sprint 7 (Polish & Enhancements)

**Last Updated:** January 2025

---

## Conclusion

The Smart Suggestions Pattern addresses the #1 complaint in PSA platforms (manual timesheet entry) through simple statistical pattern recognition. By analyzing frequency and day-of-week patterns, we can suggest time entries with 40%+ acceptance rate, saving users 15+ minutes per week. This creates a clear competitive advantage ("AI-powered time tracking") without the complexity of machine learning models.

**Key Takeaway:** Start simple (frequency analysis), measure impact (acceptance rate, time saved), then enhance with ML if needed. Users value time savings more than fancy algorithms.
