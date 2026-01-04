# Agentic Memory System

> An AI assistant with persistent context graph memory powered by GraphLite

**Repository**: https://github.com/khalilhimura/cg-glite

This is a sample application demonstrating how [GraphLite](https://github.com/GraphLite-AI/GraphLite) enables next-generation AI agents through the **Context Graph paradigm**. Unlike traditional vector-based RAG systems, this agent maintains rich, queryable relationships between entities, enabling deeper understanding and more intelligent context retrieval.

## What is a Context Graph?

A **Context Graph** is a living representation of an AI agent's knowledge that captures:

- **Entities**: People, topics, tasks, documents, and events
- **Relationships**: How entities connect (knows, works_on, related_to, mentioned_in)
- **Temporal Patterns**: When things happened and how they evolved
- **Semantic Structure**: Hierarchies and dependencies

This enables agents to:
- Traverse relationships ("find all tasks related to people working on Project X")
- Understand causality ("what events led to this decision?")
- Build knowledge incrementally over multiple interactions
- Query complex patterns using GQL instead of just semantic similarity

## Features

- **Persistent Memory**: All conversations stored in embedded GraphLite database
- **Entity Extraction**: Automatic extraction of people, topics, tasks, and documents using LLMs
- **Context-Aware Responses**: Agent retrieves relevant context from graph before responding
- **Interactive CLI**: Beautiful terminal interface with entity highlighting
- **Privacy-First**: All data stored locally, no external memory services
- **Flexible LLM Support**: Works with Anthropic, OpenAI, or OpenRouter (access to 200+ models)

## Architecture

```
┌─────────────────────────────────────────────┐
│           AI Agent Layer                     │
│  (LLM Integration + Orchestration)          │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│      Context Graph Manager                   │
│  - Entity Extraction                         │
│  - Relationship Mapping                      │
│  - Query Construction                        │
│  - Memory Retrieval                          │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│           GraphLite Database                 │
│  - Embedded Graph Storage                    │
│  - ISO GQL Query Engine                      │
│  - ACID Transactions                         │
│  - Pattern Matching                          │
└─────────────────────────────────────────────┘
```

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- API key for one of:
  - Anthropic Claude API
  - OpenAI API
  - OpenRouter API (unified access to 200+ models from multiple providers)

## Installation

1. Clone this repository:
```bash
git clone https://github.com/khalilhimura/cg-glite.git
cd cg-glite
```

2. Copy the example environment file and configure it:
```bash
cp .env.example .env
```

3. Edit `.env` and add your API key:
```bash
# For Anthropic Claude (recommended)
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=your_api_key_here
LLM_MODEL=claude-3-5-sonnet-20241022

# Or for OpenAI
# LLM_PROVIDER=openai
# OPENAI_API_KEY=your_api_key_here
# LLM_MODEL=gpt-4-turbo-preview

# Or for OpenRouter (access to 200+ models)
# LLM_PROVIDER=openrouter
# OPENROUTER_API_KEY=your_api_key_here
# LLM_MODEL=anthropic/claude-3.5-sonnet
# OPENROUTER_APP_NAME=MyApp  # Optional: for app tracking
# OPENROUTER_SITE_URL=https://myapp.com  # Optional: for app ranking
```

4. Build the project:
```bash
cargo build --release
```

## Usage

Run the agentic memory assistant:

```bash
cargo run --release
```

Or use the compiled binary:

```bash
./target/release/agentic-memory
```

### Command Line Options

```bash
# Specify custom database path
cargo run -- --db-path ./custom/path/memory.db

# Set conversation title
cargo run -- --title "Project Planning Session"

# Custom credentials
cargo run -- --user admin --password secret123
```

### Example Interaction

```
You: My colleague Alice is working on the new GraphQL API project
[Extracted: People: Alice | Topics: GraphQL API]
Assistant: Got it! I've noted that Alice is working on the GraphQL API project.
I'll remember this for our future conversations.

You: What is Alice working on?
Assistant: Alice is working on the GraphQL API project, which you mentioned earlier.

You: Bob knows Alice and he's interested in machine learning
[Extracted: People: Bob, Alice | Topics: machine learning]
Assistant: I've connected Bob to Alice and noted Bob's interest in machine learning.

You: Who do I know that's interested in ML?
Assistant: Bob is interested in machine learning, and he knows Alice who's
working on the GraphQL API project. Would you like me to tell you more about
what I know regarding machine learning topics?
```

## Project Structure

```
agentic-memory/
├── src/
│   ├── main.rs              # CLI entry point and interactive REPL
│   ├── graph/
│   │   ├── mod.rs           # Graph module exports
│   │   ├── schema.rs        # Graph schema and entity definitions
│   │   └── operations.rs    # GraphLite database operations
│   ├── llm/
│   │   ├── mod.rs           # LLM module exports
│   │   ├── client.rs        # LLM API client (OpenAI/Anthropic)
│   │   └── extraction.rs    # Entity extraction logic
│   └── agent/
│       ├── mod.rs           # Agent module exports
│       ├── memory.rs        # Agentic memory orchestration
│       └── retrieval.rs     # Context retrieval strategies
├── Cargo.toml               # Rust dependencies
├── .env.example             # Environment configuration template
├── PRD.md                   # Product Requirements Document
└── README.md                # This file
```

## Graph Schema

The context graph uses the following schema:

### Node Types

- **Conversation**: Chat sessions with timestamps
- **Message**: Individual messages (user/assistant)
- **Person**: People mentioned in conversations
- **Topic**: Subjects, technologies, concepts discussed
- **Task**: Action items and todos
- **Document**: Files, links, resources referenced

### Relationship Types

- **PART_OF**: Message is part of a Conversation
- **MENTIONED_IN**: Entity mentioned in a Message
- **RELATES_TO**: Semantic connection between entities
- **KNOWS**: Connection between people
- **WORKS_ON**: Person working on a Topic or Task

## How It Works

1. **User Input**: You type a message in the CLI
2. **Entity Extraction**: LLM analyzes the message and extracts entities (people, topics, tasks, documents)
3. **Graph Storage**: Message and entities are stored in GraphLite with relationships
4. **Context Retrieval**: Before responding, agent queries the graph for relevant context
5. **Response Generation**: LLM generates response using retrieved context
6. **Memory Update**: Assistant's response is also stored in the graph

### Key Differentiators vs. Vector RAG

| Feature | Traditional Vector RAG | Context Graph (This) |
|---------|----------------------|---------------------|
| Storage | Embeddings in vector DB | Structured graph with relationships |
| Retrieval | Cosine similarity search | Pattern matching with GQL |
| Relationships | None (similarity only) | Explicit relationships tracked |
| Temporal Understanding | Limited | Full temporal tracking |
| Query Complexity | Simple similarity | Multi-hop traversal, complex patterns |
| Knowledge Evolution | Replace chunks | Incremental graph updates |

## Why GraphLite?

This project uses GraphLite because:

1. **Embedded**: No server to run, works like SQLite
2. **Standards-Based**: Implements ISO GQL standard
3. **Lightweight**: 11 MB binary, minimal dependencies
4. **ACID Compliant**: Data integrity guaranteed
5. **Pattern Matching**: Powerful graph queries
6. **Rust Performance**: Memory-safe and fast

## Development

### Running Tests

```bash
cargo test
```

### Running in Development Mode

```bash
cargo run
```

### Enabling Debug Logging

The application uses the `anyhow` crate for error handling. Errors are displayed with full context chains.

## Extending the Application

### Adding New Entity Types

1. Define the struct in `src/graph/schema.rs`
2. Update `ExtractedEntities` to include the new type
3. Modify the extraction prompt in `src/llm/extraction.rs`
4. Add graph operations in `src/graph/operations.rs`

### Adding New Relationship Types

1. Use `INSERT (entity1)-[:NEW_RELATIONSHIP]->(entity2)` in GQL queries
2. Update context retrieval logic in `src/agent/memory.rs`

### Custom Query Patterns

Add new query methods in `src/agent/retrieval.rs` using GQL pattern matching:

```rust
pub fn custom_query(&self, session: &Session) -> Result<String> {
    let query = "MATCH (p:Person)-[:WORKS_ON]->(t:Topic)<-[:WORKS_ON]-(p2:Person) \
                 WHERE p <> p2 \
                 RETURN p.name, t.name, p2.name";
    // Execute and process results
}
```

## Limitations & Future Work

### Current Limitations

- GraphLite result parsing is simplified (may need adjustment based on actual API)
- No graph visualization (CLI only)
- Single-user mode only
- Limited error recovery in graph operations

### Potential Enhancements

- [ ] Add graph visualization using D3.js or Graphviz
- [ ] Implement graph summarization for large contexts
- [ ] Add export/import capabilities (JSON, GraphML)
- [ ] Multi-user support with access control
- [ ] Integration with LangChain, AutoGen, or other agent frameworks
- [ ] Implement more sophisticated entity linking (coreference resolution)
- [ ] Add support for document embeddings alongside graph structure
- [ ] Implement graph-based recommendation system

## Contributing

This is a sample/demo application. Feel free to:

- Fork and extend for your own use cases
- Report issues or suggest improvements
- Submit pull requests with enhancements

## License

MIT License - feel free to use this code for your own projects.

## Acknowledgments

- [GraphLite](https://github.com/GraphLite-AI/GraphLite) - Embedded graph database
- [Anthropic](https://anthropic.com) - Claude AI API
- [OpenAI](https://openai.com) - GPT API
- [OpenRouter](https://openrouter.ai) - Unified LLM API with access to 200+ models

## Related Resources

- [PRD.md](./PRD.md) - Full Product Requirements Document
- [GraphLite Documentation](https://github.com/GraphLite-AI/GraphLite)
- [ISO GQL Standard](https://www.iso.org/standard/76120.html)

## Support

For issues related to:
- **This sample app**: Open an issue at https://github.com/khalilhimura/cg-glite/issues
- **GraphLite**: Visit the [GraphLite repository](https://github.com/GraphLite-AI/GraphLite)
- **LLM APIs**: Check Anthropic or OpenAI documentation

---

Built with ❤️ to showcase the power of Context Graphs for Agentic AI