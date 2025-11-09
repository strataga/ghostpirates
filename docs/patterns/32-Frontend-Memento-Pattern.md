# Frontend Memento Pattern

## Overview

The Memento Pattern in React applications provides the ability to capture and
restore the internal state of components without violating encapsulation. This
pattern is essential for implementing undo/redo functionality, draft saving,
form recovery, and state history management in complex oil & gas applications.

## Problem Statement

Complex frontend applications often need to:

- **Implement undo/redo** functionality for form editing
- **Save drafts** automatically to prevent data loss
- **Recover form state** after browser crashes or navigation
- **Track state history** for audit purposes
- **Restore previous versions** of data entry
- **Handle complex multi-step** form workflows

Traditional state management approaches lead to:

- **Tightly coupled** state management and business logic
- **Difficult implementation** of undo/redo functionality
- **Memory leaks** from storing too much history
- **Complex state restoration** logic scattered throughout components
- **Inconsistent behavior** across different forms

## Solution

Implement the Memento Pattern to create a clean separation between state
management and state persistence, enabling powerful undo/redo functionality and
draft management.

## Implementation

### Base Memento Interface

```typescript
// lib/memento/interfaces.ts
export interface Memento<T> {
  getState(): T;
  getTimestamp(): Date;
  getDescription(): string;
  getId(): string;
}

export interface Originator<T> {
  createMemento(): Memento<T>;
  restoreFromMemento(memento: Memento<T>): void;
  getCurrentState(): T;
}

export interface Caretaker<T> {
  saveMemento(memento: Memento<T>): void;
  getMemento(id: string): Memento<T> | null;
  getAllMementos(): Memento<T>[];
  undo(): Memento<T> | null;
  redo(): Memento<T> | null;
  canUndo(): boolean;
  canRedo(): boolean;
  clear(): void;
}

export interface MementoConfig {
  maxHistory: number;
  autoSave: boolean;
  autoSaveInterval: number;
  persistToStorage: boolean;
  storageKey: string;
  compressionEnabled: boolean;
}
```

### Memento Implementation

```typescript
// lib/memento/memento.ts
export class FormMemento<T> implements Memento<T> {
  private id: string;
  private timestamp: Date;
  private state: T;
  private description: string;

  constructor(state: T, description: string = '') {
    this.id = crypto.randomUUID();
    this.timestamp = new Date();
    this.state = this.deepClone(state);
    this.description = description;
  }

  getId(): string {
    return this.id;
  }

  getState(): T {
    return this.deepClone(this.state);
  }

  getTimestamp(): Date {
    return this.timestamp;
  }

  getDescription(): string {
    return this.description;
  }

  private deepClone(obj: T): T {
    if (obj === null || typeof obj !== 'object') {
      return obj;
    }

    if (obj instanceof Date) {
      return new Date(obj.getTime()) as unknown as T;
    }

    if (Array.isArray(obj)) {
      return obj.map((item) => this.deepClone(item)) as unknown as T;
    }

    const cloned = {} as T;
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        cloned[key] = this.deepClone(obj[key]);
      }
    }

    return cloned;
  }

  // Serialization for storage
  serialize(): string {
    return JSON.stringify({
      id: this.id,
      timestamp: this.timestamp.toISOString(),
      state: this.state,
      description: this.description,
    });
  }

  static deserialize<T>(data: string): FormMemento<T> {
    const parsed = JSON.parse(data);
    const memento = new FormMemento<T>(parsed.state, parsed.description);
    memento.id = parsed.id;
    memento.timestamp = new Date(parsed.timestamp);
    return memento;
  }
}
```

### Caretaker Implementation

```typescript
// lib/memento/caretaker.ts
export class MementoCaretaker<T> implements Caretaker<T> {
  private history: Memento<T>[] = [];
  private currentIndex: number = -1;
  private config: MementoConfig;

  constructor(config: Partial<MementoConfig> = {}) {
    this.config = {
      maxHistory: 50,
      autoSave: false,
      autoSaveInterval: 30000, // 30 seconds
      persistToStorage: false,
      storageKey: 'memento-history',
      compressionEnabled: false,
      ...config,
    };

    if (this.config.persistToStorage) {
      this.loadFromStorage();
    }
  }

  saveMemento(memento: Memento<T>): void {
    // Remove any future history if we're not at the end
    this.history = this.history.slice(0, this.currentIndex + 1);

    // Add new memento
    this.history.push(memento);
    this.currentIndex++;

    // Limit history size
    if (this.history.length > this.config.maxHistory) {
      this.history.shift();
      this.currentIndex--;
    }

    if (this.config.persistToStorage) {
      this.saveToStorage();
    }
  }

  getMemento(id: string): Memento<T> | null {
    return this.history.find((memento) => memento.getId() === id) || null;
  }

  getAllMementos(): Memento<T>[] {
    return [...this.history];
  }

  undo(): Memento<T> | null {
    if (this.canUndo()) {
      this.currentIndex--;
      const memento = this.history[this.currentIndex];

      if (this.config.persistToStorage) {
        this.saveToStorage();
      }

      return memento;
    }
    return null;
  }

  redo(): Memento<T> | null {
    if (this.canRedo()) {
      this.currentIndex++;
      const memento = this.history[this.currentIndex];

      if (this.config.persistToStorage) {
        this.saveToStorage();
      }

      return memento;
    }
    return null;
  }

  canUndo(): boolean {
    return this.currentIndex > 0;
  }

  canRedo(): boolean {
    return this.currentIndex < this.history.length - 1;
  }

  clear(): void {
    this.history = [];
    this.currentIndex = -1;

    if (this.config.persistToStorage) {
      localStorage.removeItem(this.config.storageKey);
    }
  }

  getCurrentMemento(): Memento<T> | null {
    if (this.currentIndex >= 0 && this.currentIndex < this.history.length) {
      return this.history[this.currentIndex];
    }
    return null;
  }

  getHistorySize(): number {
    return this.history.length;
  }

  private saveToStorage(): void {
    try {
      const serializedHistory = this.history.map((memento) =>
        (memento as FormMemento<T>).serialize(),
      );

      const data = {
        history: serializedHistory,
        currentIndex: this.currentIndex,
      };

      localStorage.setItem(this.config.storageKey, JSON.stringify(data));
    } catch (error) {
      console.warn('Failed to save memento history to storage:', error);
    }
  }

  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(this.config.storageKey);
      if (stored) {
        const data = JSON.parse(stored);
        this.history = data.history.map((serialized: string) =>
          FormMemento.deserialize<T>(serialized),
        );
        this.currentIndex = data.currentIndex;
      }
    } catch (error) {
      console.warn('Failed to load memento history from storage:', error);
      this.clear();
    }
  }
}
```

### Form State Manager

```typescript
// lib/memento/form-state-manager.ts
export class FormStateManager<T> implements Originator<T> {
  private state: T;
  private caretaker: MementoCaretaker<T>;
  private autoSaveTimer?: NodeJS.Timeout;
  private config: MementoConfig;

  constructor(initialState: T, config: Partial<MementoConfig> = {}) {
    this.state = initialState;
    this.config = {
      maxHistory: 50,
      autoSave: true,
      autoSaveInterval: 30000,
      persistToStorage: true,
      storageKey: `form-state-${Date.now()}`,
      compressionEnabled: false,
      ...config,
    };

    this.caretaker = new MementoCaretaker<T>(this.config);

    // Save initial state
    this.saveState('Initial state');

    // Setup auto-save
    if (this.config.autoSave) {
      this.startAutoSave();
    }
  }

  createMemento(): Memento<T> {
    return new FormMemento(this.state);
  }

  restoreFromMemento(memento: Memento<T>): void {
    this.state = memento.getState();
  }

  getCurrentState(): T {
    return this.state;
  }

  updateState(newState: Partial<T>, description?: string): void {
    this.state = { ...this.state, ...newState };

    if (description) {
      this.saveState(description);
    }
  }

  setState(newState: T, description?: string): void {
    this.state = newState;

    if (description) {
      this.saveState(description);
    }
  }

  saveState(description: string = ''): void {
    const memento = new FormMemento(this.state, description);
    this.caretaker.saveMemento(memento);
  }

  undo(): T | null {
    const memento = this.caretaker.undo();
    if (memento) {
      this.restoreFromMemento(memento);
      return this.state;
    }
    return null;
  }

  redo(): T | null {
    const memento = this.caretaker.redo();
    if (memento) {
      this.restoreFromMemento(memento);
      return this.state;
    }
    return null;
  }

  canUndo(): boolean {
    return this.caretaker.canUndo();
  }

  canRedo(): boolean {
    return this.caretaker.canRedo();
  }

  getHistory(): Array<{ id: string; timestamp: Date; description: string }> {
    return this.caretaker.getAllMementos().map((memento) => ({
      id: memento.getId(),
      timestamp: memento.getTimestamp(),
      description: memento.getDescription(),
    }));
  }

  restoreToPoint(mementoId: string): boolean {
    const memento = this.caretaker.getMemento(mementoId);
    if (memento) {
      this.restoreFromMemento(memento);
      return true;
    }
    return false;
  }

  clear(): void {
    this.caretaker.clear();
    this.stopAutoSave();
  }

  private startAutoSave(): void {
    this.autoSaveTimer = setInterval(() => {
      this.saveState('Auto-save');
    }, this.config.autoSaveInterval);
  }

  private stopAutoSave(): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = undefined;
    }
  }

  destroy(): void {
    this.stopAutoSave();
    this.clear();
  }
}
```

### React Hook Integration

```typescript
// hooks/use-form-history.ts
export function useFormHistory<T>(initialState: T, config: Partial<MementoConfig> = {}) {
  const [stateManager] = useState(() => new FormStateManager(initialState, config));

  const [formState, setFormState] = useState<T>(initialState);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  // Update local state when manager state changes
  useEffect(() => {
    const currentState = stateManager.getCurrentState();
    setFormState(currentState);
    setCanUndo(stateManager.canUndo());
    setCanRedo(stateManager.canRedo());
  }, [stateManager]);

  const updateState = useCallback(
    (newState: Partial<T> | T, description?: string) => {
      if (typeof newState === 'object' && newState !== null) {
        // Check if it's a partial update or full state
        const keys = Object.keys(newState);
        const isPartialUpdate = keys.some(
          (key) =>
            !(key in initialState) ||
            typeof (newState as any)[key] !== typeof (initialState as any)[key],
        );

        if (isPartialUpdate) {
          stateManager.updateState(newState as Partial<T>, description);
        } else {
          stateManager.setState(newState as T, description);
        }
      } else {
        stateManager.setState(newState as T, description);
      }

      setFormState(stateManager.getCurrentState());
      setCanUndo(stateManager.canUndo());
      setCanRedo(stateManager.canRedo());
    },
    [stateManager, initialState],
  );

  const undo = useCallback(() => {
    const previousState = stateManager.undo();
    if (previousState) {
      setFormState(previousState);
      setCanUndo(stateManager.canUndo());
      setCanRedo(stateManager.canRedo());
      return previousState;
    }
    return null;
  }, [stateManager]);

  const redo = useCallback(() => {
    const nextState = stateManager.redo();
    if (nextState) {
      setFormState(nextState);
      setCanUndo(stateManager.canUndo());
      setCanRedo(stateManager.canRedo());
      return nextState;
    }
    return null;
  }, [stateManager]);

  const saveState = useCallback(
    (description: string) => {
      stateManager.saveState(description);
      setCanUndo(stateManager.canUndo());
      setCanRedo(stateManager.canRedo());
    },
    [stateManager],
  );

  const getHistory = useCallback(() => {
    return stateManager.getHistory();
  }, [stateManager]);

  const restoreToPoint = useCallback(
    (mementoId: string) => {
      const success = stateManager.restoreToPoint(mementoId);
      if (success) {
        setFormState(stateManager.getCurrentState());
        setCanUndo(stateManager.canUndo());
        setCanRedo(stateManager.canRedo());
      }
      return success;
    },
    [stateManager],
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stateManager.destroy();
    };
  }, [stateManager]);

  return {
    formState,
    updateState,
    undo,
    redo,
    canUndo,
    canRedo,
    saveState,
    getHistory,
    restoreToPoint,
  };
}

// hooks/use-draft-manager.ts
export function useDraftManager<T>(
  formId: string,
  initialState: T,
  autoSaveInterval: number = 30000,
) {
  const { formState, updateState, saveState, getHistory } = useFormHistory(initialState, {
    autoSave: true,
    autoSaveInterval,
    persistToStorage: true,
    storageKey: `draft-${formId}`,
    maxHistory: 10, // Limit drafts to save storage space
  });

  const [isDraft, setIsDraft] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

  useEffect(() => {
    // Check if there's a saved draft
    const history = getHistory();
    if (history.length > 1) {
      // More than just initial state
      setIsDraft(true);
      setLastSaved(history[history.length - 1].timestamp);
    }
  }, [getHistory]);

  const saveDraft = useCallback(
    (description: string = 'Draft saved') => {
      saveState(description);
      setIsDraft(true);
      setLastSaved(new Date());
    },
    [saveState],
  );

  const clearDraft = useCallback(() => {
    // This would clear the storage
    localStorage.removeItem(`draft-${formId}`);
    setIsDraft(false);
    setLastSaved(null);
  }, [formId]);

  return {
    formState,
    updateState,
    saveDraft,
    clearDraft,
    isDraft,
    lastSaved,
  };
}
```

### Component Usage

```typescript
// components/forms/well-completion-form.tsx
interface WellCompletionFormData {
  apiNumber: string;
  wellName: string;
  wellType: string;
  totalDepth: number;
  spudDate: string;
  completionDate: string;
  // ... other fields
}

export function WellCompletionForm({ wellId, onSubmit }: Props) {
  const initialState: WellCompletionFormData = {
    apiNumber: '',
    wellName: '',
    wellType: 'vertical',
    totalDepth: 0,
    spudDate: '',
    completionDate: '',
  };

  const {
    formState,
    updateState,
    undo,
    redo,
    canUndo,
    canRedo,
    saveState,
    getHistory,
  } = useFormHistory(initialState, {
    maxHistory: 100,
    autoSave: true,
    autoSaveInterval: 30000,
    persistToStorage: true,
    storageKey: `well-completion-${wellId}`,
  });

  const [showHistory, setShowHistory] = useState(false);

  const handleFieldChange = (field: keyof WellCompletionFormData, value: any) => {
    updateState({ [field]: value }, `Updated ${field}`);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    try {
      await onSubmit(formState);
      saveState('Form submitted successfully');
      toast.success('Well completion form submitted successfully');
    } catch (error) {
      toast.error('Failed to submit form');
    }
  };

  const handleUndo = () => {
    const previousState = undo();
    if (previousState) {
      toast.success('Undid last change');
    }
  };

  const handleRedo = () => {
    const nextState = redo();
    if (nextState) {
      toast.success('Redid last change');
    }
  };

  return (
    <div className="space-y-6">
      {/* Undo/Redo Controls */}
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleUndo}
          disabled={!canUndo}
        >
          <Undo className="h-4 w-4 mr-2" />
          Undo
        </Button>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleRedo}
          disabled={!canRedo}
        >
          <Redo className="h-4 w-4 mr-2" />
          Redo
        </Button>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => setShowHistory(!showHistory)}
        >
          <History className="h-4 w-4 mr-2" />
          History
        </Button>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => saveState('Manual save')}
        >
          <Save className="h-4 w-4 mr-2" />
          Save Draft
        </Button>
      </div>

      {/* History Panel */}
      {showHistory && (
        <HistoryPanel
          history={getHistory()}
          onRestoreToPoint={(mementoId) => {
            // Implementation would restore to specific point
            console.log('Restore to:', mementoId);
          }}
        />
      )}

      {/* Form Fields */}
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Label htmlFor="apiNumber">API Number</Label>
          <Input
            id="apiNumber"
            value={formState.apiNumber}
            onChange={(e) => handleFieldChange('apiNumber', e.target.value)}
            placeholder="Enter 14-digit API number"
          />
        </div>

        <div>
          <Label htmlFor="wellName">Well Name</Label>
          <Input
            id="wellName"
            value={formState.wellName}
            onChange={(e) => handleFieldChange('wellName', e.target.value)}
            placeholder="Enter well name"
          />
        </div>

        <div>
          <Label htmlFor="wellType">Well Type</Label>
          <Select
            value={formState.wellType}
            onValueChange={(value) => handleFieldChange('wellType', value)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="vertical">Vertical</SelectItem>
              <SelectItem value="horizontal">Horizontal</SelectItem>
              <SelectItem value="directional">Directional</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* More form fields... */}

        <div className="flex gap-2">
          <Button type="submit">
            Submit Form
          </Button>
          <Button type="button" variant="outline">
            Save as Draft
          </Button>
        </div>
      </form>
    </div>
  );
}

// components/history-panel.tsx
interface HistoryPanelProps {
  history: Array<{ id: string; timestamp: Date; description: string }>;
  onRestoreToPoint: (mementoId: string) => void;
}

function HistoryPanel({ history, onRestoreToPoint }: HistoryPanelProps) {
  return (
    <div className="border rounded-lg p-4 bg-muted/50">
      <h3 className="font-semibold mb-3">Form History</h3>
      <div className="space-y-2 max-h-60 overflow-y-auto">
        {history.map((item) => (
          <div
            key={item.id}
            className="flex items-center justify-between p-2 bg-background rounded border"
          >
            <div>
              <p className="text-sm font-medium">{item.description}</p>
              <p className="text-xs text-muted-foreground">
                {item.timestamp.toLocaleString()}
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => onRestoreToPoint(item.id)}
            >
              Restore
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}
```

## Benefits

### 1. **Undo/Redo Functionality**

- Complete undo/redo support for complex forms
- Granular control over what changes are saved
- User-friendly state restoration

### 2. **Draft Management**

- Automatic draft saving to prevent data loss
- Recovery from browser crashes or accidental navigation
- Multiple draft versions with timestamps

### 3. **State History**

- Complete audit trail of form changes
- Ability to restore to any previous state
- Useful for debugging and user support

### 4. **Memory Management**

- Configurable history limits to prevent memory leaks
- Efficient state cloning and storage
- Optional compression for large forms

## Best Practices

### 1. **Memory Management**

```typescript
// ✅ Good: Limit history size
const config = {
  maxHistory: 50,
  autoSave: true,
  autoSaveInterval: 30000,
};

// ❌ Bad: Unlimited history
const config = {
  maxHistory: Infinity, // Memory leak risk
};
```

### 2. **Meaningful Descriptions**

```typescript
// ✅ Good: Descriptive memento descriptions
updateState({ wellType: 'horizontal' }, 'Changed well type to horizontal');

// ❌ Bad: Generic descriptions
updateState({ wellType: 'horizontal' }, 'Updated field');
```

### 3. **Cleanup**

```typescript
// ✅ Good: Cleanup on unmount
useEffect(() => {
  return () => {
    stateManager.destroy();
  };
}, [stateManager]);
```

## Testing

```typescript
// __tests__/memento/form-state-manager.test.ts
describe('FormStateManager', () => {
  let manager: FormStateManager<{ name: string; age: number }>;

  beforeEach(() => {
    manager = new FormStateManager({ name: '', age: 0 });
  });

  afterEach(() => {
    manager.destroy();
  });

  it('should save and restore state', () => {
    manager.updateState({ name: 'John' }, 'Set name');
    manager.updateState({ age: 30 }, 'Set age');

    expect(manager.getCurrentState()).toEqual({ name: 'John', age: 30 });

    const previousState = manager.undo();
    expect(previousState).toEqual({ name: 'John', age: 0 });

    const nextState = manager.redo();
    expect(nextState).toEqual({ name: 'John', age: 30 });
  });

  it('should limit history size', () => {
    const smallManager = new FormStateManager({ count: 0 }, { maxHistory: 3 });

    for (let i = 1; i <= 5; i++) {
      smallManager.updateState({ count: i }, `Set to ${i}`);
    }

    expect(smallManager.getHistory().length).toBe(3);
    smallManager.destroy();
  });
});
```

The Memento Pattern provides a robust foundation for implementing undo/redo
functionality and draft management in complex React applications, ensuring users
never lose their work and can easily recover from mistakes.
