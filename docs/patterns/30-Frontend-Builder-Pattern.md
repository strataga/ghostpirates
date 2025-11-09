# Frontend Builder Pattern

## Overview

The Builder Pattern in React applications provides a fluent interface for
constructing complex components, particularly forms and UI structures. This
pattern is especially valuable for oil & gas applications that require complex,
conditional forms like well completion reports, regulatory submissions, and
equipment configurations.

## Problem Statement

Complex forms in oil & gas applications often have:

- **Conditional fields** that appear based on previous selections
- **Multi-step workflows** with validation at each step
- **Dynamic sections** that can be added or removed
- **Complex validation rules** that depend on multiple fields
- **Different layouts** for different user roles or contexts
- **Reusable patterns** that need to be consistent across forms

Traditional form building approaches lead to:

- **Monolithic components** that are hard to maintain
- **Duplicated logic** across similar forms
- **Difficult testing** due to complex conditional logic
- **Poor reusability** of form patterns

## Solution

Implement the Builder Pattern to create a fluent, composable API for building
complex forms and UI structures with clear separation of concerns and high
reusability.

## Implementation

### Base Builder Interface

```typescript
// lib/builders/interfaces.ts
export interface Builder<T> {
  build(): T;
  reset(): this;
}

export interface FormFieldConfig {
  name: string;
  type: FieldType;
  label: string;
  placeholder?: string;
  required?: boolean;
  validation?: ValidationRule[];
  condition?: FieldCondition;
  options?: SelectOption[];
  defaultValue?: any;
  disabled?: boolean;
  description?: string;
  width?: 'full' | 'half' | 'third' | 'quarter';
}

export interface FormSectionConfig {
  title: string;
  description?: string;
  fields: FormFieldConfig[];
  condition?: SectionCondition;
  collapsible?: boolean;
  defaultExpanded?: boolean;
}

export interface FormConfig {
  title: string;
  description?: string;
  sections: FormSectionConfig[];
  validation: ValidationSchema;
  onSubmit: SubmitHandler;
  onCancel?: () => void;
  submitText?: string;
  cancelText?: string;
  layout?: 'single-column' | 'two-column' | 'grid';
}

export type FieldType =
  | 'text'
  | 'email'
  | 'password'
  | 'number'
  | 'tel'
  | 'select'
  | 'multiselect'
  | 'checkbox'
  | 'radio'
  | 'textarea'
  | 'date'
  | 'datetime'
  | 'time'
  | 'file'
  | 'image'
  | 'currency'
  | 'percentage'
  | 'api-number'
  | 'coordinates'
  | 'measurement';
```

### Form Builder Implementation

```typescript
// lib/builders/form.builder.ts
export class FormBuilder implements Builder<React.ReactElement> {
  private config: Partial<FormConfig> = {
    sections: [],
    layout: 'single-column',
  };

  private currentSection: FormSectionBuilder | null = null;

  title(title: string): this {
    this.config.title = title;
    return this;
  }

  description(description: string): this {
    this.config.description = description;
    return this;
  }

  layout(layout: FormConfig['layout']): this {
    this.config.layout = layout;
    return this;
  }

  section(title: string): FormSectionBuilder {
    this.currentSection = new FormSectionBuilder(title, this);
    return this.currentSection;
  }

  validation(schema: ValidationSchema): this {
    this.config.validation = schema;
    return this;
  }

  onSubmit(handler: SubmitHandler): this {
    this.config.onSubmit = handler;
    return this;
  }

  onCancel(handler: () => void): this {
    this.config.onCancel = handler;
    return this;
  }

  submitText(text: string): this {
    this.config.submitText = text;
    return this;
  }

  cancelText(text: string): this {
    this.config.cancelText = text;
    return this;
  }

  addSection(section: FormSectionConfig): this {
    this.config.sections!.push(section);
    return this;
  }

  build(): React.ReactElement {
    if (!this.config.title || !this.config.onSubmit || !this.config.validation) {
      throw new Error('Form must have title, onSubmit handler, and validation schema');
    }

    return <DynamicForm config={this.config as FormConfig} />;
  }

  reset(): this {
    this.config = { sections: [], layout: 'single-column' };
    this.currentSection = null;
    return this;
  }
}

export class FormSectionBuilder {
  private section: Partial<FormSectionConfig> = {
    fields: [],
  };

  constructor(
    title: string,
    private formBuilder: FormBuilder
  ) {
    this.section.title = title;
  }

  description(description: string): this {
    this.section.description = description;
    return this;
  }

  collapsible(collapsible = true): this {
    this.section.collapsible = collapsible;
    return this;
  }

  defaultExpanded(expanded = true): this {
    this.section.defaultExpanded = expanded;
    return this;
  }

  condition(condition: SectionCondition): this {
    this.section.condition = condition;
    return this;
  }

  // Field builder methods
  textField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('text', name, label, this);
  }

  emailField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('email', name, label, this);
  }

  numberField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('number', name, label, this);
  }

  selectField(name: string, label: string, options: SelectOption[]): FieldBuilder {
    const builder = new FieldBuilder('select', name, label, this);
    builder.options(options);
    return builder;
  }

  checkboxField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('checkbox', name, label, this);
  }

  textareaField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('textarea', name, label, this);
  }

  dateField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('date', name, label, this);
  }

  fileField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('file', name, label, this);
  }

  // Oil & Gas specific fields
  apiNumberField(name: string, label: string = 'API Number'): FieldBuilder {
    return new FieldBuilder('api-number', name, label, this)
      .validation([
        { type: 'required', message: 'API number is required' },
        { type: 'pattern', pattern: /^\d{14}$/, message: 'API number must be 14 digits' },
      ])
      .placeholder('Enter 14-digit API number');
  }

  coordinatesField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('coordinates', name, label, this);
  }

  measurementField(name: string, label: string, unit: string): FieldBuilder {
    return new FieldBuilder('measurement', name, label, this)
      .placeholder(`Enter ${label.toLowerCase()} in ${unit}`);
  }

  currencyField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('currency', name, label, this);
  }

  addField(field: FormFieldConfig): this {
    this.section.fields!.push(field);
    return this;
  }

  endSection(): FormBuilder {
    this.formBuilder.addSection(this.section as FormSectionConfig);
    return this.formBuilder;
  }
}

export class FieldBuilder {
  private field: Partial<FormFieldConfig>;

  constructor(
    type: FieldType,
    name: string,
    label: string,
    private sectionBuilder: FormSectionBuilder
  ) {
    this.field = { type, name, label };
  }

  placeholder(placeholder: string): this {
    this.field.placeholder = placeholder;
    return this;
  }

  required(required = true): this {
    this.field.required = required;
    return this;
  }

  disabled(disabled = true): this {
    this.field.disabled = disabled;
    return this;
  }

  description(description: string): this {
    this.field.description = description;
    return this;
  }

  defaultValue(value: any): this {
    this.field.defaultValue = value;
    return this;
  }

  width(width: FormFieldConfig['width']): this {
    this.field.width = width;
    return this;
  }

  options(options: SelectOption[]): this {
    this.field.options = options;
    return this;
  }

  validation(rules: ValidationRule[]): this {
    this.field.validation = rules;
    return this;
  }

  condition(condition: FieldCondition): this {
    this.field.condition = condition;
    return this;
  }

  // Conditional field methods
  showWhen(fieldName: string, value: any): this {
    this.field.condition = {
      field: fieldName,
      operator: 'equals',
      value,
    };
    return this;
  }

  hideWhen(fieldName: string, value: any): this {
    this.field.condition = {
      field: fieldName,
      operator: 'not_equals',
      value,
    };
    return this;
  }

  showWhenAny(fieldName: string, values: any[]): this {
    this.field.condition = {
      field: fieldName,
      operator: 'in',
      value: values,
    };
    return this;
  }

  endField(): FormSectionBuilder {
    this.sectionBuilder.addField(this.field as FormFieldConfig);
    return this.sectionBuilder;
  }
}
```

### Well Completion Form Example

```typescript
// lib/builders/well-completion-form.builder.ts
export class WellCompletionFormBuilder extends FormBuilder {
  static create(): WellCompletionFormBuilder {
    return new WellCompletionFormBuilder();
  }

  buildBasicWellForm(): this {
    return this.title('Well Completion Report')
      .description('Complete all required fields for well completion reporting')
      .layout('two-column')
      .section('Basic Well Information')
      .apiNumberField('apiNumber', 'API Number')
      .required()
      .endField()
      .textField('wellName', 'Well Name')
      .required()
      .placeholder('Enter well name')
      .endField()
      .selectField('wellType', 'Well Type', [
        { value: 'vertical', label: 'Vertical' },
        { value: 'horizontal', label: 'Horizontal' },
        { value: 'directional', label: 'Directional' },
      ])
      .required()
      .endField()
      .selectField('wellStatus', 'Well Status', [
        { value: 'drilling', label: 'Drilling' },
        { value: 'completed', label: 'Completed' },
        { value: 'producing', label: 'Producing' },
        { value: 'shut_in', label: 'Shut In' },
      ])
      .required()
      .endField()
      .endSection();
  }

  buildLocationSection(): this {
    return this.section('Location Information')
      .coordinatesField('surfaceLocation', 'Surface Location')
      .required()
      .description('Enter surface coordinates in decimal degrees')
      .endField()
      .coordinatesField('bottomHoleLocation', 'Bottom Hole Location')
      .showWhen('wellType', 'horizontal')
      .description('Required for horizontal wells')
      .endField()
      .textField('county', 'County')
      .required()
      .endField()
      .textField('state', 'State')
      .required()
      .defaultValue('TX')
      .endField()
      .endSection();
  }

  buildCompletionSection(): this {
    return this.section('Completion Details')
      .dateField('spudDate', 'Spud Date')
      .required()
      .endField()
      .dateField('completionDate', 'Completion Date')
      .required()
      .endField()
      .measurementField('totalDepth', 'Total Depth', 'feet')
      .required()
      .validation([
        { type: 'min', value: 0, message: 'Depth must be positive' },
        { type: 'max', value: 50000, message: 'Depth seems unrealistic' },
      ])
      .endField()
      .measurementField('lateralLength', 'Lateral Length', 'feet')
      .showWhen('wellType', 'horizontal')
      .validation([{ type: 'min', value: 0, message: 'Length must be positive' }])
      .endField()
      .numberField('perforationCount', 'Number of Perforations')
      .showWhenAny('wellType', ['horizontal', 'directional'])
      .endField()
      .endSection();
  }

  buildProductionSection(): this {
    return this.section('Initial Production')
      .description('Enter initial production test results')
      .collapsible()
      .defaultExpanded(false)
      .measurementField('initialOilRate', 'Initial Oil Rate', 'bbl/day')
      .endField()
      .measurementField('initialGasRate', 'Initial Gas Rate', 'mcf/day')
      .endField()
      .measurementField('initialWaterRate', 'Initial Water Rate', 'bbl/day')
      .endField()
      .numberField('testDuration', 'Test Duration (hours)')
      .validation([
        {
          type: 'min',
          value: 1,
          message: 'Test duration must be at least 1 hour',
        },
      ])
      .endField()
      .endSection();
  }

  buildRegulatorySection(): this {
    return this.section('Regulatory Information')
      .textField('permitNumber', 'Drilling Permit Number')
      .required()
      .endField()
      .dateField('permitDate', 'Permit Date')
      .required()
      .endField()
      .fileField('completionReport', 'Completion Report')
      .required()
      .description('Upload PDF completion report')
      .endField()
      .checkboxField('environmentalCompliance', 'Environmental Compliance Confirmed')
      .required()
      .endField()
      .endSection();
  }

  buildComplete(): React.ReactElement {
    return this.buildBasicWellForm()
      .buildLocationSection()
      .buildCompletionSection()
      .buildProductionSection()
      .buildRegulatorySection()
      .validation(wellCompletionSchema)
      .onSubmit(handleWellCompletionSubmit)
      .submitText('Submit Completion Report')
      .cancelText('Save as Draft')
      .build();
  }
}

// Usage
const WellCompletionForm = WellCompletionFormBuilder.create().buildComplete();
```

### Multi-Step Form Builder

```typescript
// lib/builders/multi-step-form.builder.ts
export class MultiStepFormBuilder implements Builder<React.ReactElement> {
  private steps: FormStepConfig[] = [];
  private currentStep = 0;
  private title = '';
  private onComplete?: (data: any) => void;

  title(title: string): this {
    this.title = title;
    return this;
  }

  step(title: string): FormStepBuilder {
    const stepBuilder = new FormStepBuilder(title, this);
    return stepBuilder;
  }

  onComplete(handler: (data: any) => void): this {
    this.onComplete = handler;
    return this;
  }

  addStep(step: FormStepConfig): this {
    this.steps.push(step);
    return this;
  }

  build(): React.ReactElement {
    if (!this.title || !this.onComplete || this.steps.length === 0) {
      throw new Error('Multi-step form must have title, completion handler, and at least one step');
    }

    return (
      <MultiStepForm
        title={this.title}
        steps={this.steps}
        onComplete={this.onComplete}
      />
    );
  }

  reset(): this {
    this.steps = [];
    this.currentStep = 0;
    this.title = '';
    this.onComplete = undefined;
    return this;
  }
}

export class FormStepBuilder {
  private step: Partial<FormStepConfig> = {
    fields: [],
  };

  constructor(
    title: string,
    private multiStepBuilder: MultiStepFormBuilder
  ) {
    this.step.title = title;
  }

  description(description: string): this {
    this.step.description = description;
    return this;
  }

  validation(schema: ValidationSchema): this {
    this.step.validation = schema;
    return this;
  }

  canSkip(canSkip = true): this {
    this.step.canSkip = canSkip;
    return this;
  }

  // Add field methods similar to FormSectionBuilder
  textField(name: string, label: string): FieldBuilder {
    return new FieldBuilder('text', name, label, this as any);
  }

  // ... other field methods

  endStep(): MultiStepFormBuilder {
    this.multiStepBuilder.addStep(this.step as FormStepConfig);
    return this.multiStepBuilder;
  }
}

// Usage for regulatory report submission
const RegulatoryReportForm = new MultiStepFormBuilder()
  .title('Monthly Production Report')
  .step('Well Selection')
    .description('Select wells to include in this report')
    .multiSelectField('wells', 'Wells', wellOptions)
      .required()
      .validation([{ type: 'min_length', value: 1, message: 'Select at least one well' }])
      .endField()
    .dateField('reportingPeriod', 'Reporting Period')
      .required()
      .endField()
    .endStep()
  .step('Production Data')
    .description('Enter production data for selected wells')
    // Dynamic fields based on selected wells
    .endStep()
  .step('Review & Submit')
    .description('Review your report before submission')
    .checkboxField('certifyAccuracy', 'I certify that this information is accurate')
      .required()
      .endField()
    .endStep()
  .onComplete(submitRegulatoryReport)
  .build();
```

### Table Builder

```typescript
// lib/builders/table.builder.ts
export class TableBuilder<T> implements Builder<React.ReactElement> {
  private config: Partial<TableConfig<T>> = {
    columns: [],
  };

  data(data: T[]): this {
    this.config.data = data;
    return this;
  }

  loading(loading = true): this {
    this.config.loading = loading;
    return this;
  }

  column(key: keyof T, header: string): ColumnBuilder<T> {
    return new ColumnBuilder<T>(key, header, this);
  }

  pagination(config: PaginationConfig): this {
    this.config.pagination = config;
    return this;
  }

  sorting(config: SortingConfig): this {
    this.config.sorting = config;
    return this;
  }

  filtering(config: FilteringConfig): this {
    this.config.filtering = config;
    return this;
  }

  actions(actions: TableAction<T>[]): this {
    this.config.actions = actions;
    return this;
  }

  addColumn(column: TableColumnConfig<T>): this {
    this.config.columns!.push(column);
    return this;
  }

  build(): React.ReactElement {
    if (!this.config.data || this.config.columns!.length === 0) {
      throw new Error('Table must have data and at least one column');
    }

    return <DataTable config={this.config as TableConfig<T>} />;
  }

  reset(): this {
    this.config = { columns: [] };
    return this;
  }
}

// Usage for well production table
const WellProductionTable = new TableBuilder<WellProduction>()
  .data(wellProductionData)
  .column('wellName', 'Well Name')
    .sortable()
    .filterable()
    .width('200px')
    .endColumn()
  .column('apiNumber', 'API Number')
    .render((apiNumber) => (
      <code className="text-sm">{apiNumber}</code>
    ))
    .endColumn()
  .column('oilProduction', 'Oil (bbl)')
    .sortable()
    .align('right')
    .render((oil) => oil.toLocaleString())
    .endColumn()
  .column('gasProduction', 'Gas (mcf)')
    .sortable()
    .align('right')
    .render((gas) => gas.toLocaleString())
    .endColumn()
  .actions([
    {
      label: 'View Details',
      onClick: (well) => router.push(`/wells/${well.id}`),
      icon: <Eye className="h-4 w-4" />,
    },
    {
      label: 'Edit',
      onClick: (well) => openEditDialog(well),
      icon: <Edit className="h-4 w-4" />,
    },
  ])
  .pagination({
    page: 1,
    pageSize: 25,
    total: wellProductionData.length,
    onPageChange: handlePageChange,
  })
  .build();
```

## Benefits

### 1. **Fluent Interface**

- Intuitive, readable API
- Method chaining for complex configurations
- Self-documenting code

### 2. **Reusability**

- Common patterns can be extracted
- Builders can be extended for specific use cases
- Consistent form structure across application

### 3. **Maintainability**

- Clear separation of form structure and logic
- Easy to modify form configurations
- Centralized form building logic

### 4. **Flexibility**

- Conditional fields and sections
- Dynamic form generation
- Multiple layout options

## Best Practices

### 1. **Builder Naming**

```typescript
// ✅ Good: Descriptive builder names
WellCompletionFormBuilder;
RegulatoryReportBuilder;
ProductionDataTableBuilder;

// ❌ Bad: Generic names
FormBuilder;
DataBuilder;
```

### 2. **Method Chaining**

```typescript
// ✅ Good: Fluent interface
const form = new FormBuilder()
  .title('Well Data')
  .section('Basic Info')
  .textField('name', 'Well Name')
  .required()
  .endField()
  .endSection()
  .build();

// ❌ Bad: Imperative style
const builder = new FormBuilder();
builder.setTitle('Well Data');
const section = builder.addSection('Basic Info');
section.addTextField('name', 'Well Name', true);
```

### 3. **Validation**

```typescript
// ✅ Good: Validate at build time
build(): React.ReactElement {
  if (!this.config.title || !this.config.onSubmit) {
    throw new Error('Form must have title and submit handler');
  }
  return <DynamicForm config={this.config} />;
}
```

## Testing

```typescript
// __tests__/builders/form.builder.test.ts
describe('FormBuilder', () => {
  it('should build a basic form', () => {
    const form = new FormBuilder()
      .title('Test Form')
      .section('Test Section')
      .textField('name', 'Name')
      .required()
      .endField()
      .endSection()
      .validation(testSchema)
      .onSubmit(jest.fn())
      .build();

    expect(form).toBeDefined();
    expect(form.props.config.title).toBe('Test Form');
    expect(form.props.config.sections).toHaveLength(1);
  });

  it('should throw error for incomplete form', () => {
    expect(() => {
      new FormBuilder().title('Test Form').build();
    }).toThrow('Form must have title, onSubmit handler, and validation schema');
  });
});
```

The Builder Pattern provides a powerful, flexible way to construct complex forms
and UI components with a clean, maintainable API that's perfect for the complex
requirements of oil & gas applications.
