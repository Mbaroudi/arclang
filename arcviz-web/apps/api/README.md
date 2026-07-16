# ArcViz API

Fastify-based REST API for the ArcViz MBSE platform.

## Tech Stack

- **Fastify** - Fast and low overhead web framework
- **Prisma** - Type-safe database ORM
- **PostgreSQL** - Primary database
- **Redis** - Caching and session storage
- **JWT** - Authentication
- **TypeScript** - Type safety

## Setup

### 1. Install Dependencies

```bash
pnpm install
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your configuration
```

### 3. Run Database Migrations

```bash
pnpm prisma:generate
pnpm prisma:migrate
```

### 4. Start Development Server

```bash
pnpm dev
```

The API will be available at `http://localhost:4000`

## API Endpoints

### Authentication

#### Register
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "name": "John Doe"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### Get Current User
```http
GET /api/auth/me
Authorization: Bearer <token>
```

#### Logout
```http
POST /api/auth/logout
Authorization: Bearer <token>
```

### Projects

#### List Projects
```http
GET /api/projects
Authorization: Bearer <token>
```

#### Get Project
```http
GET /api/projects/:id
Authorization: Bearer <token>
```

#### Create Project
```http
POST /api/projects
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My MBSE Project",
  "description": "Project description",
  "visibility": "PRIVATE"
}
```

#### Update Project
```http
PATCH /api/projects/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Name",
  "description": "Updated description"
}
```

#### Delete Project
```http
DELETE /api/projects/:id
Authorization: Bearer <token>
```

### Files

#### List Files in Project
```http
GET /api/files/project/:projectId
Authorization: Bearer <token>
```

#### Get File
```http
GET /api/files/:id
Authorization: Bearer <token>
```

#### Create File
```http
POST /api/files?projectId=<projectId>
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "main.arc",
  "path": "/src/main.arc",
  "content": "operational_analysis {\n  // ArcLang code\n}",
  "language": "arclang"
}
```

#### Update File
```http
PATCH /api/files/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "// Updated code"
}
```

#### Delete File
```http
DELETE /api/files/:id
Authorization: Bearer <token>
```

### Compilation

#### Compile Code
```http
POST /api/compile
Authorization: Bearer <token>
Content-Type: application/json

{
  "code": "operational_analysis { ... }",
  "fileId": "optional-file-id"
}
```

Response:
```json
{
  "success": true,
  "output": "Compilation output",
  "stats": {
    "requirements": 5,
    "components": 8,
    "functions": 12,
    "traces": 3
  },
  "diagram": {
    "nodes": [...],
    "edges": [...],
    "layer": "logical"
  },
  "compilationId": "compilation-id"
}
```

#### Validate Code
```http
POST /api/compile/validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "code": "operational_analysis { ... }"
}
```

#### Get Compilation History
```http
GET /api/compile/history/:fileId
Authorization: Bearer <token>
```

### Diagrams

#### List Diagrams in Project
```http
GET /api/diagrams/project/:projectId
Authorization: Bearer <token>
```

#### Get Diagram
```http
GET /api/diagrams/:id
Authorization: Bearer <token>
```

#### Delete Diagram
```http
DELETE /api/diagrams/:id
Authorization: Bearer <token>
```

## Database Schema

### User
- `id` - Unique identifier
- `email` - Email address (unique)
- `passwordHash` - Hashed password
- `name` - User name
- `role` - USER | ADMIN
- `avatarUrl` - Profile picture URL
- `createdAt` - Creation timestamp
- `updatedAt` - Update timestamp
- `lastLoginAt` - Last login timestamp

### Project
- `id` - Unique identifier
- `name` - Project name
- `description` - Project description
- `visibility` - PRIVATE | PUBLIC | ORGANIZATION
- `ownerId` - User ID of owner
- `createdAt` - Creation timestamp
- `updatedAt` - Update timestamp

### File
- `id` - Unique identifier
- `name` - File name
- `path` - File path within project
- `content` - File content
- `language` - Programming language (default: arclang)
- `projectId` - Project ID
- `authorId` - User ID of author
- `createdAt` - Creation timestamp
- `updatedAt` - Update timestamp

### Compilation
- `id` - Unique identifier
- `success` - Compilation success flag
- `output` - Compilation output
- `errors` - Error messages
- `warnings` - Warning messages
- `stats` - Compilation statistics (JSON)
- `fileId` - File ID
- `createdAt` - Creation timestamp

### Diagram
- `id` - Unique identifier
- `name` - Diagram name
- `layer` - Architecture layer (OPERATIONAL_ANALYSIS | SYSTEM_ANALYSIS | LOGICAL_ARCHITECTURE | PHYSICAL_ARCHITECTURE | EPBS)
- `nodes` - Node data (JSON)
- `edges` - Edge data (JSON)
- `layout` - Layout data (JSON)
- `projectId` - Project ID
- `compilationId` - Compilation ID (optional)
- `createdAt` - Creation timestamp
- `updatedAt` - Update timestamp

## ArcLang Compiler Integration

The API integrates with the ArcLang compiler located at `/Users/malek/Arclang/target/release/arclang`.

### Compiler Configuration

Set the compiler path in `.env`:
```
ARCLANG_COMPILER_PATH=/Users/malek/Arclang/target/release/arclang
```

### Compilation Process

1. **Write code to temp file** - Create temporary `.arc` file
2. **Execute compiler** - Run `arclang compile input.arc --output output.json --format json`
3. **Parse output** - Extract nodes, edges, stats from JSON
4. **Store results** - Save compilation record and diagram to database
5. **Clean up** - Delete temporary files

### Compiler Output Format

```json
{
  "architecture": {
    "layer": "logical",
    "components": [
      {
        "id": "COMP-001",
        "name": "Component Name",
        "description": "...",
        "safety_level": "ASIL_B"
      }
    ],
    "functions": [...],
    "requirements": [...],
    "traces": [
      {
        "from": "COMP-001",
        "to": "REQ-001",
        "type": "satisfies",
        "label": "..."
      }
    ]
  }
}
```

## Redis Caching

Redis is used for:
- Session storage
- Compilation result caching
- Rate limiting
- Real-time collaboration (future)

### Cache Keys

- `session:{token}` - User session data
- `compilation:{fileId}:{hash}` - Compilation results
- `diagram:{id}` - Diagram data

## Error Handling

All endpoints return consistent error responses:

```json
{
  "error": "Error message"
}
```

HTTP Status Codes:
- `200` - Success
- `201` - Created
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `500` - Internal Server Error

## Security

- **JWT Authentication** - All protected routes require valid JWT token
- **Password Hashing** - bcryptjs with 10 rounds
- **CORS** - Configured for `http://localhost:3002`
- **Input Validation** - Zod schemas for all inputs
- **SQL Injection Protection** - Prisma parameterized queries
- **Audit Logging** - All mutations logged to `audit_logs` table

## Development

### Run Migrations

```bash
pnpm prisma:migrate
```

### Generate Prisma Client

```bash
pnpm prisma:generate
```

### Open Prisma Studio

```bash
pnpm prisma:studio
```

### Type Checking

```bash
pnpm type-check
```

### Linting

```bash
pnpm lint
```

## Production Deployment

### Build

```bash
pnpm build
```

### Start

```bash
pnpm start
```

### Environment Variables

Required environment variables for production:
- `NODE_ENV=production`
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_HOST`, `REDIS_PORT`, `REDIS_PASSWORD` - Redis configuration
- `JWT_SECRET` - Strong random secret for JWT signing
- `JWT_REFRESH_SECRET` - Strong random secret for refresh tokens
- `CORS_ORIGIN` - Frontend URL
- `ARCLANG_COMPILER_PATH` - Path to ArcLang compiler binary

## Health Check

```http
GET /health
```

Response:
```json
{
  "status": "ok",
  "timestamp": "2025-01-15T10:30:00.000Z"
}
```

## Logging

Fastify logger is configured with:
- **Development**: `info` level with pretty printing
- **Production**: `warn` level with JSON output

## Performance

- **Connection pooling** - PostgreSQL connection pool
- **Redis caching** - Compilation results cached
- **Async/await** - Non-blocking I/O
- **Type safety** - Full TypeScript coverage

## Next Steps

1. **Add WebSocket support** - Real-time collaboration
2. **Implement rate limiting** - Redis-based rate limiter
3. **Add file upload** - Support for importing Capella XML
4. **Add export** - Generate PDF, SVG, Capella XML
5. **Add AI assistant** - OpenAI/Claude integration
6. **Add RBAC** - Role-based access control
7. **Add organization support** - Multi-user projects
8. **Add version control** - Git-like file versioning

---

For more information, see the main [project README](../../README.md).
