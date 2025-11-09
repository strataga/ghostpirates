# Backend Communication Template Pattern

## Overview

The Backend Communication Template Pattern provides a structured approach to managing email and SMS templates in backend applications. This pattern centralizes template management, ensures consistent branding, promotes reusability, and separates presentation concerns from business logic.

**Note**: This pattern applies to the NestJS API (apps/api), not the SCADA ingestion service which is 100% Rust and does not handle email/SMS communications.

## Problem Statement

Backend applications often need to send various types of communications (emails, SMS) with:

- **Consistent branding** across all communications
- **Reusable components** (headers, footers, formatting)
- **Type-safe data interfaces** for template rendering
- **Multiple communication types** (transactional, operational, marketing)
- **Easy maintainability** when designs change
- **Template testing** independent of sending logic

Traditional approaches lead to:

- **Inline HTML strings** scattered throughout service files
- **Duplicated styling** across multiple email methods
- **Difficult to test** template rendering
- **Hard to update branding** (requires changes in multiple places)
- **Mixing concerns** (business logic + presentation)
- **No type safety** for template data

## Solution

Implement a template hierarchy with abstract base classes that provide common functionality, while concrete template classes handle specific communication types with type-safe data interfaces.

## Implementation

### Directory Structure

```
apps/api/src/infrastructure/templates/
├── base.template.ts              # Abstract base class
├── email/                        # Email templates
│   ├── field-entry-report.template.ts
│   ├── email-verification.template.ts
│   └── password-reset.template.ts
├── sms/                          # SMS templates
│   └── field-alert.template.ts
└── README.md                     # Template documentation
```

### Base Template Class

```typescript
// apps/api/src/infrastructure/templates/base.template.ts
export abstract class BaseTemplate<T = any> {
  /**
   * Render the template with provided data
   */
  abstract render(data: T): string;

  /**
   * Get the email subject line
   */
  abstract getSubject(data: T): string;

  /**
   * Replace template variables with actual values
   */
  protected replaceVariables(
    template: string,
    variables: Record<string, any>,
  ): string {
    let result = template;
    for (const [key, value] of Object.entries(variables)) {
      const regex = new RegExp(`{{${key}}}`, 'g');
      result = result.replace(regex, String(value ?? ''));
    }
    return result;
  }

  /**
   * Create common email header with WellOS branding
   */
  protected getEmailHeader(): string {
    return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>WellOS</title>
</head>
<body style="margin: 0; padding: 0; font-family: Arial, sans-serif; background-color: #f4f4f4;">
  <table role="presentation" style="width: 100%; border-collapse: collapse;">
    <tr>
      <td style="padding: 40px 20px;">
        <table role="presentation" style="max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
          <!-- Header -->
          <tr>
            <td style="padding: 40px 40px 20px; text-align: center; border-bottom: 3px solid #0066cc;">
              <h1 style="margin: 0; color: #0066cc; font-size: 28px; font-weight: bold;">WellOS</h1>
              <p style="margin: 10px 0 0; color: #666666; font-size: 14px;">Field Data Management Platform</p>
            </td>
          </tr>`;
  }

  /**
   * Create common email footer with copyright
   */
  protected getEmailFooter(message?: string): string {
    return `
          <!-- Footer -->
          <tr>
            <td style="padding: 30px 40px; background-color: #f8f9fa; border-top: 1px solid #e0e0e0; border-radius: 0 0 8px 8px;">
              ${message ? `<p style="margin: 0; color: #999999; font-size: 12px; line-height: 1.5;">${message}</p>` : ''}
              <p style="margin: 15px 0 0; color: #999999; font-size: 12px; line-height: 1.5;">
                &copy; ${new Date().getFullYear()} WellOS. All rights reserved.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>`;
  }
}
```

### Email Template Example

```typescript
// apps/api/src/infrastructure/templates/email/field-entry-report.template.ts
import { BaseTemplate } from '../base.template';

export interface FieldEntryReportData {
  senderName: string;
  wellName?: string;
  photoCount: number;
  entryData?: {
    productionVolume?: number;
    pressure?: number;
    temperature?: number;
    gasVolume?: number;
    waterCut?: number;
    notes?: string;
    recordedAt?: string;
    latitude?: number;
    longitude?: number;
  };
  checklist?: Array<{
    label: string;
    checked: boolean;
  }>;
}

export class FieldEntryReportTemplate extends BaseTemplate<FieldEntryReportData> {
  getSubject(data: FieldEntryReportData): string {
    return data.wellName
      ? `Field Entry Report - ${data.wellName}`
      : 'Field Entry Report';
  }

  render(data: FieldEntryReportData): string {
    const { senderName, wellName, photoCount, entryData, checklist } = data;

    return `
${this.getEmailHeader()}

          <!-- Content -->
          <tr>
            <td style="padding: 40px;">
              <h2 style="margin: 0 0 20px; color: #333333; font-size: 24px; font-weight: normal;">Field Entry Report</h2>

              <p style="margin: 0 0 20px; color: #666666; font-size: 16px; line-height: 1.5;">
                ${senderName} has shared a field entry report with you${wellName ? ` for <strong>${wellName}</strong>` : ''}. This report includes ${photoCount} photo${photoCount > 1 ? 's' : ''} and field data captured during the inspection.
              </p>

              ${this.renderWellInfo(wellName)}
              ${this.renderEntryData(entryData)}
              ${this.renderChecklist(checklist)}
              ${this.renderPhotoNotice(photoCount)}
            </td>
          </tr>

${this.getEmailFooter(`You received this email because ${senderName} shared field entry data with you through WellOS.`)}
    `.trim();
  }

  private renderWellInfo(wellName?: string): string {
    if (!wellName) return '';

    return `
              <div style="background-color: #e7f3ff; border-left: 4px solid #0066cc; padding: 15px; margin: 20px 0; border-radius: 4px;">
                <p style="margin: 0; color: #0066cc; font-size: 16px; font-weight: 600;">
                  <strong>Well:</strong> ${wellName}
                </p>
              </div>
    `;
  }

  private renderEntryData(
    entryData?: FieldEntryReportData['entryData'],
  ): string {
    if (!entryData) return '';

    const hasData =
      entryData.productionVolume !== undefined ||
      entryData.pressure !== undefined ||
      entryData.temperature !== undefined;

    if (!hasData && !entryData.notes && !entryData.latitude) return '';

    return `
      <div style="background-color: #f8f9fa; border-radius: 8px; padding: 20px; margin: 20px 0;">
        <h3 style="margin: 0 0 15px; color: #333333; font-size: 18px; font-weight: 600;">Field Entry Data</h3>

        ${this.renderDataTable(entryData)}
        ${this.renderGPS(entryData.latitude, entryData.longitude)}
        ${this.renderNotes(entryData.notes)}
      </div>
    `;
  }

  private renderDataTable(
    entryData: FieldEntryReportData['entryData'],
  ): string {
    if (!entryData) return '';

    const rows: string[] = [];

    if (entryData.productionVolume !== undefined) {
      rows.push(
        this.renderTableRow('Oil Production:', `${entryData.productionVolume} BBL`),
      );
    }
    if (entryData.pressure !== undefined) {
      rows.push(this.renderTableRow('Pressure:', `${entryData.pressure} PSI`));
    }
    if (entryData.temperature !== undefined) {
      rows.push(
        this.renderTableRow('Temperature:', `${entryData.temperature}°F`, true),
      );
    }

    if (rows.length === 0) return '';

    return `
        <table style="width: 100%; border-collapse: collapse;">
          ${rows.join('\n')}
        </table>
    `;
  }

  private renderTableRow(
    label: string,
    value: string,
    isLast: boolean = false,
  ): string {
    const borderStyle = isLast ? '' : 'border-bottom: 1px solid #e0e0e0;';
    return `
          <tr>
            <td style="padding: 8px 0; color: #666666; font-size: 14px; ${borderStyle}"><strong>${label}</strong></td>
            <td style="padding: 8px 0; color: #333333; font-size: 14px; text-align: right; ${borderStyle}">${value}</td>
          </tr>
    `;
  }

  private renderGPS(latitude?: number, longitude?: number): string {
    if (latitude === undefined || longitude === undefined) return '';

    return `
        <p style="margin: 15px 0 0; color: #666666; font-size: 13px;">
          <strong>GPS Location:</strong> ${latitude.toFixed(6)}, ${longitude.toFixed(6)}
        </p>
    `;
  }

  private renderNotes(notes?: string): string {
    if (!notes) return '';

    return `
        <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #e0e0e0;">
          <p style="margin: 0 0 5px; color: #666666; font-size: 14px; font-weight: 600;">Notes:</p>
          <p style="margin: 0; color: #333333; font-size: 14px; line-height: 1.5;">${notes}</p>
        </div>
    `;
  }

  private renderChecklist(
    checklist?: FieldEntryReportData['checklist'],
  ): string {
    if (!checklist || checklist.length === 0) return '';

    const items = checklist
      .map((item) => this.renderChecklistItem(item.label, item.checked))
      .join('\n');

    return `
      <div style="background-color: #f8f9fa; border-radius: 8px; padding: 20px; margin: 20px 0;">
        <h3 style="margin: 0 0 15px; color: #333333; font-size: 18px; font-weight: 600;">Daily Checklist</h3>
        <table style="width: 100%;">
          ${items}
        </table>
      </div>
    `;
  }

  private renderChecklistItem(label: string, checked: boolean): string {
    const checkboxBgColor = checked ? '#10b981' : '#ffffff';
    const checkboxBorderColor = checked ? '#10b981' : '#d1d5db';
    const checkmark = checked ? '✓' : '';
    const textColor = checked ? '#333333' : '#999999';

    return `
          <tr>
            <td style="padding: 6px 0;">
              <span style="display: inline-block; width: 18px; height: 18px; border: 2px solid ${checkboxBorderColor}; background-color: ${checkboxBgColor}; border-radius: 3px; text-align: center; line-height: 14px; color: #ffffff; font-size: 12px; font-weight: bold; margin-right: 8px; vertical-align: middle;">
                ${checkmark}
              </span>
              <span style="color: ${textColor}; font-size: 14px; vertical-align: middle;">
                ${label}
              </span>
            </td>
          </tr>
    `;
  }

  private renderPhotoNotice(photoCount: number): string {
    return `
              <div style="background-color: #fff9e6; border-left: 4px solid #f59e0b; padding: 15px; margin: 20px 0; border-radius: 4px;">
                <p style="margin: 0; color: #92400e; font-size: 14px; line-height: 1.5;">
                  <strong>📎 Attached Photos:</strong> This email includes ${photoCount} photo${photoCount > 1 ? 's' : ''} as attachment${photoCount > 1 ? 's' : ''}. You can view them directly in your email client without logging in.
                </p>
              </div>
    `;
  }
}
```

### SMS Template Example

```typescript
// apps/api/src/infrastructure/templates/sms/field-alert.template.ts
export interface FieldAlertSmsData {
  wellName: string;
  alertType: 'pressure' | 'temperature' | 'production' | 'equipment' | 'safety';
  value?: string;
  threshold?: string;
  message: string;
  timestamp: string;
  operatorName?: string;
}

export class FieldAlertSmsTemplate {
  /**
   * Render SMS message (160 characters max for single SMS)
   * Keep it concise but informative
   */
  render(data: FieldAlertSmsData): string {
    const { wellName, alertType, value, message } = data;

    // Format alert type with emoji for quick recognition
    const alertIcon = this.getAlertIcon(alertType);

    // SMS messages should be under 160 characters when possible
    // Format: [EMOJI] WELL: Alert message
    return `${alertIcon} ${wellName}: ${message}${value ? ` (${value})` : ''}`;
  }

  private getAlertIcon(alertType: FieldAlertSmsData['alertType']): string {
    const icons = {
      pressure: '⚠️',
      temperature: '🌡️',
      production: '📉',
      equipment: '🔧',
      safety: '🚨',
    };
    return icons[alertType] || '⚠️';
  }

  /**
   * Validate SMS length (160 chars for single SMS, 306 for concatenated)
   */
  validateLength(message: string): {
    valid: boolean;
    length: number;
    segmentCount: number;
  } {
    const length = message.length;
    const segmentCount = Math.ceil(length / 153); // 153 chars per segment for concatenated SMS

    return {
      valid: length <= 306, // Max 2 segments
      length,
      segmentCount: length <= 160 ? 1 : segmentCount,
    };
  }
}
```

### Service Integration

```typescript
// Email service using template pattern
import {
  FieldEntryReportTemplate,
  FieldEntryReportData,
} from '../templates/email/field-entry-report.template';

export class EmailService {
  constructor(private readonly configService: ConfigService) {
    // Initialize nodemailer transporter
  }

  async sendFieldEntryPhotos(
    recipientEmails: string[],
    photos: Array<{ localUri: string; remoteUrl?: string }>,
    senderName: string,
    wellName?: string,
    entryData?: FieldEntryReportData['entryData'],
    checklist?: Array<{ label: string; checked: boolean }>,
  ): Promise<void> {
    // Prepare template data
    const templateData: FieldEntryReportData = {
      senderName,
      wellName,
      photoCount: photos.length,
      entryData,
      checklist,
    };

    // Render email using template
    const template = new FieldEntryReportTemplate();
    const htmlContent = template.render(templateData);
    const subject = template.getSubject(templateData);

    try {
      // Download photos and create attachments
      const attachments = await this.createPhotoAttachments(photos);

      await this.transporter.sendMail({
        from: this.configService.get<string>('SMTP_FROM'),
        to: recipientEmails.join(', '),
        subject,
        html: htmlContent,
        attachments,
      });

      this.logger.log(
        `Field entry photos email sent to ${recipientEmails.length} recipient(s)`,
      );
    } catch (error) {
      this.logger.error('Failed to send field entry photos email', error);
      throw error;
    }
  }

  private async createPhotoAttachments(
    photos: Array<{ localUri: string; remoteUrl?: string }>,
  ): Promise<Array<{ filename: string; content: Buffer; contentType: string }>> {
    const attachments = [];

    for (let i = 0; i < photos.length; i++) {
      const photo = photos[i];
      try {
        let photoBuffer: Buffer;

        if (photo.remoteUrl) {
          // Download from remote URL
          const response = await fetch(photo.remoteUrl);
          const arrayBuffer = await response.arrayBuffer();
          photoBuffer = Buffer.from(arrayBuffer);
        } else if (photo.localUri.startsWith('data:')) {
          // Extract base64 data from data URI
          const matches = photo.localUri.match(/^data:([^;]+);base64,(.+)$/);
          if (matches) {
            photoBuffer = Buffer.from(matches[2], 'base64');
          } else {
            continue;
          }
        } else if (photo.localUri.startsWith('http')) {
          // Download from HTTP URL
          const response = await fetch(photo.localUri);
          const arrayBuffer = await response.arrayBuffer();
          photoBuffer = Buffer.from(arrayBuffer);
        } else {
          // Skip local file paths
          continue;
        }

        attachments.push({
          filename: `photo-${i + 1}.jpg`,
          content: photoBuffer,
          contentType: 'image/jpeg',
        });
      } catch (error) {
        this.logger.warn(`Failed to process photo ${i + 1}:`, error);
        // Continue with other photos
      }
    }

    return attachments;
  }
}
```

## Benefits

### 1. **Separation of Concerns**

- Template rendering logic separated from business logic
- Email service focuses on sending, not formatting
- Easy to update branding without touching service code

### 2. **Type Safety**

- TypeScript interfaces ensure correct template data
- Compile-time checking of template usage
- Autocomplete support in IDEs

### 3. **Reusability**

- Common components (header, footer) shared across templates
- Helper methods reduce duplication
- Easy to create new templates by extending base

### 4. **Maintainability**

- Templates organized in dedicated directory
- Clear file naming conventions
- Comprehensive documentation in README.md

### 5. **Testability**

- Templates can be tested independently
- No need to mock email transport for template tests
- Easy to verify rendered output

## Best Practices

### 1. **Email Design Guidelines**

```typescript
// ✅ Good: Inline CSS for email client compatibility
<td style="padding: 40px; color: #333333;">Content</td>

// ❌ Bad: External CSS (stripped by email clients)
<style>.content { padding: 40px; color: #333333; }</style>
<td class="content">Content</td>
```

### 2. **Template Organization**

```typescript
// ✅ Good: Clear file structure
templates/
├── email/
│   ├── transactional/     # User-triggered emails
│   ├── operational/       # System notifications
│   └── marketing/         # Promotional emails
└── sms/
    ├── alerts/            # Critical alerts
    └── notifications/     # General notifications

// ❌ Bad: Flat structure with all templates mixed
templates/
├── email1.template.ts
├── email2.template.ts
├── sms1.template.ts
└── ...
```

### 3. **Type-Safe Data Interfaces**

```typescript
// ✅ Good: Explicit interface with optional fields
export interface FieldEntryReportData {
  senderName: string;              // Required
  wellName?: string;               // Optional
  photoCount: number;              // Required
  entryData?: { /* ... */ };      // Optional
}

// ❌ Bad: Using 'any' or loosely typed data
export interface FieldEntryReportData {
  data: any;
}
```

### 4. **SMS Length Validation**

```typescript
// ✅ Good: Validate SMS length before sending
const template = new FieldAlertSmsTemplate();
const message = template.render(data);
const validation = template.validateLength(message);

if (!validation.valid) {
  throw new Error(`SMS too long: ${validation.length} chars`);
}

// ❌ Bad: No length validation (may be truncated or split)
const message = template.render(data);
sendSms(message); // Could fail or be truncated
```

### 5. **Graceful Degradation**

```typescript
// ✅ Good: Optional sections that gracefully handle missing data
private renderNotes(notes?: string): string {
  if (!notes) return '';  // Return empty string if no notes

  return `
    <div>
      <p><strong>Notes:</strong> ${notes}</p>
    </div>
  `;
}

// ❌ Bad: Crashing on missing data
private renderNotes(notes: string): string {
  return `<div><p>${notes.toUpperCase()}</p></div>`;  // Crashes if notes undefined
}
```

## Testing

```typescript
// apps/api/src/infrastructure/templates/email/__tests__/field-entry-report.template.spec.ts
describe('FieldEntryReportTemplate', () => {
  let template: FieldEntryReportTemplate;

  beforeEach(() => {
    template = new FieldEntryReportTemplate();
  });

  describe('getSubject', () => {
    it('should include well name in subject when provided', () => {
      const data: FieldEntryReportData = {
        senderName: 'John Doe',
        wellName: 'Smith #1',
        photoCount: 3,
      };

      const subject = template.getSubject(data);

      expect(subject).toBe('Field Entry Report - Smith #1');
    });

    it('should use generic subject when well name not provided', () => {
      const data: FieldEntryReportData = {
        senderName: 'John Doe',
        photoCount: 3,
      };

      const subject = template.getSubject(data);

      expect(subject).toBe('Field Entry Report');
    });
  });

  describe('render', () => {
    it('should render email with all data sections', () => {
      const data: FieldEntryReportData = {
        senderName: 'John Doe',
        wellName: 'Smith #1',
        photoCount: 3,
        entryData: {
          productionVolume: 50,
          pressure: 1200,
          temperature: 145,
          notes: 'All systems normal',
          latitude: 31.7619,
          longitude: -106.485,
        },
        checklist: [
          { label: 'Check pressure', checked: true },
          { label: 'Inspect pump', checked: true },
        ],
      };

      const html = template.render(data);

      expect(html).toContain('WellOS'); // Header
      expect(html).toContain('Field Entry Report'); // Title
      expect(html).toContain('John Doe'); // Sender name
      expect(html).toContain('Smith #1'); // Well name
      expect(html).toContain('50 BBL'); // Production volume
      expect(html).toContain('1200 PSI'); // Pressure
      expect(html).toContain('145°F'); // Temperature
      expect(html).toContain('All systems normal'); // Notes
      expect(html).toContain('31.7619'); // Latitude
      expect(html).toContain('Check pressure'); // Checklist
      expect(html).toContain('3 photos'); // Photo count
    });

    it('should gracefully handle missing optional data', () => {
      const data: FieldEntryReportData = {
        senderName: 'John Doe',
        photoCount: 1,
      };

      const html = template.render(data);

      expect(html).toContain('WellOS');
      expect(html).toContain('John Doe');
      expect(html).not.toContain('undefined');
      expect(html).not.toContain('null');
    });

    it('should use proper HTML structure for email clients', () => {
      const data: FieldEntryReportData = {
        senderName: 'John Doe',
        photoCount: 1,
      };

      const html = template.render(data);

      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('<table role="presentation"'); // Table-based layout
      expect(html).toContain('style='); // Inline CSS
      expect(html).toContain('</html>');
    });
  });
});
```

## Related Patterns

- **[Strategy Pattern](./05-Strategy-Pattern-File-Storage.md)**: Use for selecting different email providers (SendGrid, AWS SES, etc.)
- **[Factory Pattern](./03-Factory-Pattern-Database-Connections.md)**: Use for creating different template instances based on email type
- **[Template Method Pattern (Frontend)](./34-Frontend-Template-Method-Pattern.md)**: Similar pattern applied to frontend report generation

## WellOS-Specific Implementation

In WellOS, this pattern is used for:

1. **Field Entry Reports**: Share inspection photos and data via email (apps/api/src/infrastructure/templates/email/field-entry-report.template.ts)
2. **Field Alerts**: Send critical SMS alerts to operators (apps/api/src/infrastructure/templates/sms/field-alert.template.ts)
3. **Authentication Emails**: Verification codes and password resets (inline, should be extracted)

## Future Enhancements

Consider adding these templates:

- **Email Verification Template** (extract from inline)
- **Password Reset Template** (extract from inline)
- **Weekly Production Summary Email**
- **Maintenance Reminder Notifications**
- **Compliance Report Templates**
- **Invoice Notifications**
- **Team Invitation Emails**
- **Safety Alert Templates**

## Migration Strategy

To migrate existing inline templates:

1. **Create Template File**: Copy inline HTML to new template file
2. **Define Data Interface**: Extract template data to TypeScript interface
3. **Implement Methods**: Add `render()` and `getSubject()` methods
4. **Refactor Service**: Update service to use template instance
5. **Write Tests**: Add template-specific tests
6. **Remove Inline Code**: Delete old inline HTML methods

Example migration:

```typescript
// Before: Inline template in email.service.ts
private createVerificationEmailTemplate(code: string, url: string): string {
  return `<!DOCTYPE html>...200+ lines of HTML...`;
}

// After: Dedicated template file
// apps/api/src/infrastructure/templates/email/email-verification.template.ts
export class EmailVerificationTemplate extends BaseTemplate<EmailVerificationData> {
  getSubject(): string {
    return 'Verify your WellOS account';
  }

  render(data: EmailVerificationData): string {
    return `
${this.getEmailHeader()}
      <!-- Verification-specific content -->
${this.getEmailFooter()}
    `.trim();
  }
}

// Service usage
const template = new EmailVerificationTemplate();
const html = template.render({ code, verificationUrl });
await this.sendMail({ html, subject: template.getSubject() });
```

The Backend Communication Template Pattern provides a clean, maintainable approach to managing email and SMS templates in backend applications, ensuring consistent branding and type-safe template rendering.
