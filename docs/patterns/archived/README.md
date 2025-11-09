# Archived Patterns

This directory contains pattern documentation for technologies that are no longer used in the WellOS platform.

## Why These Patterns Are Archived

WellOS has fully migrated to a **Rust + Python** architecture:
- **Backend API**: Rust + Axum framework + SQLx
- **ML Service**: Python + FastAPI
- **Frontend**: React + Next.js (TypeScript is still used here)

The following patterns were created when the backend used NestJS (TypeScript) and Drizzle ORM, but are no longer applicable.

---

## Archived Patterns

### Pattern 40: Drizzle ORM Patterns
**Status**: Archived (replaced by SQLx)
**Reason**: Backend migrated from TypeScript/Drizzle to Rust/SQLx
**Alternative**: See Pattern 53 (Database Performance Optimization with SQLx)

### Pattern 89: NestJS CQRS Module Organization
**Status**: Archived (NestJS no longer used)
**Reason**: Backend migrated from NestJS to Rust/Axum
**Alternative**: See Pattern 05 (CQRS Pattern with Rust)

### Pattern 90: NestJS Parameter Decorator Pattern
**Status**: Archived (NestJS no longer used)
**Reason**: Backend migrated from NestJS to Rust/Axum
**Alternative**: Use Axum extractors (`FromRequestParts` trait)

---

## Historical Value

These patterns are preserved for:
1. **Knowledge Transfer**: Understanding the evolution of the platform
2. **Migration Reference**: Comparing old vs. new implementation approaches
3. **Architecture Decisions**: Documenting why certain technologies were chosen/replaced

---

## Active Patterns

For current patterns applicable to the Rust + Python stack, see:
- [Main Pattern Catalog](../README.md)
- [Pattern Integration Guide](../16-Pattern-Integration-Guide.md)
- [Rust-Specific Patterns](../94-Rust-Anti-Corruption-Layer-Pattern.md)
