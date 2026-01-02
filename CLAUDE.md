# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an **Agentic Memory System** - a sample application demonstrating the Context Graph paradigm using GraphLite. Unlike traditional vector-based RAG systems, this agent maintains rich, queryable relationships between entities (people, topics, tasks, documents) in an embedded graph database.

**Key Concept**: The system extracts entities from user messages, stores them in a GraphLite graph with explicit relationships, then retrieves relevant context via GQL pattern matching (not vector similarity) before generating responses.

## Development Commands

### Building and Running

```bash
# Development build and run
cargo run

# Production build
cargo build --release

# Run production binary
./target/release/agentic-memory

# With custom options
cargo run -- --db-path ./custom/memory.db --title "Session Title"
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Environment Setup

1. Copy `.env.example` to `.env`
2. Set `LLM_PROVIDER` (anthropic or openai)
3. Add corresponding API key (`ANTHROPIC_API_KEY` or `OPENAI_API_KEY`)
4. Set `LLM_MODEL` (e.g., `claude-3-5-sonnet-20241022`)

## Architecture

### Three-Layer Architecture

1. **Graph Layer** (`src/graph/`):
   - `schema.rs`: Entity types (Person, Topic, Task, Document, Message, Conversation)
   - `operations.rs`: GraphDB wrapper, CRUD operations, GQL query construction
   - Uses GraphLite embedded database (no server needed)

2. **LLM Layer** (`src/llm/`):
   - `client.rs`: Unified client for Anthropic/OpenAI APIs
   - `extraction.rs`: Structured entity extraction from user messages
   - Returns JSON with `{people: [], topics: [], tasks: [], documents: []}`

3. **Agent Layer** (`src/agent/`):
   - `memory.rs`: AgenticMemory orchestrator - main entry point
   - `retrieval.rs`: Context retrieval strategies using GQL patterns
   - Coordinates: extract → store → retrieve context → respond

### Data Flow

```
User Input
  ↓ extract_entities()
LLM extracts {people, topics, tasks, docs}
  ↓ add_message()
Store Message node + Entity nodes + MENTIONED_IN relationships
  ↓ build_context()
Query graph for related entities via GQL pattern matching
  ↓ generate_response()
LLM responds with graph context
  ↓ add_message()
Store assistant response in graph
```

### Graph Schema

**Node Types**: Conversation, Message, Person, Topic, Task, Document

**Relationship Types**:
- `PART_OF`: Message → Conversation
- `MENTIONED_IN`: Entity → Message
- `RELATES_TO`: Entity → Entity
- `KNOWS`: Person → Person
- `WORKS_ON`: Person → Topic/Task

All nodes have UUIDs. Messages/Conversations have RFC3339 timestamps.

## Key Implementation Details

### GQL Query Construction

Queries are built as strings in `src/graph/operations.rs`:

```rust
let query = format!(
    "MATCH (p:Person {{name: '{}'}})-[:MENTIONED_IN]->(m:Message) RETURN m",
    person_name
);
```

**Important**: Input escaping is critical to prevent GQL injection. Single quotes in user input must be escaped.

### Entity Extraction Flow

1. User message → LLM with extraction prompt
2. LLM returns JSON: `{"people": ["Alice"], "topics": ["GraphQL"], ...}`
3. Parse JSON into `ExtractedEntities` struct
4. Create nodes and `MENTIONED_IN` relationships for each entity

See `src/llm/extraction.rs` for the extraction prompt template.

### Session Management

GraphLite requires username/password per session:
```rust
let session = graph_db.session("admin", "admin123")?;
```

Sessions are used for all queries/mutations. Default credentials are in `.env`.

## Common Patterns

### Adding a New Entity Type

1. Define struct in `src/graph/schema.rs`:
   ```rust
   pub struct Event {
       pub name: String,
       pub date: DateTime<Utc>,
   }
   ```

2. Add to `ExtractedEntities`:
   ```rust
   pub struct ExtractedEntities {
       // ... existing fields
       pub events: Vec<Event>,
   }
   ```

3. Update extraction prompt in `src/llm/extraction.rs` to include "events"

4. Add linking logic in `src/graph/operations.rs::add_message()`:
   ```rust
   for event in &entities.events {
       // INSERT event node and MENTIONED_IN relationship
   }
   ```

### Adding a Custom GQL Query

In `src/agent/retrieval.rs`:

```rust
pub fn find_collaborators(&self, session: &Session, person: &str) -> Result<Vec<String>> {
    let query = format!(
        "MATCH (p1:Person {{name: '{}'}})-[:WORKS_ON]->(t:Topic)<-[:WORKS_ON]-(p2:Person) \
         WHERE p1 <> p2 \
         RETURN DISTINCT p2.name",
        person
    );
    // Parse and return results
}
```

Use in `src/agent/memory.rs::build_context()` to enrich context.

## GraphLite Specifics

- **Installation**: `GraphLite::install(path, user, pass)` creates new DB
- **Opening**: `GraphLite::open(path)` opens existing DB
- **Queries**: `session.execute(gql_string)` returns `QueryResult`
- **Result Parsing**: Currently basic string parsing; may need enhancement based on GraphLite SDK updates

**Note**: GraphLite is embedded like SQLite - no separate server process. Database is a single file.

## LLM Provider Configuration

The `LLMClient` abstracts Anthropic and OpenAI:

```rust
match provider {
    LLMProvider::Anthropic => // Use Anthropic API format
    LLMProvider::OpenAI => // Use OpenAI API format
}
```

Both providers return text responses. Entity extraction expects JSON in response body.

## Testing Strategy

- **Unit Tests**: Entity parsing, GQL query construction, string escaping
- **Integration Tests**: Full flow with mocked LLM responses
- **Manual Testing**: Run CLI and test multi-turn conversations with relationship queries

No tests currently exist - this is a sample/demo application.

## Important Constraints

1. **Single-threaded GQL**: GraphLite operations are synchronous via sessions
2. **No Graph Visualization**: CLI only, no visual graph browser
3. **Basic Error Handling**: Uses `anyhow` for error propagation
4. **No Entity Deduplication**: Same entity name creates multiple nodes (limitation to address)

## Future Enhancement Areas

- Implement `KNOWS` and `WORKS_ON` relationship inference from context
- Entity deduplication/linking (detect "Alice" == "Alice Smith")
- Graph query result caching
- Streaming LLM responses while storing to graph
- Export/import graph data (GraphML, JSON)
