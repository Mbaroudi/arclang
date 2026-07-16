# ArcViz Web - Modern MBSE Platform

AI-powered Model-Based Systems Engineering platform with advanced Arcadia methodology support.

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ 
- pnpm 8+
- Docker & Docker Compose

### Installation

```bash
# Install dependencies
pnpm install

# Copy environment variables
cp .env.example .env

# Start databases (PostgreSQL + Redis)
pnpm db:up

# Run database migrations
pnpm db:migrate

# Start development servers
pnpm dev
```

The application will be available at:
- Frontend: http://localhost:3002
- Backend API: http://localhost:4000
- PostgreSQL: localhost:5432
- Redis: localhost:6379

### Optional: Database Tools

```bash
# Start pgAdmin and Redis Commander
pnpm db:tools
```

- pgAdmin: http://localhost:5050 (admin@arcviz.local / admin)
- Redis Commander: http://localhost:8081

## 📁 Project Structure

```
arcviz-web/
├── apps/
│   ├── web/          # Next.js frontend
│   └── api/          # Fastify backend
├── packages/
│   ├── arcviz-core/  # Shared ArcViz logic
│   ├── arcviz-parser/# ArcLang parser
│   └── ui/           # Shared UI components
└── docker/           # Docker configuration
```

## 🛠️ Available Commands

```bash
# Development
pnpm dev              # Start all apps in dev mode
pnpm build            # Build all apps
pnpm start            # Start all apps in production mode
pnpm lint             # Lint all apps
pnpm test             # Run tests

# Database
pnpm db:up            # Start PostgreSQL + Redis
pnpm db:down          # Stop databases
pnpm db:reset         # Reset databases (⚠️ deletes data)
pnpm db:tools         # Start pgAdmin + Redis Commander
pnpm db:migrate       # Run Prisma migrations
pnpm db:studio        # Open Prisma Studio
pnpm db:seed          # Seed database with sample data

# Cleanup
pnpm clean            # Remove all build artifacts and node_modules
```

## 🐳 Docker Services

### PostgreSQL
- **Image**: postgres:16-alpine
- **Port**: 5432
- **User**: arcviz
- **Password**: arcviz_dev_password (change in production!)
- **Database**: arcviz

### Redis
- **Image**: redis:7-alpine
- **Port**: 6379
- **Password**: arcviz_redis_password (change in production!)
- **Persistence**: AOF enabled

### Health Checks
All services include health checks for reliability:
```bash
# Check service status
docker-compose ps
```

## 🔐 Security Notes

⚠️ **Important**: The default passwords are for development only!

For production:
1. Change all passwords in `.env`
2. Use strong, randomly generated secrets
3. Enable TLS/SSL for all services
4. Use Docker secrets or environment variable injection
5. Restrict network access with firewalls

## 📚 Documentation

- [Architecture Design](./docs/architecture.md)
- [API Documentation](./docs/api.md)
- [Deployment Guide](./docs/deployment.md)
- [Contributing Guidelines](./docs/contributing.md)

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](./docs/contributing.md).

## 📄 License

MIT License - see [LICENSE](./LICENSE) for details.

## 🙋 Support

- Documentation: [docs.arcviz.io](https://docs.arcviz.io)
- Issues: [GitHub Issues](https://github.com/mbaroudi/arcviz-web/issues)
- Discord: [Join our community](https://discord.gg/arcviz)

---

Built with ❤️ for Systems Engineers
