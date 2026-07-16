# ArcViz Web Frontend

Next.js 14 frontend application for ArcViz MBSE platform.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS + shadcn/ui
- **State Management**: Zustand + React Query
- **Editor**: Monaco Editor
- **Diagrams**: D3.js + ELK.js
- **Authentication**: NextAuth.js

## Getting Started

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Start production server
pnpm start
```

## Project Structure

```
apps/web/
├── app/                    # Next.js app directory
│   ├── (auth)/            # Auth routes (login, register)
│   ├── (dashboard)/       # Protected dashboard routes
│   │   ├── projects/      # Project management
│   │   ├── editor/        # ArcLang editor
│   │   └── visualizer/    # Architecture visualization
│   ├── api/               # API routes
│   ├── layout.tsx         # Root layout
│   ├── page.tsx           # Landing page
│   └── globals.css        # Global styles
├── components/
│   ├── ui/                # shadcn/ui components
│   ├── editor/            # Monaco editor components
│   ├── diagram/           # D3.js diagram components
│   └── providers/         # Context providers
├── lib/
│   ├── arcviz/            # ArcViz engine integration
│   ├── api/               # API client
│   └── utils.ts           # Utility functions
└── public/                # Static assets
```

## Features

- ✅ Modern, responsive UI with dark mode
- ✅ Real-time collaboration
- ✅ Monaco editor with ArcLang syntax
- ✅ Interactive architecture diagrams
- ✅ AI-powered assistant
- ✅ Project management
- ✅ Import/Export capabilities

## Development

```bash
# Lint code
pnpm lint

# Type check
pnpm type-check

# Format code
pnpm format
```

## Environment Variables

See `.env.example` in the root directory for required environment variables.

## License

MIT
