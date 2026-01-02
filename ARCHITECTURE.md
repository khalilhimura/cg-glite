# Architecture Deep Dive

**Repository**: https://github.com/khalilhimura/cg-glite

## Overview

The Agentic Memory System demonstrates the **Context Graph paradigm** - a novel approach to AI agent memory that goes beyond traditional vector-based RAG systems.

## Core Concept: Context Graph vs. Vector RAG

### Traditional Vector RAG
```
User Query → Embedding → Vector Similarity Search → Retrieve Top-K Chunks → LLM
```

**Limitations:**
- Only finds "similar" content
- No understanding of relationships
- Can't traverse connections
- Limited temporal awareness
- Requires expensive embedding operations

### Context Graph (This System)
```
User Query → Entity Extraction → Graph Pattern Matching → Retrieve Connected Context → LLM
```

**Advantages:**
- Explicit relationship tracking
- Multi-hop graph traversal
- Temporal ordering of events
- Complex pattern queries
- Incremental knowledge building

## System Components

### 1. Graph Layer (`src/graph/`)

**Purpose**: Manages the GraphLite database and graph operations

**schema.rs**:
- Defines entity types (Person, Topic, Task, Document, Message, Conversation)
- Structures for extracted entities
- Helper functions for IDs and timestamps

**operations.rs**:
- `GraphDB` struct wraps GraphLite SDK
- CRUD operations for conversations and messages
- Entity linking logic
- GQL query construction
- Relationship creation

**Key Operations**:
```rust
start_conversation() // Creates new conversation node
add_message()        // Adds message with extracted entities
link_entities()      // Creates entity nodes and relationships
find_related_entities() // Graph traversal queries
```

### 2. LLM Layer (`src/llm/`)

**Purpose**: Interfaces with LLM APIs for entity extraction and response generation

**client.rs**:
- `LLMClient` supports both Anthropic and OpenAI
- Async HTTP requests to LLM APIs
- Response parsing and error handling
- Provider abstraction layer

**extraction.rs**:
- `EntityExtractor` uses structured LLM output
- Parses JSON responses into `ExtractedEntities`
- Robust error handling for malformed JSON
- Extraction prompt engineering

**Entity Extraction Flow**:
1. User message sent to LLM with extraction prompt
2. LLM returns JSON with people, topics, tasks, documents
3. Parser extracts arrays into structured format
4. Returns `ExtractedEntities` for graph storage

### 3. Agent Layer (`src/agent/`)

**Purpose**: Orchestrates memory management and context retrieval

**memory.rs**:
- `AgenticMemory` is the main orchestrator
- Manages graph DB and LLM client
- Processes user messages (extract + store)
- Generates responses with graph context
- Builds context strings from graph queries

**retrieval.rs**:
- `ContextRetriever` implements query patterns
- Methods for person/topic context
- Conversation history retrieval
- Future: complex graph traversals

**Memory Flow**:
```
User Message
    ↓
Extract Entities (LLM)
    ↓
Store in Graph (Message + Entities + Relationships)
    ↓
Query Graph for Context (GQL pattern matching)
    ↓
Generate Response with Context (LLM)
    ↓
Store Response in Graph
```

### 4. CLI Layer (`src/main.rs`)

**Purpose**: Interactive user interface

**Features**:
- Command-line argument parsing (clap)
- Interactive REPL (rustyline)
- Colored output for better UX
- Entity extraction visualization
- Error handling and user feedback

**Flow**:
1. Parse args and load environment
2. Initialize LLM client (Anthropic/OpenAI)
3. Initialize AgenticMemory
4. Start conversation
5. REPL loop:
   - Read user input
   - Process message (extract + store + retrieve + respond)
   - Display response with entity highlights
   - Repeat

## Data Flow

### Complete Interaction Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Input                              │
│                "Alice is working on GraphQL"                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                   Entity Extraction (LLM)                       │
│     Input: User message                                         │
│     Output: {people: ["Alice"], topics: ["GraphQL"], ...}       │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    Graph Storage (GraphLite)                    │
│                                                                 │
│  1. CREATE (:Message {content: "...", role: "user"})           │
│  2. MATCH (c:Conversation {id: "..."})                          │
│     INSERT (m)-[:PART_OF]->(c)                                  │
│  3. INSERT (:Person {name: "Alice"})                            │
│  4. INSERT (p)-[:MENTIONED_IN]->(m)                             │
│  5. INSERT (:Topic {name: "GraphQL"})                           │
│  6. INSERT (t)-[:MENTIONED_IN]->(m)                             │
│  7. (Future) INSERT (p)-[:WORKS_ON]->(t)                        │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                  Context Retrieval (Graph Queries)              │
│                                                                 │
│  MATCH (t:Topic {name: "GraphQL"})-[:MENTIONED_IN]->            │
│        (m:Message)<-[:MENTIONED_IN]-(e)                         │
│  RETURN e.name                                                  │
│                                                                 │
│  Result: ["Alice", ...other related entities]                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                  Response Generation (LLM)                      │
│     Input: User message + Graph context                         │
│     Output: "Got it! Alice is working on GraphQL..."            │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              Store Assistant Response (Graph)                   │
│  INSERT (:Message {content: "...", role: "assistant"})         │
└─────────────────────────────────────────────────────────────────┘
```

## Graph Schema Details

### Node Properties

**Conversation**:
```rust
{
    id: String,           // UUID
    started_at: DateTime, // RFC3339 timestamp
    title: Option<String> // User-provided or auto-generated
}
```

**Message**:
```rust
{
    id: String,           // UUID
    role: String,         // "user" or "assistant"
    content: String,      // Message text
    timestamp: DateTime   // RFC3339 timestamp
}
```

**Person**:
```rust
{
    name: String,              // Person's name
    description: Option<String> // Additional context
}
```

**Topic**:
```rust
{
    name: String,               // Topic name
    category: Option<String>    // Topic categorization
}
```

**Task**:
```rust
{
    description: String,  // Task description
    status: String,       // "pending", "in_progress", "completed"
    created_at: DateTime  // RFC3339 timestamp
}
```

### Relationship Types

All relationships are directional and typed:

- `PART_OF`: Message → Conversation
- `MENTIONED_IN`: Entity → Message
- `RELATES_TO`: Entity → Entity (semantic similarity)
- `KNOWS`: Person → Person (acquaintance)
- `WORKS_ON`: Person → Topic/Task (activity)
- `DEPENDS_ON`: Task → Task (prerequisites)

### Example Graph Structure

```
(Conversation:1)
    ↑ PART_OF
(Message:1 "Alice works on GraphQL")
    ↑ MENTIONED_IN     ↑ MENTIONED_IN
(Person:Alice)       (Topic:GraphQL)
    ↓ WORKS_ON
(Topic:GraphQL)

Later:
(Message:2 "What is Alice working on?")
    ↑ MENTIONED_IN
(Person:Alice) [already exists]

Query: MATCH (p:Person {name: "Alice"})-[:WORKS_ON]->(t:Topic)
Result: GraphQL
```

## GQL Query Patterns

### Basic Patterns

**Find all people mentioned**:
```gql
MATCH (p:Person)-[:MENTIONED_IN]->(m:Message)
RETURN DISTINCT p.name
```

**Find topics related to a person**:
```gql
MATCH (p:Person {name: "Alice"})-[:MENTIONED_IN]->(m:Message)
      <-[:MENTIONED_IN]-(t:Topic)
RETURN DISTINCT t.name
```

**Recent conversation history**:
```gql
MATCH (m:Message)-[:PART_OF]->(c:Conversation {id: "conv-123"})
RETURN m.role, m.content, m.timestamp
ORDER BY m.timestamp DESC
LIMIT 10
```

### Advanced Patterns

**Multi-hop: Find people working on related topics**:
```gql
MATCH (p1:Person)-[:WORKS_ON]->(t:Topic)
      <-[:WORKS_ON]-(p2:Person)
WHERE p1 <> p2
RETURN p1.name, t.name, p2.name
```

**Temporal: Topics discussed in time range**:
```gql
MATCH (t:Topic)-[:MENTIONED_IN]->(m:Message)
WHERE m.timestamp > "2024-01-01T00:00:00Z"
  AND m.timestamp < "2024-12-31T23:59:59Z"
RETURN t.name, COUNT(m) as mention_count
ORDER BY mention_count DESC
```

**Transitive: Friends of friends**:
```gql
MATCH (p1:Person {name: "Alice"})-[:KNOWS*1..2]-(p2:Person)
RETURN DISTINCT p2.name
```

## LLM Integration Patterns

### Entity Extraction Prompt

```
You are an expert entity extractor for an AI agent's memory system.
Extract the following types of entities from the user's message:

1. PEOPLE: Names of individuals mentioned
2. TOPICS: Subjects, concepts, technologies, projects, or areas of interest
3. TASKS: Action items, todos, or work that needs to be done
4. DOCUMENTS: Files, links, resources, or references mentioned

Return your response as a JSON object with this exact structure:
{
  "people": ["name1", "name2"],
  "topics": ["topic1", "topic2"],
  "tasks": ["task1", "task2"],
  "documents": ["doc1", "doc2"]
}
```

### Context-Aware Response Prompt

```
You are a helpful AI assistant with persistent memory powered by a context graph.

CONTEXT FROM YOUR MEMORY:
{graph_context}

Use this context to provide informed, personalized responses...
```

## Performance Characteristics

### GraphLite

- **Database Size**: ~50MB for 1000 conversations (estimated)
- **Query Latency**: <100ms for typical pattern matches
- **Write Latency**: <50ms for message + entity storage
- **Memory Usage**: Minimal (embedded, no server)

### LLM Calls

- **Entity Extraction**: 1 call per user message (~500ms)
- **Response Generation**: 1 call per user message (~1-2s)
- **Total Latency**: ~2-3s per interaction

### Optimization Opportunities

1. **Batch Entity Creation**: Group INSERTs in single transaction
2. **Caching**: Cache recent extractions for similar messages
3. **Async Queries**: Parallel entity linking and context retrieval
4. **Streaming**: Stream LLM responses while storing in graph

## Security Considerations

### Current Security Posture

✅ **Strengths**:
- All data stored locally (no cloud exposure)
- ACID transactions prevent corruption
- Input escaping for GQL queries
- API keys via environment variables

⚠️ **Limitations**:
- No authentication beyond basic DB credentials
- GQL injection possible if escaping fails
- No encryption at rest
- No rate limiting on LLM calls

### Production Hardening

For production use, consider:
- Encrypt database files at rest
- Add proper authentication/authorization
- Implement GQL query parameterization
- Add rate limiting and cost tracking
- Sanitize all user inputs
- Add audit logging

## Extension Points

### Adding Custom Entity Types

1. Define struct in `schema.rs`
2. Update `ExtractedEntities`
3. Modify extraction prompt
4. Add linking logic in `operations.rs`
5. Update context building in `memory.rs`

### Adding Custom Queries

1. Define method in `retrieval.rs`
2. Implement GQL pattern
3. Parse results
4. Integrate into context building

### Integrating with Frameworks

**LangChain**:
```rust
// Create GraphLite memory backend
impl BaseChatMemory for AgenticMemory {
    fn get_history() -> Vec<Message> { ... }
    fn add_message(msg: Message) { ... }
}
```

**AutoGen**:
```python
# Python bindings wrapper
class GraphLiteMemory:
    def __init__(self, db_path):
        self.rust_lib = load_rust_library()
```

## Testing Strategy

### Unit Tests

- Entity extraction parsing
- GQL query construction
- String escaping
- Error handling

### Integration Tests

- Full message flow (extract → store → retrieve → respond)
- Graph query patterns
- LLM integration (mocked)

### Manual Tests

- Multi-turn conversations
- Complex relationship queries
- Error recovery
- Performance under load

## Future Directions

### Short Term

- Implement KNOWS relationships automatically
- Add WORKS_ON inference from context
- Improve entity deduplication
- Add conversation summaries

### Medium Term

- Graph visualization dashboard
- Export/import capabilities
- Multi-user support
- Query DSL for easier retrieval

### Long Term

- Hybrid graph + vector search
- Automatic relationship inference using LLMs
- Graph neural networks for recommendations
- Federated graph queries across multiple databases

---

This architecture showcases how embedded graph databases like GraphLite enable sophisticated agentic memory systems with rich, queryable context - a key capability for next-generation AI assistants.
