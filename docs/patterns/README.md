# WellOS Pattern Library

A comprehensive collection of architectural, design, and implementation patterns for building the WellOS PSA platform.

---

## 📚 Pattern Categories

### Core Architectural Patterns (01-17)

**Foundation patterns that define the overall system architecture:**

- **[01 - RBAC CASL Pattern](./01-RBAC-CASL-Pattern.md)** - Role-Based Access Control with CASL
- **[02 - SOLID Principles](./02-SOLID-Principles.md)** - Five principles of object-oriented design
- **[03 - Hexagonal Architecture](./03-Hexagonal-Architecture.md)** - Ports and adapters architecture
- **[04 - Domain-Driven Design](./04-Domain-Driven-Design.md)** - Strategic and tactical DDD patterns
- **[05 - CQRS Pattern](./05-CQRS-Pattern.md)** - Command Query Responsibility Segregation
- **[06 - Repository Pattern](./06-Repository-Pattern.md)** - Data access abstraction
- **[07 - DTO Pattern](./07-DTO-Pattern.md)** - Data Transfer Objects for API contracts
- **[08 - Specification Pattern](./08-Specification-Pattern.md)** - Business rule encapsulation
- **[09 - Unit of Work Pattern](./09-Unit-of-Work-Pattern.md)** - Transaction management
- **[10 - Strategy Pattern](./10-Strategy-Pattern.md)** - Algorithmic encapsulation
- **[11 - Factory Pattern](./11-Factory-Pattern.md)** - Object creation abstraction
- **[12 - Observer Pattern](./12-Observer-Pattern.md)** - Event-driven architecture
- **[13 - Circuit Breaker Pattern](./13-Circuit-Breaker-Pattern.md)** - Fault tolerance for external calls
- **[14 - Anti-Corruption Layer Pattern](./14-Anti-Corruption-Layer-Pattern.md)** - External system integration
- **[15 - Retry Pattern](./15-Retry-Pattern.md)** - Resilient external call handling
- **[16 - Pattern Integration Guide](./16-Pattern-Integration-Guide.md)** ⭐ - **START HERE** - How to combine patterns
- **[17 - Multi-Tenancy Pattern](./17-Multi-Tenancy-Pattern.md)** - Organization data isolation

---

### Frontend Patterns (18-36)

**Patterns specific to the Next.js/React frontend:**

- **[18 - Frontend Patterns Guide](./18-Frontend-Patterns-Guide.md)** ⭐ - Overview of frontend architecture
- **[19 - Soft Delete Implementation Guide](./19-Soft-Delete-Implementation-Guide.md)** - Audit-compliant deletion
- **[21 - Frontend Repository Pattern](./21-Frontend-Repository-Pattern.md)** - Client-side data access
- **[22 - Frontend Command Query Separation](./22-Frontend-Command-Query-Separation.md)** - CQRS for frontend
- **[23 - Frontend Event-Driven Architecture](./23-Frontend-Event-Driven-Architecture.md)** - Event handling patterns
- **[24 - Frontend Component Factory Pattern](./24-Frontend-Component-Factory-Pattern.md)** - Dynamic component creation
- **[25 - Frontend State Management Pattern](./25-Frontend-State-Management-Pattern.md)** - React Query + Zustand
- **[27 - Additional Frontend Patterns Recommendations](./27-Additional-Frontend-Patterns-Recommendations.md)** - Supplementary patterns
- **[28 - Frontend Adapter Pattern](./28-Frontend-Adapter-Pattern.md)** - API integration adapters
- **[29 - Frontend Decorator Pattern](./29-Frontend-Decorator-Pattern.md)** - Component enhancement
- **[30 - Frontend Builder Pattern](./30-Frontend-Builder-Pattern.md)** - Complex object construction
- **[31 - Frontend Chain of Responsibility Pattern](./31-Frontend-Chain-of-Responsibility-Pattern.md)** - Request handling chains
- **[32 - Frontend Memento Pattern](./32-Frontend-Memento-Pattern.md)** - State snapshots and undo/redo
- **[33 - Frontend Proxy Pattern](./33-Frontend-Proxy-Pattern.md)** - Lazy loading and access control
- **[34 - Frontend Template Method Pattern](./34-Frontend-Template-Method-Pattern.md)** - Algorithm structure definition
- **[35 - Frontend Visitor Pattern](./35-Frontend-Visitor-Pattern.md)** - Operation separation from objects
- **[36 - Frontend Patterns Integration Architecture](./36-Frontend-Patterns-Integration-Architecture.md)** ⭐ - Frontend pattern combination guide
- **[77 - Form Field Auto-Generation Pattern](./77-Form-Field-Auto-Generation-Pattern.md)** 🆕 📝 - Cascading form field auto-generation with React Hook Form

---

### Backend Patterns & Integration (37-41)

**Backend-specific implementation patterns:**

- **[37 - Backend Patterns Integration Architecture](./37-Backend-Patterns-Integration-Architecture.md)** ⭐ - Backend pattern combination guide
- **[39 - Security Patterns Guide](./39-Security-Patterns-Guide.md)** ⭐ - Authentication, authorization, encryption
- **[40 - Drizzle ORM Patterns](./40-Drizzle-ORM-Patterns.md)** - Database access with Drizzle
- **[40 - Jest Testing Patterns](./40-Jest-Testing-Patterns.md)** - Unit and integration testing
- **[41 - Database Constraint Race Condition Pattern](./41-Database-Constraint-Race-Condition-Pattern.md)** - Handling unique constraint violations
- **[41 - REST API Best Practices](./41-REST-API-Best-Practices.md)** - RESTful API design

---

### API & Integration Patterns (42-51)

**External integration and communication patterns:**

- **[42 - GraphQL API Patterns](./42-GraphQL-API-Patterns.md)** - GraphQL schema design (future)
- **[43 - WebSocket RealTime Patterns](./43-WebSocket-RealTime-Patterns.md)** - Real-time communication
- **[44 - File Upload Download Patterns](./44-File-Upload-Download-Patterns.md)** - File handling strategies
- **[45 - Background Job Patterns](./45-Background-Job-Patterns.md)** - Scheduled and asynchronous tasks
- **[46 - Caching Strategy Patterns](./46-Caching-Strategy-Patterns.md)** - Redis caching patterns
- **[47 - Monitoring Observability Patterns](./47-Monitoring-Observability-Patterns.md)** - Logging and metrics
- **[48 - API Gateway Pattern](./48-API-Gateway-Pattern.md)** - API routing and aggregation
- **[49 - Event Sourcing Pattern](./49-Event-Sourcing-Pattern.md)** - Event-based state management
- **[50 - SAGA Pattern](./50-SAGA-Pattern.md)** - Distributed transactions
- **[51 - Feature Flags Pattern](./51-Feature-Flags-Pattern.md)** - Progressive feature rollout
- **[73 - Custom Metrics Dashboard Pattern](./73-Custom-Metrics-Dashboard-Pattern.md)** 🆕 💰 - Cost-saving alternative to Grafana on Azure
- **[74 - Stress Testing and Load Testing Pattern](./74-Stress-Testing-And-Load-Testing-Pattern.md)** 🆕 ⚡ - Artillery & custom scripts for performance testing
- **[75 - Database Provisioning Automation Pattern](./75-Database-Provisioning-Automation-Pattern.md)** 🆕 🗄️ - Automated database lifecycle management with Bash scripts
- **[76 - Cross-Platform Code Reuse Pattern](./76-Cross-Platform-Code-Reuse-Pattern.md)** 🆕 📱 - React Native monorepo for iOS/Android/macOS/Windows with 75-85% code sharing
- **[78 - Backend Communication Template Pattern](./78-Backend-Communication-Template-Pattern.md)** 🆕 📧 - Email/SMS template management with type-safe rendering

---

### Implementation Excellence Patterns (52-62)

**Practical patterns for building robust PSA features:**

- **[52 - User-Friendly Error Handling Pattern](./52-User-Friendly-Error-Handling-Pattern.md)** - Translating technical errors to user messages
- **[53 - Database Performance Optimization Pattern](./53-Database-Performance-Optimization-Pattern.md)** - Query optimization and indexing
- **[54 - PSA Patterns And Best Practices](./54-PSA-Patterns-And-Best-Practices.md)** ⭐ - PSA-specific implementation patterns
- **[55 - Implementation Excellence Pattern](./55-Implementation-Excellence-Pattern.md)** 🆕 - Comprehensive feature implementation workflow
- **[56 - Smart Suggestions Pattern](./56-Smart-Suggestions-Pattern.md)** 🆕 - Contextual recommendations and insights
- **[57 - Contextual Intelligence Pattern](./57-Contextual-Intelligence-Pattern.md)** 🆕 - Proactive contextual alerts (Sprint 7 feature)
- **[58 - JWT Authentication Session Management Pattern](./58-JWT-Authentication-Session-Management-Pattern.md)** - Secure token-based auth
- **[59 - Pagination Offset Calculation Pattern](./59-Pagination-Offset-Calculation-Pattern.md)** - Repository pagination formulas
- **[60 - Pagination Index Convention Pattern](./60-Pagination-Index-Convention-Pattern.md)** - 1-indexed vs 0-indexed consistency
- **[61 - Value Object Layer Boundary Pattern](./61-Value-Object-Layer-Boundary-Pattern.md)** 🆕 - Accessing value objects across architectural layers
- **[62 - React Query Cache Invalidation Pattern](./62-React-Query-Cache-Invalidation-Pattern.md)** 🆕 - Proper cache invalidation to avoid race conditions
- **[63 - PostgreSQL JSONB Undefined Value Handling](./63-PostgreSQL-JSONB-Undefined-Value-Handling.md)** 🆕 - Handling undefined values in JSONB columns
- **[79 - Statistical Test Data Generation Pattern](./79-Statistical-Test-Data-Generation-Pattern.md)** 🆕 🎲 - Probability-based test data generation with intentional outliers

---

### Performance Optimization Patterns (91-93)

**High-impact patterns for maximizing application performance:**

- **[91 - TimescaleDB Time-Series Database Pattern](./91-TimescaleDB-Time-Series-Database-Pattern.md)** 🆕 ⚡ 🗄️ - 20x faster inserts, 14,000x query speedup with hypertables, compression (90% storage savings), and continuous aggregates for high-frequency SCADA data
- **[92 - React Server Components Pattern](./92-React-Server-Components-Pattern.md)** 🆕 ⚡ 🎨 - 10x faster TTFB (800ms → 80ms) and 89% smaller bundles through server-side rendering with progressive streaming (Next.js 15+)
- **[93 - Virtual Scrolling UI Performance Pattern](./93-Virtual-Scrolling-UI-Performance-Pattern.md)** 🆕 ⚡ 🎨 - 400x rendering speedup (12s → 30ms) and 99.5% memory reduction for large datasets (10K+ rows) maintaining 60 FPS scroll performance

---

### Rust & SCADA Patterns (94-95)

**Advanced patterns for Rust-based SCADA ingestion service:**

- **[94 - Rust Anti-Corruption Layer Pattern](./94-Rust-Anti-Corruption-Layer-Pattern.md)** 🆕 🦀 - Protocol-agnostic SCADA integration with Modbus, OPC UA, DNP3, and MQTT adapters isolating domain model from external systems
- **[95 - Rust SAGA Orchestration Pattern](./95-Rust-SAGA-Orchestration-Pattern.md)** 🆕 🦀 - Distributed transaction management for multi-tenant operations with compensating transactions and state persistence

---

## 🎯 Quick Start Guides

### New to the Codebase?

Start with these patterns in order:

1. **[03 - Hexagonal Architecture](./03-Hexagonal-Architecture.md)** - Understand the overall structure
2. **[16 - Pattern Integration Guide](./16-Pattern-Integration-Guide.md)** - Learn how patterns work together
3. **[04 - Domain-Driven Design](./04-Domain-Driven-Design.md)** - Core business logic patterns
4. **[05 - CQRS Pattern](./05-CQRS-Pattern.md)** - Command/query separation
5. **[06 - Repository Pattern](./06-Repository-Pattern.md)** - Data access patterns

### Building a New Feature?

Follow this decision tree:

1. **Define the domain model** → Use [04 - Domain-Driven Design](./04-Domain-Driven-Design.md)
2. **Create data access layer** → Use [06 - Repository Pattern](./06-Repository-Pattern.md)
3. **Implement business logic** → Use [05 - CQRS Pattern](./05-CQRS-Pattern.md)
4. **Add authorization** → Use [01 - RBAC CASL Pattern](./01-RBAC-CASL-Pattern.md)
5. **Build frontend** → Use [36 - Frontend Patterns Integration Architecture](./36-Frontend-Patterns-Integration-Architecture.md)
6. **Follow best practices** → Use [54 - PSA Patterns And Best Practices](./54-PSA-Patterns-And-Best-Practices.md)

### Implementing a Complex Workflow?

Check these patterns:

- **Multi-step processes** → [09 - Unit of Work Pattern](./09-Unit-of-Work-Pattern.md)
- **External integrations** → [14 - Anti-Corruption Layer Pattern](./14-Anti-Corruption-Layer-Pattern.md) + [13 - Circuit Breaker Pattern](./13-Circuit-Breaker-Pattern.md)
- **Dynamic behavior** → [10 - Strategy Pattern](./10-Strategy-Pattern.md) + [11 - Factory Pattern](./11-Factory-Pattern.md)
- **Complex validation** → [08 - Specification Pattern](./08-Specification-Pattern.md)
- **Real-time features** → [43 - WebSocket RealTime Patterns](./43-WebSocket-RealTime-Patterns.md)
- **Background processing** → [45 - Background Job Patterns](./45-Background-Job-Patterns.md)

---

## 📖 Pattern Usage by Sprint

### Completed Sprints

**Sprint 1 - Authentication**

- [01 - RBAC CASL Pattern](./01-RBAC-CASL-Pattern.md)
- [06 - Repository Pattern](./06-Repository-Pattern.md)
- [07 - DTO Pattern](./07-DTO-Pattern.md)
- [39 - Security Patterns Guide](./39-Security-Patterns-Guide.md)

**Sprint 2 - Organizations & Multi-tenancy**

- [17 - Multi-Tenancy Pattern](./17-Multi-Tenancy-Pattern.md)
- [06 - Repository Pattern](./06-Repository-Pattern.md)

**Sprint 3 - RBAC V2**

- [01 - RBAC CASL Pattern](./01-RBAC-CASL-Pattern.md)
- [06 - Repository Pattern](./06-Repository-Pattern.md)
- [08 - Specification Pattern](./08-Specification-Pattern.md)

**Sprint 4 - Time Tracking & Projects**

- [04 - Domain-Driven Design](./04-Domain-Driven-Design.md)
- [05 - CQRS Pattern](./05-CQRS-Pattern.md)
- [06 - Repository Pattern](./06-Repository-Pattern.md)
- [09 - Unit of Work Pattern](./09-Unit-of-Work-Pattern.md)
- [12 - Observer Pattern](./12-Observer-Pattern.md)

**Sprint 5 - Clients & Invoices**

- [04 - Domain-Driven Design](./04-Domain-Driven-Design.md)
- [05 - CQRS Pattern](./05-CQRS-Pattern.md)
- [06 - Repository Pattern](./06-Repository-Pattern.md)
- [11 - Factory Pattern](./11-Factory-Pattern.md)
- [10 - Strategy Pattern](./10-Strategy-Pattern.md)
- [44 - File Upload Download Patterns](./44-File-Upload-Download-Patterns.md) (PDF generation)

**Sprint 6 - Profitability & Expenses** ✅ Just Completed

- [04 - Domain-Driven Design](./04-Domain-Driven-Design.md) (Expense entity, ProfitabilityMetrics VO)
- [05 - CQRS Pattern](./05-CQRS-Pattern.md) (Expense commands, profitability queries)
- [06 - Repository Pattern](./06-Repository-Pattern.md) (ExpenseRepository)
- [11 - Factory Pattern](./11-Factory-Pattern.md) (Expense.create())
- [12 - Observer Pattern](./12-Observer-Pattern.md) (Domain events)
- [52 - User-Friendly Error Handling Pattern](./52-User-Friendly-Error-Handling-Pattern.md)
- [53 - Database Performance Optimization Pattern](./53-Database-Performance-Optimization-Pattern.md)

### Upcoming Sprints

**Sprint 7 - Polish & Enhancements** (includes Contextual Insights)

- [57 - Contextual Intelligence Pattern](./57-Contextual-Intelligence-Pattern.md) 🆕
- [56 - Smart Suggestions Pattern](./56-Smart-Suggestions-Pattern.md) 🆕
- [45 - Background Job Patterns](./45-Background-Job-Patterns.md) (Budget alerts scheduled job)
- [46 - Caching Strategy Patterns](./46-Caching-Strategy-Patterns.md) (Redis for profitability)

**Sprint 8 - Beta Launch** (includes AI Chat)

- [57 - Contextual Intelligence Pattern](./57-Contextual-Intelligence-Pattern.md) (AI-enhanced insights)
- [14 - Anti-Corruption Layer Pattern](./14-Anti-Corruption-Layer-Pattern.md) (Claude API integration)
- [13 - Circuit Breaker Pattern](./13-Circuit-Breaker-Pattern.md) (Claude API resilience)
- [43 - WebSocket RealTime Patterns](./43-WebSocket-RealTime-Patterns.md) (AI chat)

---

## 🔧 Pattern Maintenance

### Adding a New Pattern

1. **Create the pattern file** with the next available number: `XX-Pattern-Name.md`
2. **Follow the standard structure:**
   - Problem statement
   - Solution overview
   - Implementation example
   - When to use / When not to use
   - Trade-offs
   - Related patterns
3. **Add to this README** in the appropriate category
4. **Reference in CLAUDE.md** if it's a core pattern

### Pattern Numbering

- Patterns are numbered sequentially (01-57 currently)
- Some numbers are duplicated (40, 41) due to parallel pattern additions
- New patterns should use the next available number
- Categories are loosely organized by number range

### Pattern Versioning

Patterns evolve with the codebase:

- **Major updates:** Add a version note at the top (e.g., "Updated: January 2025")
- **Deprecations:** Mark with ⚠️ DEPRECATED and link to replacement pattern
- **New insights:** Add to existing pattern rather than creating duplicate

---

## 🎓 Pattern Learning Path

### Beginner (Weeks 1-2)

Focus on understanding the architecture:

- ✅ [03 - Hexagonal Architecture](./03-Hexagonal-Architecture.md)
- ✅ [04 - Domain-Driven Design](./04-Domain-Driven-Design.md) (just Entities and Value Objects)
- ✅ [06 - Repository Pattern](./06-Repository-Pattern.md)
- ✅ [07 - DTO Pattern](./07-DTO-Pattern.md)

### Intermediate (Weeks 3-4)

Expand to advanced backend patterns:

- ✅ [05 - CQRS Pattern](./05-CQRS-Pattern.md)
- ✅ [09 - Unit of Work Pattern](./09-Unit-of-Work-Pattern.md)
- ✅ [12 - Observer Pattern](./12-Observer-Pattern.md)
- ✅ [16 - Pattern Integration Guide](./16-Pattern-Integration-Guide.md)

### Advanced (Weeks 5-6)

Master complex integrations:

- ✅ [14 - Anti-Corruption Layer Pattern](./14-Anti-Corruption-Layer-Pattern.md)
- ✅ [13 - Circuit Breaker Pattern](./13-Circuit-Breaker-Pattern.md)
- ✅ [50 - SAGA Pattern](./50-SAGA-Pattern.md)
- ✅ [37 - Backend Patterns Integration Architecture](./37-Backend-Patterns-Integration-Architecture.md)

### Expert (Ongoing)

Apply PSA-specific patterns:

- ✅ [54 - PSA Patterns And Best Practices](./54-PSA-Patterns-And-Best-Practices.md)
- ✅ [55 - Implementation Excellence Pattern](./55-Implementation-Excellence-Pattern.md)
- ✅ [57 - Contextual Intelligence Pattern](./57-Contextual-Intelligence-Pattern.md)

---

## 📊 Pattern Statistics

- **Total Patterns:** 72
- **Architectural Patterns:** 17
- **Frontend Patterns:** 20
- **Backend Patterns:** 5
- **Integration Patterns:** 13 (includes #73, #74, #75)
- **Implementation Patterns:** 12
- **Performance Optimization Patterns:** 3 (#91, #92, #93)
- **Rust & SCADA Patterns:** 2 (#94, #95)
- **New Patterns (October 2025):** 10 (58, 59, 60, 61, 62, 63, 73, 74, 75, 77)
- **New Patterns (November 2025):** 5 (91, 92, 93, 94, 95)

---

## 🔍 Finding the Right Pattern

### By Feature Type

**Time Tracking / Project Management**
→ DDD + CQRS + Repository + Unit of Work + Observer

**Invoice / Billing**
→ DDD + Factory + Strategy + Repository + Unit of Work

**External Integrations (QuickBooks, etc.)**
→ Anti-Corruption Layer + Adapter + Circuit Breaker + Retry

**SCADA Protocol Integration (Modbus, OPC UA, DNP3)**
→ Rust Anti-Corruption Layer (#94) + Circuit Breaker + Retry

**Multi-Tenant Operations (Provisioning, Offline Sync)**
→ Rust SAGA Orchestration (#95) + Event Sourcing + State Persistence

**Real-time Features (Chat, Notifications)**
→ WebSocket + Observer + Event-Driven Architecture

**Reporting / Analytics**
→ CQRS (read models) + Caching + Background Jobs

**User Management / Auth**
→ RBAC + Strategy + Security Patterns

**Client Portal**
→ Multi-Tenancy + RBAC + Frontend Patterns Integration

---

## 📚 External Resources

- **Hexagonal Architecture:** [Alistair Cockburn's original article](https://alistair.cockburn.us/hexagonal-architecture/)
- **Domain-Driven Design:** [Eric Evans' DDD book](https://www.domainlanguage.com/ddd/)
- **CQRS Pattern:** [Martin Fowler's CQRS article](https://martinfowler.com/bliki/CQRS.html)
- **Enterprise Patterns:** [Martin Fowler's Patterns of Enterprise Application Architecture](https://martinfowler.com/books/eaa.html)
- **React Patterns:** [React Patterns website](https://reactpatterns.com/)

---

**Last Updated:** November 3, 2025
**Pattern Count:** 72 patterns
**Version:** 2.8 (Sprint 6-7 + Rust SCADA Patterns)
