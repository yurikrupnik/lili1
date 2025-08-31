# Zerg - Full Stack AI Agent Application

A full-stack application combining a TypeScript LangGraph agent, SolidJS UI, and Rust API.

## Architecture

- **UI (SolidJS)**: Frontend with two chat interfaces - regular HTTP and streaming
- **API (Rust)**: Backend API that handles HTTP requests and communicates with the LangGraph agent
- **Agent (TypeScript)**: LangGraph-based AI agent with supervisor pattern managing multiple specialized agents

## Features

### Two Chat Interfaces

1. **Regular Chat**: Traditional request/response pattern
2. **Streaming Chat**: Real-time streaming responses with Server-Sent Events (SSE)

## Quick Start

### Prerequisites

- Bun (for TypeScript/SolidJS)
- Rust and Cargo
- Node.js (for TypeScript agent)

### Running All Services

Use the Nu script to start all services at once:

```bash
./start-all.nu
```

Or start services individually:

### 1. Start the LangGraph Agent

```bash
cd ts-agent
bun run dev:server
```

The agent will be available at `http://localhost:3001`

### 2. Start the Rust API

```bash
cd api
cargo run
```

The API will be available at `http://localhost:8080`

### 3. Start the SolidJS UI

```bash
cd ui
bun run dev
```

The UI will be available at `http://localhost:4173`

## API Endpoints

### Rust API

- `POST /api/chat` - Regular chat endpoint
- `POST /api/chat/stream` - Streaming chat endpoint
- `GET /tasks` - Sample task endpoint
- `GET /metrics` - Prometheus metrics

### LangGraph Agent

- `POST /chat` - Main chat endpoint with streaming support

## Environment Configuration

Make sure to set up your environment variables for the AI models:

```bash
# For OpenAI (required for development agent)
OPENAI_API_KEY=your_openai_key

# For Google GenAI (required for knowledge base agent)
GOOGLE_API_KEY=your_google_key
```

## Development

Each service can be developed independently:

- **UI**: Uses Rsbuild with hot reload
- **API**: Uses Cargo with automatic recompilation
- **Agent**: Uses Bun with watch mode

## Testing

Run tests for individual components:

```bash
# TypeScript agent tests
cd ts-agent
bun test

# Rust API tests
cd api
cargo test
```