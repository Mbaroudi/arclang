.PHONY: help install dev build start clean db-up db-down db-reset db-tools db-migrate db-studio

help:
	@echo "ArcViz Web - Available commands:"
	@echo ""
	@echo "  make install      - Install all dependencies"
	@echo "  make dev          - Start development servers"
	@echo "  make build        - Build all apps for production"
	@echo "  make start        - Start production servers"
	@echo "  make clean        - Clean all build artifacts"
	@echo ""
	@echo "  make db-up        - Start PostgreSQL + Redis"
	@echo "  make db-down      - Stop databases"
	@echo "  make db-reset     - Reset databases (⚠️  deletes all data)"
	@echo "  make db-tools     - Start pgAdmin + Redis Commander"
	@echo "  make db-migrate   - Run database migrations"
	@echo "  make db-studio    - Open Prisma Studio"
	@echo ""

install:
	pnpm install

dev:
	pnpm dev

build:
	pnpm build

start:
	pnpm start

clean:
	pnpm clean

db-up:
	docker-compose up -d postgres redis
	@echo "Waiting for databases to be ready..."
	@sleep 5
	@echo "✅ PostgreSQL available at: localhost:5432"
	@echo "✅ Redis available at: localhost:6379"

db-down:
	docker-compose down

db-reset:
	@echo "⚠️  WARNING: This will delete all data!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker-compose down -v; \
		docker-compose up -d postgres redis; \
		echo "✅ Databases reset successfully"; \
	else \
		echo "❌ Cancelled"; \
	fi

db-tools:
	docker-compose --profile tools up -d pgadmin redis-commander
	@echo "✅ pgAdmin: http://localhost:5050 (admin@arcviz.local / admin)"
	@echo "✅ Redis Commander: http://localhost:8081"

db-migrate:
	cd apps/api && pnpm prisma migrate dev

db-studio:
	cd apps/api && pnpm prisma studio
