# Ghost Pirates Makefile - Quality Commands

.PHONY: help install dev build test lint format clean check-all

help: ## Show this help message
	@echo "Ghost Pirates - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## Install all dependencies
	npm install
	cd apps/api && cargo build

dev: ## Start development servers (web + api)
	@echo "Starting development servers..."
	@make -j2 dev-web dev-api

dev-web: ## Start Next.js dev server
	npm run web:dev

dev-api: ## Start Rust API dev server
	npm run api:dev

build: ## Build all apps
	npm run build
	npm run api:build

test: ## Run all tests
	npm run test
	npm run api:test

lint: ## Lint all code
	npm run lint
	npm run api:clippy

format: ## Format all code
	npm run format
	npm run api:fmt

format-fix: ## Format all code (with auto-fix)
	npx prettier --write "**/*.{ts,tsx,js,jsx,json,md}"
	cd apps/api && cargo fmt

type-check: ## Type check TypeScript
	npm run type-check

check-all: ## Run all quality checks (lint + format + type-check + clippy)
	@echo "Running all quality checks..."
	npm run api:fmt
	npm run api:clippy
	npm run lint
	npm run type-check
	@echo "✅ All quality checks passed!"

clean: ## Clean build artifacts and dependencies
	npm run clean
	cd apps/api && cargo clean
	find . -name "node_modules" -type d -prune -exec rm -rf '{}' +
	find . -name ".next" -type d -prune -exec rm -rf '{}' +
	find . -name "dist" -type d -prune -exec rm -rf '{}' +

api-migrate: ## Run database migrations
	cd apps/api && sqlx migrate run

api-migrate-revert: ## Revert last migration
	cd apps/api && sqlx migrate revert

api-db-create: ## Create database
	cd apps/api && sqlx database create

api-db-drop: ## Drop database
	cd apps/api && sqlx database drop

api-db-reset: ## Reset database (drop + create + migrate)
	cd apps/api && sqlx database drop -y && sqlx database create && sqlx migrate run

docker-up: ## Start Docker services (PostgreSQL + Redis)
	docker-compose up -d

docker-down: ## Stop Docker services
	docker-compose down

docker-logs: ## Show Docker logs
	docker-compose logs -f

commit: ## Run pre-commit checks
	@make check-all
	@echo "✅ Ready to commit!"
