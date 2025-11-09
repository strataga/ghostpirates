# Pattern 62: Drag and Drop Visual Feedback

**Category**: User Interface Patterns
**Status**: Active
**Last Updated**: 2025-10-17

## Overview

When implementing drag-and-drop functionality, especially for kanban boards or sortable lists, it's critical to provide clear visual feedback to users about where they can drop items. This pattern uses the `@dnd-kit` library's `useDroppable` hook to show visual drop targets during drag operations.

## Problem

Without visual feedback during drag operations:

- Users don't know if they're hovering over a valid drop zone
- The drag-and-drop interaction feels unresponsive and confusing
- Users may drop items in wrong locations or give up on the feature
- Accessibility and UX suffer significantly

## Solution

Use the `isOver` state from `useDroppable` to apply visual styling when a draggable item is hovering over a droppable container.

### Implementation

```tsx
import { useDroppable } from '@dnd-kit/core';

export function DroppableColumn({ id, children }) {
  // Extract isOver state from useDroppable
  const { setNodeRef, isOver } = useDroppable({
    id,
  });

  return (
    <div
      ref={setNodeRef}
      className={`
        min-h-[400px]
        rounded-lg
        border-2
        border-dashed
        p-3
        transition-all
        ${isOver ? 'border-primary border-solid bg-primary/5 ring-2 ring-primary/20' : ''}
      `}
    >
      {children}
    </div>
  );
}
```

### Visual Feedback Styles

When `isOver` is `true`, apply these visual changes:

1. **Border Changes**:
   - Change from `border-dashed` to `border-solid`
   - Change border color to `border-primary`

2. **Background Highlight**:
   - Add subtle background tint: `bg-primary/5`

3. **Ring/Glow Effect**:
   - Add outer ring for emphasis: `ring-2 ring-primary/20`

4. **Smooth Transitions**:
   - Use `transition-all` for smooth visual changes

## Key Principles

### 1. **Immediate Feedback**

Visual changes should occur instantly when hovering over a drop zone.

### 2. **Clear Contrast**

The drop target should be visually distinct from the default state:

```tsx
// Default state
border-2 border-dashed

// Drop target state
border-2 border-solid border-primary bg-primary/5 ring-2 ring-primary/20
```

### 3. **Subtle but Noticeable**

Use opacity levels that are visible but not overwhelming:

- Background: `/5` (20% opacity)
- Ring: `/20` (5% opacity)

### 4. **Smooth Transitions**

Always include `transition-all` to prevent jarring visual changes.

## Related Patterns

- **DragOverlay**: Shows a ghost of the dragged item
- **Sortable Context**: Manages sortable items within a container
- **Collision Detection**: Determines which drop zone is active

## Example: Kanban Board

```tsx
export function KanbanColumn({ status, tasks }: KanbanColumnProps) {
  const { setNodeRef, isOver } = useDroppable({
    id: status, // Column status (TODO, IN_PROGRESS, DONE)
  });

  return (
    <div className="space-y-3">
      <h3>Column Title</h3>

      {/* Drop zone with visual feedback */}
      <div
        ref={setNodeRef}
        className={`
          space-y-2
          min-h-[400px]
          rounded-lg
          border-2
          border-dashed
          p-3
          transition-all
          ${isOver ? 'border-primary border-solid bg-primary/5 ring-2 ring-primary/20' : ''}
        `}
      >
        <SortableContext items={taskIds}>
          {tasks.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))}
        </SortableContext>
      </div>
    </div>
  );
}
```

## Debug Logging

Add console logging to verify drop detection:

```tsx
const { setNodeRef, isOver } = useDroppable({
  id: status,
});

console.log('[DEBUG Column] Status:', status, 'isOver:', isOver);
```

## Common Pitfalls

### ❌ Forgetting to Extract `isOver`

```tsx
// Wrong - isOver not extracted
const { setNodeRef } = useDroppable({ id });
```

```tsx
// Correct
const { setNodeRef, isOver } = useDroppable({ id });
```

### ❌ No Transition Effect

```tsx
// Wrong - jarring visual changes
<div className={isOver ? 'border-solid' : 'border-dashed'} />
```

```tsx
// Correct - smooth transitions
<div className={`transition-all ${isOver ? 'border-solid' : 'border-dashed'}`} />
```

### ❌ Overly Aggressive Styling

```tsx
// Wrong - too intense
bg-primary ring-8 border-4
```

```tsx
// Correct - subtle but clear
bg-primary/5 ring-2 ring-primary/20 border-2
```

## Testing

Verify visual feedback works correctly:

1. **Drag Start**: Pick up an item
2. **Hover Different Zones**: Move over various drop zones
3. **Visual Change**: Confirm border/background/ring appear
4. **Hover Leave**: Confirm styling reverts when leaving
5. **Drop**: Confirm item moves to correct location

## Browser Compatibility

This pattern works across all modern browsers that support:

- CSS transitions
- Flexbox/Grid layouts
- Pointer events

## Accessibility

The visual feedback pattern improves accessibility by:

- Providing clear visual cues for sighted users
- Working alongside `aria-label` attributes for screen readers
- Supporting keyboard navigation when combined with proper focus states

## Performance

The `isOver` state updates are efficient because:

- React re-renders only the affected droppable component
- CSS transitions are hardware-accelerated
- No JavaScript animations required

## Optimistic Updates

For the best user experience, combine visual feedback with optimistic updates so the UI reflects changes immediately:

```tsx
// In your mutation hook
export function useUpdateTaskStatusMutation(options) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, data }) => {
      return taskRepository.updateTaskStatus(id, data);
    },
    onMutate: async (variables) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: projectTasksQueryKeys.byProject(options.projectId),
      });

      // Snapshot previous state for rollback
      const previousTasks = queryClient.getQueryData(
        projectTasksQueryKeys.byProject(options.projectId),
      );

      // Optimistically update the cache
      queryClient.setQueryData(projectTasksQueryKeys.byProject(options.projectId), (old) => {
        // Move task between columns immediately in cache
        // ... optimistic update logic
        return newState;
      });

      return { previousTasks };
    },
    onError: (error, variables, context) => {
      // Rollback on error
      if (context?.previousTasks) {
        queryClient.setQueryData(
          projectTasksQueryKeys.byProject(options.projectId),
          context.previousTasks,
        );
      }
    },
    onSuccess: () => {
      // Refetch to sync with server state
      queryClient.invalidateQueries({
        queryKey: projectTasksQueryKeys.byProject(options.projectId),
      });
    },
  });
}
```

**Benefits of Optimistic Updates:**

- Instant UI feedback (no waiting for server response)
- Better perceived performance
- Automatic rollback on error
- Server sync after success

**See Implementation:**

- Visual Feedback: `apps/web/components/task/kanban-column.tsx:48-73`
- Optimistic Updates: `apps/web/hooks/task/use-update-task-status-mutation.ts:34-103`
- Reorder Updates: `apps/web/hooks/task/use-reorder-tasks-mutation.ts:33-83`

## References

- [@dnd-kit Documentation](https://docs.dndkit.com/)
- [useDroppable Hook](https://docs.dndkit.com/api-documentation/droppable/usedroppable)
- [React Query Optimistic Updates](https://tanstack.com/query/latest/docs/react/guides/optimistic-updates)
- Catalyst Implementation: `apps/web/components/task/kanban-column.tsx:48-73`
