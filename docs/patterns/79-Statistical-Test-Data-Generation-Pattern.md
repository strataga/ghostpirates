# 79 - Statistical Test Data Generation Pattern

**Category:** Development Productivity & Testing
**Complexity:** Medium
**Last Updated:** October 29, 2025

---

## Problem Statement

Manual test data entry during development is tedious, slow, and often fails to test edge cases:

- **Time sink** - Developers waste hours manually filling 40+ field forms during testing
- **Incomplete testing** - Manual data is usually "happy path" only (missing edge cases)
- **Inconsistent data** - Different developers test with different values
- **Validation blind spots** - Out-of-range values, boundary conditions, and validation rules go untested
- **Context switching** - Stopping to think "what's a realistic water cut percentage?" breaks flow

Traditional approaches either use hardcoded fixtures (no randomness) or fully random data (not domain-realistic), neither of which effectively stress-tests validation logic.

## Solution Overview

Implement probability-based test data generation that creates **statistically realistic values with intentional outliers** to simultaneously test happy paths AND edge cases in a single button press.

**Core Principles:**

1. **Probability-Based Outliers** - Use `Math.random() > threshold` for X% chance of out-of-range values
2. **Domain-Realistic Ranges** - Generate values that match real-world data distributions
3. **Context-Aware Generation** - Populate fields based on selected entity type (e.g., well type)
4. **Conditional Field Reset** - Clear irrelevant fields before populating context-specific ones
5. **Manual Trigger Only** - Dev-only button (never auto-fill) to maintain user control
6. **Visual Affordance** - Warning colors (amber/yellow) signal "this is test data, not production"

## When to Use

✅ **Use this pattern when:**

- Complex forms with 10+ fields make manual testing impractical
- Your validation logic needs stress-testing (boundary values, outliers, malformed data)
- Different entity types require different field sets (context-aware forms)
- Development team needs consistent test data across environments
- You want to test both happy paths AND edge cases simultaneously

❌ **Don't use this pattern when:**

- Simple forms (≤5 fields) where manual entry is quick
- Production environments (test data generators are dev/staging only)
- Forms require specific, reproducible test data (use fixtures instead)
- External systems dictate data format (use mock APIs instead)

## Implementation

### Example: Oil & Gas Field Data Entry Form (40+ Fields, 5 Well Types)

#### Step 1: Define Probability Thresholds

```typescript
// apps/mobile/app/(tabs)/entry.tsx

// Probability configuration
const OUTLIER_PROBABILITY = 0.3; // 30% chance of out-of-range values
const HIGH_VALUE_PROBABILITY = 0.7; // 70% threshold for "too high" values
const ABNORMAL_PROBABILITY = 0.8; // 80% threshold for abnormal readings

// Domain-specific ranges (Permian Basin oil wells)
const RANGES = {
  productionVolume: { normal: [50, 200], outlier: [0, 1000] },
  gasVolume: { normal: [500, 2000], outlier: [0, 10000] },
  pressure: { normal: [1000, 3000], outlier: [0, 5000] },
  temperature: { normal: [150, 250], outlier: [0, 500] },
  waterCut: { normal: [0, 50], outlier: [0, 100] },
  bsw: { normal: [0, 1], outlier: [0, 12] }, // >1% violates sales requirement
  gor: { normal: [1000, 6000], outlier: [6000, 15000] }, // >6000 triggers high GOR alert
};
```

#### Step 2: Create Test Data Generation Function

```typescript
const fillWithTestData = async () => {
  // Guard: Require wells data
  if (wells.length === 0) {
    Alert.alert('No Wells', 'Please sync wells data first');
    return;
  }

  // Haptic feedback (mobile UX)
  if (Platform.OS !== 'web') {
    await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
  }

  // 1. Select random well (enables context-aware fields)
  const randomWell = wells[Math.floor(Math.random() * wells.length)];
  handleWellSelection(randomWell.label);

  // 2. Generate production data with 30% outlier probability
  setProductionVolume(
    String(
      Math.random() > HIGH_VALUE_PROBABILITY
        ? randomInRange(RANGES.productionVolume.outlier)
        : randomInRange(RANGES.productionVolume.normal)
    )
  );

  setGasVolume(
    String(
      Math.random() > HIGH_VALUE_PROBABILITY
        ? randomInRange(RANGES.gasVolume.outlier)
        : randomInRange(RANGES.gasVolume.normal)
    )
  );

  // Water cut with 40% chance of exceeding normal range
  setWaterCut(
    String(
      (Math.random() > 0.6
        ? Math.random() * 100
        : Math.random() * 50
      ).toFixed(2)
    )
  );

  // 3. Generate metrics that violate business rules (tests validation)
  setBsw(String((Math.random() * 12).toFixed(2))); // Some > 1% (fails sales spec)
  setGor(String(Math.floor(Math.random() * 15000))); // Some > 6000 (high GOR warning)

  // Casing pressure: 80% chance of zero (normal), 20% abnormal
  setCasingPressure(
    String(
      Math.random() > ABNORMAL_PROBABILITY
        ? Math.floor(Math.random() * 500)
        : 0
    )
  );

  // 4. Generate operational data with conditional logic
  const statuses: Array<'operating' | 'down' | 'maintenance'> = [
    'operating',
    'down',
    'maintenance',
  ];
  const randomStatus = statuses[Math.floor(Math.random() * statuses.length)];
  setPumpStatus(randomStatus);

  // Conditionally populate downtime fields
  if (randomStatus === 'down' || randomStatus === 'maintenance') {
    setDowntimeHours(String(Math.floor(Math.random() * 72)));
    const reasons = [
      'Pump failure',
      'Rod parted',
      'Electrical issue',
      'Scheduled maintenance',
      'Weather delay',
    ];
    setDowntimeReason(reasons[Math.floor(Math.random() * reasons.length)]);
  } else {
    setDowntimeHours('');
    setDowntimeReason('');
  }

  // 5. Context-aware equipment fields (well-type-specific)
  const well = allWellsData.find((w) => w.id === randomWell.value);
  if (well) {
    // CRITICAL: Reset ALL equipment fields first (prevents field leakage)
    resetAllEquipmentFields();

    // Populate only relevant fields for this well type
    switch (well.wellType) {
      case 'beam-pump':
        setPumpRuntime(String(Math.floor(Math.random() * 24)));
        setStrokesPerMinute(String(Math.floor(Math.random() * 30)));
        setStrokeLength(String(Math.floor(Math.random() * 120)));
        setEngineHours(String(Math.floor(Math.random() * 25000)));
        setEngineTemp(String(Math.floor(Math.random() * 350)));
        break;

      case 'pcp':
        setMotorAmps(String((Math.random() * 150).toFixed(1)));
        setMotorVoltage(String(Math.floor(Math.random() * 600)));
        setMotorTemp(String(Math.floor(Math.random() * 300)));
        setMotorRpm(String(Math.floor(Math.random() * 500)));
        setMotorRunningHours(String(Math.floor(Math.random() * 15000)));
        setDischargePressure(String(Math.floor(Math.random() * 2000)));
        break;

      case 'gas-lift':
        setGasInjectionVolume(String(Math.floor(Math.random() * 600)));
        setInjectionPressure(String(Math.floor(Math.random() * 1500)));
        setBackpressure(String(Math.floor(Math.random() * 1000)));
        const orifices = ['1/4', '3/8', '1/2', '5/8', '3/4'];
        setOrificeSize(orifices[Math.floor(Math.random() * orifices.length)]);
        break;

      // ... other well types
    }
  }

  // 6. Check all safety checklist items (validates bulk toggle logic)
  setChecklist({
    pumpOperating: true,
    noLeaks: true,
    gaugesWorking: true,
    safetyEquipment: true,
    tankLevelsChecked: true,
    separatorOperating: true,
    // ... all 15 checklist items
  });

  // 7. Add timestamped notes
  setNotes(
    `Test entry for ${randomWell.label} - Generated at ${new Date().toLocaleTimeString()}`
  );

  // Success feedback
  toast.success('Test data generated! (Some values intentionally out of range)', {
    duration: 4000,
  });
  if (Platform.OS !== 'web') {
    await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
  }
};
```

**Helper Function: Random Value in Range**

```typescript
const randomInRange = (range: [number, number]): number => {
  const [min, max] = range;
  return Math.floor(Math.random() * (max - min + 1)) + min;
};

const resetAllEquipmentFields = () => {
  // Reset all well-type-specific fields to prevent leakage
  setPumpRuntime('');
  setStrokesPerMinute('');
  setStrokeLength('');
  setEngineHours('');
  setEngineTemp('');
  setMotorAmps('');
  setMotorVoltage('');
  setMotorTemp('');
  setMotorRpm('');
  setMotorRunningHours('');
  setDischargePressure('');
  setGasInjectionVolume('');
  setInjectionPressure('');
  setBackpressure('');
  setOrificeSize('');
  setCycleTime('');
  setSurfacePressure('');
  setPlungerArrival('');
};
```

#### Step 3: Add Development-Only UI Button

```typescript
// Render test data button (dev mode only)
{
  __DEV__ && Platform.OS !== 'web' && (
    <View style={styles.testDataButtonContainer}>
      <TouchableOpacity style={styles.testDataButton} onPress={fillWithTestData}>
        <Text style={styles.testDataIcon}>🎲</Text>
        <Text style={styles.testDataButtonText}>Fill with Test Data</Text>
      </TouchableOpacity>
    </View>
  );
}
```

**Visual Styling (Warning Colors for Dev Tools):**

```typescript
const styles = StyleSheet.create({
  testDataButtonContainer: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#FEF3C7', // Light yellow (warning)
    borderRadius: 8,
    marginBottom: 16,
    borderWidth: 2,
    borderColor: '#FDE047', // Yellow accent
    borderStyle: 'dashed', // "This is temporary/dev-only"
  },
  testDataButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FBBF24', // Amber (caution)
    paddingVertical: 14,
    paddingHorizontal: 20,
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 3,
    elevation: 3,
  },
  testDataIcon: {
    fontSize: 20,
    marginRight: 8,
  },
  testDataButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#78350F', // Dark amber text
  },
});
```

#### Step 4: Guard Against Production Usage

```typescript
// Compile-time check (TypeScript/ESLint)
if (__DEV__) {
  // Test data generation code
}

// Runtime check (defense-in-depth)
const fillWithTestData = async () => {
  if (process.env.NODE_ENV === 'production') {
    console.error('[Security] Test data generation disabled in production');
    return;
  }
  // ... generation logic
};
```

## Advanced Variations

### Variation 1: Weighted Probability Distributions

Use more sophisticated distributions for realistic data:

```typescript
// Normal distribution (bell curve) for common metrics
const gaussianRandom = (mean: number, stdDev: number): number => {
  const u1 = Math.random();
  const u2 = Math.random();
  const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2.0 * Math.PI * u2);
  return Math.round(z0 * stdDev + mean);
};

// Example: Production volume follows normal distribution
setProductionVolume(
  String(Math.max(0, gaussianRandom(150, 50))) // Mean: 150 bbl/day, StdDev: 50
);
```

### Variation 2: Correlated Field Generation

Generate related fields with realistic correlations:

```typescript
// High gas volume → high GOR (correlated)
const gasVolume = randomInRange([500, 2000]);
setGasVolume(String(gasVolume));

// GOR = gas volume / oil volume (realistic ratio)
const productionVolume = randomInRange([50, 200]);
const gor = Math.floor((gasVolume * 1000) / productionVolume);
setGor(String(gor));
```

### Variation 3: Deterministic Seeded Random (Reproducible Tests)

Use seeded RNG for consistent test data across runs:

```typescript
// Install: npm install seedrandom
import seedrandom from 'seedrandom';

const fillWithTestData = async (seed: string = 'default-seed') => {
  const rng = seedrandom(seed);

  // All random calls use seeded RNG
  setProductionVolume(String(Math.floor(rng() * 200)));
  setGasVolume(String(Math.floor(rng() * 2000)));
  // ...
};
```

### Variation 4: Test Data Presets

Provide named presets for specific test scenarios:

```typescript
const TEST_PRESETS = {
  'happy-path': { outlierProbability: 0, allFieldsValid: true },
  'edge-cases': { outlierProbability: 1.0, allFieldsValid: false },
  'mixed': { outlierProbability: 0.3, allFieldsValid: false }, // Default
  'high-production': { baseMultiplier: 2.0, outlierProbability: 0.1 },
  'pump-failure': { pumpStatus: 'down', downtimeHours: 24 },
};

const fillWithTestData = async (preset: keyof typeof TEST_PRESETS = 'mixed') => {
  const config = TEST_PRESETS[preset];
  // Use config to customize generation
};
```

## Trade-offs

### Advantages ✅

1. **Developer Productivity** - 40+ fields filled in 1 tap vs. 5 minutes manual entry
2. **Edge Case Coverage** - Automatically tests outliers, boundary values, validation rules
3. **Consistent Testing** - All developers use same data distributions
4. **Realistic Data** - Domain-specific ranges match real-world values
5. **Context-Aware** - Adapts to entity types (well types, pump configs, etc.)
6. **Validation Stress-Test** - Intentional rule violations expose weak validation

### Disadvantages ❌

1. **Non-Deterministic** - Random data makes bug reproduction harder (mitigate with seeded RNG)
2. **Maintenance Overhead** - Ranges must be updated as domain knowledge evolves
3. **False Confidence** - Passing random tests doesn't guarantee all edge cases covered
4. **Complexity** - Context-aware generation adds 100+ lines of code
5. **Production Risk** - Must guard against accidental production usage

## Testing Strategies

### Unit Tests for Generator Logic

```typescript
describe('fillWithTestData', () => {
  it('should generate values within specified ranges', () => {
    const value = randomInRange([100, 200]);
    expect(value).toBeGreaterThanOrEqual(100);
    expect(value).toBeLessThanOrEqual(200);
  });

  it('should generate outliers X% of the time', () => {
    const iterations = 10000;
    let outlierCount = 0;

    for (let i = 0; i < iterations; i++) {
      const value = Math.random() > 0.7 ? 1000 : 100;
      if (value === 1000) outlierCount++;
    }

    const outlierRate = outlierCount / iterations;
    expect(outlierRate).toBeCloseTo(0.3, 1); // ~30% outliers
  });

  it('should reset equipment fields before populating', () => {
    // Set beam-pump fields
    setPumpRuntime('24');
    setStrokesPerMinute('20');

    // Switch to gas-lift well
    fillWithTestData(); // Should clear beam-pump fields

    expect(getPumpRuntime()).toBe('');
    expect(getStrokesPerMinute()).toBe('');
  });
});
```

### Integration Tests for Form Validation

```typescript
describe('Test Data Validation', () => {
  it('should trigger validation warnings for outlier values', async () => {
    await fillWithTestData();

    // Some generated values should violate rules
    const bswValue = parseFloat(getBsw());
    if (bswValue > 1.0) {
      expect(screen.getByText(/exceeds sales specification/i)).toBeInTheDocument();
    }
  });

  it('should populate context-specific fields only', async () => {
    // Select beam-pump well
    selectWell('Well #1 (beam-pump)');
    await fillWithTestData();

    // Beam-pump fields should be populated
    expect(getPumpRuntime()).not.toBe('');
    expect(getStrokesPerMinute()).not.toBe('');

    // Gas-lift fields should be empty
    expect(getGasInjectionVolume()).toBe('');
    expect(getInjectionPressure()).toBe('');
  });
});
```

## Related Patterns

- **[52 - User-Friendly Error Handling Pattern](./52-User-Friendly-Error-Handling-Pattern.md)** - Display validation errors from generated outlier values
- **[77 - Form Field Auto-Generation Pattern](./77-Form-Field-Auto-Generation-Pattern.md)** - Auto-populate derived fields
- **[04 - Repository Pattern](./04-Repository-Pattern.md)** - Save/load generated test data

## Real-World Examples in WellOS

### 1. Mobile Field Data Entry (40+ Fields, 5 Well Types)

**File:** `apps/mobile/app/(tabs)/entry.tsx`

**Statistics:**

- **Fields Populated:** 40+ (production volumes, pressures, temperatures, equipment metrics)
- **Well Types:** 5 (beam-pump, PCP, submersible, gas-lift, plunger-lift)
- **Outlier Probability:** 30% for production data, 20% for abnormal pressures
- **Checklist Items:** 15 safety checks (all marked complete)

**Validation Scenarios Tested:**

```
✅ Happy Path (70%):
   - Production: 50-200 bbl/day
   - Pressure: 1000-3000 psi
   - BS&W: 0-1%
   - GOR: 1000-6000

⚠️ Outliers (30%):
   - Production: 0 or 500+ bbl/day (triggers alert)
   - BS&W: >1% (violates sales spec)
   - GOR: >6000 (high gas warning)
   - Casing pressure: >0 (abnormal)
```

### 2. Admin Portal - Tenant Creation

**Future Implementation:**

```typescript
const fillWithTestTenant = () => {
  setCompanyName(generateCompanyName()); // "ACME Oil & Gas #47"
  setSubscriptionTier(randomEnum(['STARTER', 'PROFESSIONAL', 'ENTERPRISE']));
  setTrialDays(Math.random() > 0.5 ? 30 : 0); // 50% get trials
  setContactEmail(generateEmail()); // "admin@acme-oil-gas-47.com"
};
```

### 3. Backend API Testing - Bulk Entry Generation

```typescript
// Generate 100 field entries for load testing
const generateBulkEntries = (count: number) => {
  const entries = [];
  for (let i = 0; i < count; i++) {
    entries.push({
      wellName: `Well ${i + 1}`,
      productionVolume: randomInRange([50, 200]),
      pressure: randomInRange([1000, 3000]),
      // ... all fields
    });
  }
  return entries;
};
```

## Key Insights

`★ Insight ─────────────────────────────────────`
**Probability-Based Outliers** - Using `Math.random() > 0.7` creates a 30% chance of generating out-of-range values. This is more effective than "always valid" or "always invalid" test data, allowing you to test both happy paths AND validation edge cases in a single button press.

**Conditional Field Reset Pattern** - Always reset ALL context-specific fields to empty strings BEFORE populating based on entity type. This prevents "field leakage" where previous test data from a different context (e.g., beam-pump fields) persists when switching contexts (e.g., to gas-lift well).

**Visual Affordance for Dev Tools** - Dashed amber borders with light yellow backgrounds follow a UX pattern where "warning colors" signal "this is not production behavior." Users immediately understand this is a development-only feature, similar to how staging environments use yellow banners.
`─────────────────────────────────────────────────`

## References

- **JavaScript Random Number Generation:** https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random
- **Normal Distribution (Gaussian):** https://en.wikipedia.org/wiki/Normal_distribution
- **Seeded Random (seedrandom):** https://www.npmjs.com/package/seedrandom
- **React Native Haptics:** https://docs.expo.dev/versions/latest/sdk/haptics/

---

**Pattern Status:** ✅ Active
**Production Usage:** WellOS Mobile App (Field Data Entry)
**Last Validated:** October 29, 2025
