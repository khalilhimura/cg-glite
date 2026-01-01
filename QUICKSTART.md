# Quick Start Guide

Get the Agentic Memory System running in 5 minutes!

## Step 1: Prerequisites

Make sure you have:
- ✅ Rust installed (check with `rustc --version`)
- ✅ An Anthropic or OpenAI API key

If you don't have Rust, install it from [rustup.rs](https://rustup.rs/)

## Step 2: Configure API Key

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env and add your API key
# For Anthropic (recommended):
echo "LLM_PROVIDER=anthropic" > .env
echo "ANTHROPIC_API_KEY=your_key_here" >> .env
echo "LLM_MODEL=claude-3-5-sonnet-20241022" >> .env
```

## Step 3: Build and Run

```bash
# Build the project (first time will take a few minutes)
cargo build --release

# Run the assistant
cargo run --release
```

You should see:

```
╔══════════════════════════════════════════════════════════╗
║                                                          ║
║              🧠 AGENTIC MEMORY SYSTEM 🧠                 ║
║                                                          ║
║        AI Assistant with Context Graph Memory            ║
║              Powered by GraphLite                        ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝

Initializing agentic memory system...
Started conversation: [conversation-id]
Type your message and press Enter. Use 'exit' or 'quit' to end.

You:
```

## Step 4: Try It Out

Type a message and watch the agent extract entities and respond with context!

```
You: My colleague Alice is working on the GraphQL API project
[Extracted: People: Alice | Topics: GraphQL API]
Assistant: Got it! I've noted that Alice is working on the GraphQL API project...

You: What is Alice working on?
Assistant: Alice is working on the GraphQL API project, which you mentioned earlier.
```

## Next Steps

- Read [README.md](./README.md) for full documentation
- Read [PRD.md](./PRD.md) for design rationale
- Explore the source code in `src/`
- Try asking about people, topics, and relationships!

## Troubleshooting

**"ANTHROPIC_API_KEY not set"**
- Make sure you created `.env` file and added your API key

**Build errors**
- Run `cargo update` to update dependencies
- Make sure you're using Rust 1.70+

**GraphLite errors**
- The database will be created automatically in `./data/memory.db`
- If you see permissions errors, check that the directory is writable

## What Makes This Special?

Unlike chatbots that forget or use simple vector search:

1. **Remembers Relationships**: Knows that Alice works on GraphQL, Bob knows Alice
2. **Graph Queries**: Finds connections like "people who work on similar projects"
3. **Persistent**: Everything saved locally in embedded graph database
4. **Privacy-First**: No external memory services, all data stays on your machine
5. **Context-Aware**: Uses graph structure, not just similarity, to find relevant context

Enjoy exploring the power of Context Graphs for Agentic AI! 🚀
