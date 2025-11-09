# Frontend Template Method Pattern

## Overview

The Template Method Pattern in React applications defines the skeleton of an
algorithm in a base class, letting subclasses override specific steps without
changing the algorithm's structure. This pattern is particularly valuable for
standardizing report generation, form processing workflows, and data
transformation pipelines in oil & gas applications.

## Problem Statement

Complex frontend applications often need to:

- **Generate standardized reports** with consistent structure but varying
  content
- **Process forms** through similar validation and submission workflows
- **Transform data** through predictable processing pipelines
- **Handle workflows** with common steps but different implementations
- **Maintain consistency** across similar operations

Traditional approaches lead to:

- **Code duplication** across similar processes
- **Inconsistent workflows** between different features
- **Difficult maintenance** when common steps need updates
- **Hard to enforce** business rules and standards
- **Complex testing** due to scattered logic

## Solution

Implement the Template Method Pattern to create standardized workflows with
customizable steps, ensuring consistency while allowing flexibility for specific
requirements.

## Implementation

### Base Template Interface

```typescript
// lib/templates/interfaces.ts
export interface TemplateStep<TInput, TOutput> {
  execute(input: TInput): Promise<TOutput>;
  canSkip?(input: TInput): boolean;
  onError?(error: Error, input: TInput): Promise<void>;
}

export interface TemplateContext {
  user: User;
  permissions: string[];
  metadata: Record<string, any>;
  options: Record<string, any>;
}

export interface ProcessingResult<T> {
  success: boolean;
  data?: T;
  errors: ProcessingError[];
  warnings: ProcessingWarning[];
  metadata: Record<string, any>;
}

export interface ProcessingError {
  step: string;
  message: string;
  code: string;
  severity: 'error' | 'warning';
}

export interface ProcessingWarning {
  step: string;
  message: string;
  code: string;
}
```

### Abstract Template Base

```typescript
// lib/templates/base-template.ts
export abstract class BaseTemplate<TInput, TOutput> {
  protected context: TemplateContext;
  protected steps: string[] = [];

  constructor(context: TemplateContext) {
    this.context = context;
  }

  // Template method - defines the algorithm skeleton
  async execute(input: TInput): Promise<ProcessingResult<TOutput>> {
    const result: ProcessingResult<TOutput> = {
      success: true,
      errors: [],
      warnings: [],
      metadata: {},
    };

    try {
      // Pre-processing hook
      await this.beforeProcessing(input, result);

      // Validate input
      const validationResult = await this.validateInput(input);
      if (!validationResult.isValid) {
        result.errors.push(
          ...validationResult.errors.map((e) => ({
            step: 'validation',
            message: e.message,
            code: e.code,
            severity: 'error' as const,
          })),
        );
        result.success = false;
        return result;
      }

      // Transform input data
      const transformedInput = await this.transformInput(input);

      // Process the main logic
      const processedData = await this.processData(transformedInput);

      // Transform output data
      const transformedOutput = await this.transformOutput(processedData);

      // Validate output
      const outputValidation = await this.validateOutput(transformedOutput);
      if (!outputValidation.isValid) {
        result.errors.push(
          ...outputValidation.errors.map((e) => ({
            step: 'output_validation',
            message: e.message,
            code: e.code,
            severity: 'error' as const,
          })),
        );
        result.success = false;
        return result;
      }

      // Finalize processing
      const finalResult = await this.finalizeProcessing(transformedOutput, result);

      // Post-processing hook
      await this.afterProcessing(finalResult, result);

      result.data = finalResult;
      return result;
    } catch (error) {
      result.success = false;
      result.errors.push({
        step: 'execution',
        message: error instanceof Error ? error.message : 'Unknown error',
        code: 'EXECUTION_ERROR',
        severity: 'error',
      });

      await this.handleError(error, input, result);
      return result;
    }
  }

  // Abstract methods that subclasses must implement
  protected abstract validateInput(input: TInput): Promise<ValidationResult>;
  protected abstract processData(input: TInput): Promise<any>;
  protected abstract transformOutput(data: any): Promise<TOutput>;

  // Hook methods that subclasses can override
  protected async beforeProcessing(
    input: TInput,
    result: ProcessingResult<TOutput>,
  ): Promise<void> {
    // Default implementation - can be overridden
  }

  protected async afterProcessing(
    output: TOutput,
    result: ProcessingResult<TOutput>,
  ): Promise<void> {
    // Default implementation - can be overridden
  }

  protected async transformInput(input: TInput): Promise<TInput> {
    // Default implementation - return input as-is
    return input;
  }

  protected async validateOutput(output: TOutput): Promise<ValidationResult> {
    // Default implementation - always valid
    return { isValid: true, errors: [], warnings: [] };
  }

  protected async finalizeProcessing(
    output: TOutput,
    result: ProcessingResult<TOutput>,
  ): Promise<TOutput> {
    // Default implementation - return output as-is
    return output;
  }

  protected async handleError(
    error: unknown,
    input: TInput,
    result: ProcessingResult<TOutput>,
  ): Promise<void> {
    // Default error handling - log the error
    console.error('Template processing error:', error);
  }

  // Utility methods
  protected addWarning(
    result: ProcessingResult<TOutput>,
    step: string,
    message: string,
    code: string,
  ): void {
    result.warnings.push({ step, message, code });
  }

  protected addError(
    result: ProcessingResult<TOutput>,
    step: string,
    message: string,
    code: string,
  ): void {
    result.errors.push({ step, message, code, severity: 'error' });
    result.success = false;
  }
}
```

### Report Generation Template

```typescript
// lib/templates/report-template.ts
export interface ReportData {
  title: string;
  period: { startDate: Date; endDate: Date };
  wells: Well[];
  operator: Operator;
  reportType: string;
}

export interface ReportOutput {
  content: string;
  format: 'pdf' | 'excel' | 'csv' | 'json';
  metadata: ReportMetadata;
  attachments: ReportAttachment[];
}

export interface ReportMetadata {
  generatedAt: Date;
  generatedBy: string;
  version: string;
  checksum: string;
}

export abstract class ReportTemplate extends BaseTemplate<ReportData, ReportOutput> {
  protected async validateInput(input: ReportData): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Validate required fields
    if (!input.title) {
      errors.push({
        field: 'title',
        message: 'Report title is required',
        code: 'MISSING_TITLE',
        severity: 'error',
      });
    }

    if (!input.period.startDate || !input.period.endDate) {
      errors.push({
        field: 'period',
        message: 'Report period is required',
        code: 'MISSING_PERIOD',
        severity: 'error',
      });
    }

    if (input.period.startDate > input.period.endDate) {
      errors.push({
        field: 'period',
        message: 'Start date must be before end date',
        code: 'INVALID_PERIOD',
        severity: 'error',
      });
    }

    if (!input.wells || input.wells.length === 0) {
      warnings.push({
        field: 'wells',
        message: 'No wells selected for report',
        code: 'NO_WELLS',
      });
    }

    return { isValid: errors.length === 0, errors, warnings };
  }

  protected async transformInput(input: ReportData): Promise<ReportData> {
    // Standardize dates to UTC
    return {
      ...input,
      period: {
        startDate: new Date(input.period.startDate.toISOString().split('T')[0] + 'T00:00:00.000Z'),
        endDate: new Date(input.period.endDate.toISOString().split('T')[0] + 'T23:59:59.999Z'),
      },
    };
  }

  protected async processData(input: ReportData): Promise<any> {
    // Gather data for report
    const reportData = await this.gatherReportData(input);

    // Perform calculations
    const calculations = await this.performCalculations(reportData);

    // Apply business rules
    const processedData = await this.comlyBusinessRules(calculations);

    return processedData;
  }

  protected async transformOutput(data: any): Promise<ReportOutput> {
    // Generate report content
    const content = await this.generateReportContent(data);

    // Create metadata
    const metadata: ReportMetadata = {
      generatedAt: new Date(),
      generatedBy: this.context.user.id,
      version: '1.0',
      checksum: this.calculateChecksum(content),
    };

    // Generate attachments
    const attachments = await this.generateAttachments(data);

    return {
      content,
      format: this.getReportFormat(),
      metadata,
      attachments,
    };
  }

  protected async validateOutput(output: ReportOutput): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Validate content
    if (!output.content || output.content.trim().length === 0) {
      errors.push({
        field: 'content',
        message: 'Report content cannot be empty',
        code: 'EMPTY_CONTENT',
        severity: 'error',
      });
    }

    // Validate metadata
    if (!output.metadata.checksum) {
      warnings.push({
        field: 'metadata',
        message: 'Report checksum is missing',
        code: 'MISSING_CHECKSUM',
      });
    }

    return { isValid: errors.length === 0, errors, warnings };
  }

  // Abstract methods for subclasses
  protected abstract gatherReportData(input: ReportData): Promise<any>;
  protected abstract performCalculations(data: any): Promise<any>;
  protected abstract generateReportContent(data: any): Promise<string>;
  protected abstract getReportFormat(): 'pdf' | 'excel' | 'csv' | 'json';

  // Hook methods with default implementations
  protected async applyBusinessRules(data: any): Promise<any> {
    // Default implementation - return data as-is
    return data;
  }

  protected async generateAttachments(data: any): Promise<ReportAttachment[]> {
    // Default implementation - no attachments
    return [];
  }

  protected calculateChecksum(content: string): string {
    // Simple checksum calculation
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(16);
  }
}
```

### Production Report Implementation

```typescript
// lib/templates/production-report.template.ts
export class ProductionReportTemplate extends ReportTemplate {
  protected async gatherReportData(input: ReportData): Promise<any> {
    const productionData = await Promise.all(
      input.wells.map(async (well) => {
        const production = await this.getProductionData(
          well.id,
          input.period.startDate,
          input.period.endDate,
        );

        return {
          well,
          production,
          tests: await this.getTestData(well.id, input.period),
          equipment: await this.getEquipmentData(well.id),
        };
      }),
    );

    return {
      wells: productionData,
      operator: input.operator,
      period: input.period,
      title: input.title,
    };
  }

  protected async performCalculations(data: any): Promise<any> {
    const calculations = {
      totalOilProduction: 0,
      totalGasProduction: 0,
      totalWaterProduction: 0,
      averageOilRate: 0,
      averageGasRate: 0,
      wellCount: data.wells.length,
      activeWellCount: 0,
    };

    for (const wellData of data.wells) {
      const production = wellData.production;

      calculations.totalOilProduction += production.oil || 0;
      calculations.totalGasProduction += production.gas || 0;
      calculations.totalWaterProduction += production.water || 0;

      if (production.oil > 0 || production.gas > 0) {
        calculations.activeWellCount++;
      }
    }

    // Calculate averages
    const daysInPeriod = Math.ceil(
      (data.period.endDate.getTime() - data.period.startDate.getTime()) / (1000 * 60 * 60 * 24),
    );

    calculations.averageOilRate = calculations.totalOilProduction / daysInPeriod;
    calculations.averageGasRate = calculations.totalGasProduction / daysInPeriod;

    return {
      ...data,
      calculations,
    };
  }

  protected async applyBusinessRules(data: any): Promise<any> {
    // Apply production reporting business rules
    const processedData = { ...data };

    // Flag wells with unusual production
    processedData.wells = data.wells.map((wellData: any) => {
      const flags: string[] = [];

      if (wellData.production.oil > 1000) {
        flags.push('HIGH_OIL_PRODUCTION');
      }

      if (wellData.production.water > wellData.production.oil * 10) {
        flags.push('HIGH_WATER_CUT');
      }

      if (wellData.production.oil === 0 && wellData.production.gas === 0) {
        flags.push('NO_PRODUCTION');
      }

      return {
        ...wellData,
        flags,
      };
    });

    return processedData;
  }

  protected async generateReportContent(data: any): Promise<string> {
    // Generate HTML content for production report
    const html = `
      <!DOCTYPE html>
      <html>
        <head>
          <title>${data.title}</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .header { border-bottom: 2px solid #333; padding-bottom: 10px; }
            .summary { background: #f5f5f5; padding: 15px; margin: 20px 0; }
            .well-table { width: 100%; border-collapse: collapse; margin: 20px 0; }
            .well-table th, .well-table td { border: 1px solid #ddd; padding: 8px; text-align: right; }
            .well-table th { background: #f2f2f2; }
            .flag { color: red; font-weight: bold; }
          </style>
        </head>
        <body>
          <div class="header">
            <h1>${data.title}</h1>
            <p>Operator: ${data.operator.name}</p>
            <p>Period: ${data.period.startDate.toLocaleDateString()} - ${data.period.endDate.toLocaleDateString()}</p>
            <p>Generated: ${new Date().toLocaleString()}</p>
          </div>

          <div class="summary">
            <h2>Production Summary</h2>
            <p>Total Oil Production: ${data.calculations.totalOilProduction.toLocaleString()} bbls</p>
            <p>Total Gas Production: ${data.calculations.totalGasProduction.toLocaleString()} mcf</p>
            <p>Total Water Production: ${data.calculations.totalWaterProduction.toLocaleString()} bbls</p>
            <p>Average Oil Rate: ${data.calculations.averageOilRate.toFixed(2)} bbl/day</p>
            <p>Average Gas Rate: ${data.calculations.averageGasRate.toFixed(2)} mcf/day</p>
            <p>Active Wells: ${data.calculations.activeWellCount} of ${data.calculations.wellCount}</p>
          </div>

          <h2>Well Details</h2>
          <table class="well-table">
            <thead>
              <tr>
                <th>Well Name</th>
                <th>API Number</th>
                <th>Oil (bbls)</th>
                <th>Gas (mcf)</th>
                <th>Water (bbls)</th>
                <th>Flags</th>
              </tr>
            </thead>
            <tbody>
              ${data.wells
                .map(
                  (wellData: any) => `
                <tr>
                  <td style="text-align: left;">${wellData.well.name}</td>
                  <td style="text-align: left;">${wellData.well.apiNumber}</td>
                  <td>${(wellData.production.oil || 0).toLocaleString()}</td>
                  <td>${(wellData.production.gas || 0).toLocaleString()}</td>
                  <td>${(wellData.production.water || 0).toLocaleString()}</td>
                  <td class="flag">${wellData.flags.join(', ')}</td>
                </tr>
              `,
                )
                .join('')}
            </tbody>
          </table>
        </body>
      </html>
    `;

    return html;
  }

  protected getReportFormat(): 'pdf' | 'excel' | 'csv' | 'json' {
    return 'pdf';
  }

  protected async generateAttachments(data: any): Promise<ReportAttachment[]> {
    const attachments: ReportAttachment[] = [];

    // Generate CSV export
    const csvContent = this.generateCSVContent(data);
    attachments.push({
      name: 'production-data.csv',
      content: csvContent,
      mimeType: 'text/csv',
    });

    // Generate detailed well reports if needed
    if (data.wells.length > 10) {
      const detailedReport = await this.generateDetailedWellReport(data);
      attachments.push({
        name: 'detailed-well-report.pdf',
        content: detailedReport,
        mimeType: 'application/pdf',
      });
    }

    return attachments;
  }

  // Helper methods
  private async getProductionData(wellId: string, startDate: Date, endDate: Date): Promise<any> {
    // Implementation would fetch production data from API
    return {
      oil: Math.floor(Math.random() * 500),
      gas: Math.floor(Math.random() * 1000),
      water: Math.floor(Math.random() * 200),
    };
  }

  private async getTestData(wellId: string, period: any): Promise<any> {
    // Implementation would fetch test data
    return [];
  }

  private async getEquipmentData(wellId: string): Promise<any> {
    // Implementation would fetch equipment data
    return [];
  }

  private generateCSVContent(data: any): string {
    const headers = ['Well Name', 'API Number', 'Oil (bbls)', 'Gas (mcf)', 'Water (bbls)', 'Flags'];
    const rows = data.wells.map((wellData: any) => [
      wellData.well.name,
      wellData.well.apiNumber,
      wellData.production.oil || 0,
      wellData.production.gas || 0,
      wellData.production.water || 0,
      wellData.flags.join('; '),
    ]);

    return [headers, ...rows].map((row) => row.join(',')).join('\n');
  }

  private async generateDetailedWellReport(data: any): Promise<string> {
    // Implementation would generate detailed PDF report
    return 'detailed-report-content';
  }
}
```

### Regulatory Report Implementation

```typescript
// lib/templates/regulatory-report.template.ts
export class RegulatoryReportTemplate extends ReportTemplate {
  protected async gatherReportData(input: ReportData): Promise<any> {
    // Gather regulatory-specific data
    const regulatoryData = await Promise.all(
      input.wells.map(async (well) => {
        return {
          well,
          permits: await this.getPermitData(well.id),
          compliance: await this.getComplianceData(well.id, input.period),
          inspections: await this.getInspectionData(well.id, input.period),
          violations: await this.getViolationData(well.id, input.period),
        };
      }),
    );

    return {
      wells: regulatoryData,
      operator: input.operator,
      period: input.period,
      title: input.title,
      regulatoryBody: this.determineRegulatoryBody(input.operator.state),
    };
  }

  protected async performCalculations(data: any): Promise<any> {
    const calculations = {
      totalWells: data.wells.length,
      compliantWells: 0,
      violationsCount: 0,
      inspectionsCount: 0,
      expiredPermits: 0,
    };

    for (const wellData of data.wells) {
      if (wellData.compliance.isCompliant) {
        calculations.compliantWells++;
      }

      calculations.violationsCount += wellData.violations.length;
      calculations.inspectionsCount += wellData.inspections.length;

      const expiredPermits = wellData.permits.filter(
        (permit: any) => permit.expirationDate < new Date(),
      );
      calculations.expiredPermits += expiredPermits.length;
    }

    return { ...data, calculations };
  }

  protected async applyBusinessRules(data: any): Promise<any> {
    // Apply regulatory business rules
    const processedData = { ...data };

    // Flag compliance issues
    processedData.wells = data.wells.map((wellData: any) => {
      const issues: string[] = [];

      if (!wellData.compliance.isCompliant) {
        issues.push('NON_COMPLIANT');
      }

      if (wellData.violations.length > 0) {
        issues.push('HAS_VIOLATIONS');
      }

      const expiredPermits = wellData.permits.filter(
        (permit: any) => permit.expirationDate < new Date(),
      );
      if (expiredPermits.length > 0) {
        issues.push('EXPIRED_PERMITS');
      }

      return { ...wellData, issues };
    });

    return processedData;
  }

  protected async generateReportContent(data: any): Promise<string> {
    // Generate regulatory report content
    const html = `
      <!DOCTYPE html>
      <html>
        <head>
          <title>${data.title}</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .header { border-bottom: 2px solid #333; padding-bottom: 10px; }
            .compliance-summary { background: #e8f5e8; padding: 15px; margin: 20px 0; }
            .violations-summary { background: #ffe8e8; padding: 15px; margin: 20px 0; }
            .compliance-table { width: 100%; border-collapse: collapse; margin: 20px 0; }
            .compliance-table th, .compliance-table td { border: 1px solid #ddd; padding: 8px; }
            .compliance-table th { background: #f2f2f2; }
            .compliant { color: green; font-weight: bold; }
            .non-compliant { color: red; font-weight: bold; }
          </style>
        </head>
        <body>
          <div class="header">
            <h1>${data.title}</h1>
            <p>Operator: ${data.operator.name}</p>
            <p>Regulatory Body: ${data.regulatoryBody}</p>
            <p>Period: ${data.period.startDate.toLocaleDateString()} - ${data.period.endDate.toLocaleDateString()}</p>
            <p>Generated: ${new Date().toLocaleString()}</p>
          </div>

          <div class="compliance-summary">
            <h2>Compliance Summary</h2>
            <p>Total Wells: ${data.calculations.totalWells}</p>
            <p>Compliant Wells: ${data.calculations.compliantWells}</p>
            <p>Compliance Rate: ${((data.calculations.compliantWells / data.calculations.totalWells) * 100).toFixed(1)}%</p>
          </div>

          ${
            data.calculations.violationsCount > 0
              ? `
            <div class="violations-summary">
              <h2>Violations Summary</h2>
              <p>Total Violations: ${data.calculations.violationsCount}</p>
              <p>Wells with Violations: ${data.wells.filter((w: any) => w.violations.length > 0).length}</p>
              <p>Expired Permits: ${data.calculations.expiredPermits}</p>
            </div>
          `
              : ''
          }

          <h2>Well Compliance Details</h2>
          <table class="compliance-table">
            <thead>
              <tr>
                <th>Well Name</th>
                <th>API Number</th>
                <th>Compliance Status</th>
                <th>Violations</th>
                <th>Last Inspection</th>
                <th>Issues</th>
              </tr>
            </thead>
            <tbody>
              ${data.wells
                .map(
                  (wellData: any) => `
                <tr>
                  <td>${wellData.well.name}</td>
                  <td>${wellData.well.apiNumber}</td>
                  <td class="${wellData.compliance.isCompliant ? 'compliant' : 'non-compliant'}">
                    ${wellData.compliance.isCompliant ? 'COMPLIANT' : 'NON-COMPLIANT'}
                  </td>
                  <td>${wellData.violations.length}</td>
                  <td>${
                    wellData.inspections.length > 0
                      ? new Date(wellData.inspections[0].date).toLocaleDateString()
                      : 'None'
                  }</td>
                  <td>${wellData.issues.join(', ')}</td>
                </tr>
              `,
                )
                .join('')}
            </tbody>
          </table>
        </body>
      </html>
    `;

    return html;
  }

  protected getReportFormat(): 'pdf' | 'excel' | 'csv' | 'json' {
    return 'pdf';
  }

  // Helper methods
  private async getPermitData(wellId: string): Promise<any[]> {
    // Implementation would fetch permit data
    return [];
  }

  private async getComplianceData(wellId: string, period: any): Promise<any> {
    // Implementation would fetch compliance data
    return { isCompliant: Math.random() > 0.2 };
  }

  private async getInspectionData(wellId: string, period: any): Promise<any[]> {
    // Implementation would fetch inspection data
    return [];
  }

  private async getViolationData(wellId: string, period: any): Promise<any[]> {
    // Implementation would fetch violation data
    return [];
  }

  private determineRegulatoryBody(state: string): string {
    const regulatoryBodies: Record<string, string> = {
      TX: 'Texas Railroad Commission',
      OK: 'Oklahoma Corporation Commission',
      NM: 'New Mexico Oil Conservation Division',
      CO: 'Colorado Oil and Gas Conservation Commission',
    };

    return regulatoryBodies[state] || 'State Regulatory Body';
  }
}
```

### React Hook Integration

```typescript
// hooks/use-report-template.ts
export function useReportTemplate<TInput, TOutput>(
  templateClass: new (context: TemplateContext) => BaseTemplate<TInput, TOutput>,
) {
  const { user } = useAuth();
  const { permissions } = usePermissions();

  const [template] = useState(() => {
    const context: TemplateContext = {
      user,
      permissions,
      metadata: {},
      options: {},
    };

    return new templateClass(context);
  });

  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessingResult<TOutput> | null>(null);

  const executeTemplate = useCallback(
    async (input: TInput) => {
      setProcessing(true);
      setResult(null);

      try {
        const processingResult = await template.execute(input);
        setResult(processingResult);
        return processingResult;
      } catch (error) {
        const errorResult: ProcessingResult<TOutput> = {
          success: false,
          errors: [
            {
              step: 'execution',
              message: error instanceof Error ? error.message : 'Unknown error',
              code: 'TEMPLATE_ERROR',
              severity: 'error',
            },
          ],
          warnings: [],
          metadata: {},
        };
        setResult(errorResult);
        return errorResult;
      } finally {
        setProcessing(false);
      }
    },
    [template],
  );

  return {
    executeTemplate,
    processing,
    result,
  };
}
```

### Component Usage

```typescript
// components/reports/production-report-generator.tsx
export function ProductionReportGenerator() {
  const { executeTemplate, processing, result } = useReportTemplate(ProductionReportTemplate);
  const [reportData, setReportData] = useState<ReportData>({
    title: '',
    period: { startDate: new Date(), endDate: new Date() },
    wells: [],
    operator: {} as Operator,
    reportType: 'production',
  });

  const handleGenerateReport = async () => {
    const processingResult = await executeTemplate(reportData);

    if (processingResult.success && processingResult.data) {
      // Handle successful report generation
      toast.success('Production report generated successfully');

      // Download or display the report
      downloadReport(processingResult.data);
    } else {
      // Handle errors
      const errorMessages = processingResult.errors.map(e => e.message).join(', ');
      toast.error(`Report generation failed: ${errorMessages}`);
    }
  };

  const downloadReport = (reportOutput: ReportOutput) => {
    const blob = new Blob([reportOutput.content], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `production-report-${Date.now()}.html`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Production Report Generator</h2>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="title">Report Title</Label>
          <Input
            id="title"
            value={reportData.title}
            onChange={(e) => setReportData(prev => ({ ...prev, title: e.target.value }))}
            placeholder="Enter report title"
          />
        </div>

        <div>
          <Label htmlFor="startDate">Start Date</Label>
          <Input
            id="startDate"
            type="date"
            value={reportData.period.startDate.toISOString().split('T')[0]}
            onChange={(e) => setReportData(prev => ({
              ...prev,
              period: { ...prev.period, startDate: new Date(e.target.value) }
            }))}
          />
        </div>
      </div>

      {/* Well selection, operator selection, etc. */}

      <Button
        onClick={handleGenerateReport}
        disabled={processing || !reportData.title}
      >
        {processing ? 'Generating Report...' : 'Generate Production Report'}
      </Button>

      {result && (
        <div className="mt-6">
          {result.success ? (
            <div className="p-4 bg-green-50 border border-green-200 rounded">
              <h3 className="font-semibold text-green-800">Report Generated Successfully</h3>
              <p className="text-green-600">Your production report has been generated.</p>
            </div>
          ) : (
            <div className="p-4 bg-red-50 border border-red-200 rounded">
              <h3 className="font-semibold text-red-800">Report Generation Failed</h3>
              <ul className="text-red-600 mt-2">
                {result.errors.map((error, index) => (
                  <li key={index}>• {error.message}</li>
                ))}
              </ul>
            </div>
          )}

          {result.warnings.length > 0 && (
            <div className="p-4 bg-yellow-50 border border-yellow-200 rounded mt-4">
              <h3 className="font-semibold text-yellow-800">Warnings</h3>
              <ul className="text-yellow-600 mt-2">
                {result.warnings.map((warning, index) => (
                  <li key={index}>• {warning.message}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

## Benefits

### 1. **Standardized Workflows**

- Consistent processing steps across different report types
- Enforced business rules and validation
- Predictable error handling and logging

### 2. **Code Reusability**

- Common processing logic shared across implementations
- Easy to create new report types by extending base template
- Reduced code duplication

### 3. **Maintainability**

- Changes to common steps automatically apply to all implementations
- Clear separation between template structure and specific logic
- Easy to test individual steps

### 4. **Flexibility**

- Subclasses can override specific steps as needed
- Hook methods allow customization without changing the algorithm
- Easy to add new processing steps

## Best Practices

### 1. **Clear Step Definition**

```typescript
// ✅ Good: Clear, single-purpose steps
protected abstract validateInput(input: TInput): Promise<ValidationResult>;
protected abstract processData(input: TInput): Promise<any>;
protected abstract transformOutput(data: any): Promise<TOutput>;

// ❌ Bad: Vague, multi-purpose steps
protected abstract doEverything(input: TInput): Promise<TOutput>;
```

### 2. **Error Handling**

```typescript
// ✅ Good: Comprehensive error handling
protected async handleError(error: unknown, input: TInput, result: ProcessingResult<TOutput>): Promise<void> {
  console.error('Template processing error:', error);
  // Log to monitoring service
  // Send notifications if needed
}

// ❌ Bad: No error handling
protected async handleError(): Promise<void> {
  // Empty implementation
}
```

### 3. **Validation**

```typescript
// ✅ Good: Thorough validation
protected async validateInput(input: TInput): Promise<ValidationResult> {
  const errors: ValidationError[] = [];

  // Check all required fields
  // Validate data types
  // Check business rules

  return { isValid: errors.length === 0, errors, warnings: [] };
}
```

## Testing

```typescript
// __tests__/templates/production-report.template.test.ts
describe('ProductionReportTemplate', () => {
  let template: ProductionReportTemplate;
  let mockContext: TemplateContext;

  beforeEach(() => {
    mockContext = {
      user: { id: 'user1', name: 'Test User' },
      permissions: ['read:reports', 'generate:reports'],
      metadata: {},
      options: {},
    };

    template = new ProductionReportTemplate(mockContext);
  });

  it('should generate production report successfully', async () => {
    const input: ReportData = {
      title: 'Test Production Report',
      period: {
        startDate: new Date('2024-01-01'),
        endDate: new Date('2024-01-31'),
      },
      wells: [mockWell],
      operator: mockOperator,
      reportType: 'production',
    };

    const result = await template.execute(input);

    expect(result.success).toBe(true);
    expect(result.data).toBeDefined();
    expect(result.data?.content).toContain('Test Production Report');
    expect(result.errors).toHaveLength(0);
  });

  it('should validate input and return errors for invalid data', async () => {
    const input: ReportData = {
      title: '', // Invalid - empty title
      period: {
        startDate: new Date('2024-01-31'),
        endDate: new Date('2024-01-01'), // Invalid - end before start
      },
      wells: [],
      operator: mockOperator,
      reportType: 'production',
    };

    const result = await template.execute(input);

    expect(result.success).toBe(false);
    expect(result.errors.length).toBeGreaterThan(0);
    expect(result.errors.some((e) => e.code === 'MISSING_TITLE')).toBe(true);
    expect(result.errors.some((e) => e.code === 'INVALID_PERIOD')).toBe(true);
  });
});
```

The Template Method Pattern provides a powerful way to standardize complex
workflows while maintaining flexibility for specific implementations, ensuring
consistency and maintainability across your application.
