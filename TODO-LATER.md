# TODO-LATER: Manual Testing & Future Tasks

## Manual Testing Required: Query Result Parsing

### Overview
The query result parsing functions (`get_conversation_messages` and `find_related_entities`) have been implemented but require manual verification with a real GraphLite database to confirm column naming conventions and overall behavior.

### Testing Setup

1. **Start the application**:
   ```bash
   cargo run
   ```

2. **Create test database** (if not already present):
   - The application will create a GraphLite database at the configured path
   - Default location is typically `./memory.db` or similar

---

## Test 1: get_conversation_messages

### Objective
Verify that messages are correctly retrieved and parsed from conversations.

### Steps

1. **Create test conversation**:
   - Start a new conversation session
   - Send multiple messages (at least 3-5)
   - Include various content types (questions, statements, etc.)

2. **Verify storage**:
   - Check that messages are stored in the database
   - Verify entities are extracted and linked

3. **Test retrieval** (requires integration):
   - Currently `get_conversation_messages` is not called by the application
   - Options:
     - **A. Add debug call**: Temporarily add a call to this function in the agent code
     - **B. Create integration test**: Write a test that creates data and retrieves it
     - **C. Use GraphLite CLI**: Query the database directly to verify structure

4. **Expected behavior**:
   - Messages returned as `Vec<(role, content, timestamp)>`
   - Ordered by timestamp DESC (most recent first)
   - Empty vector if no messages found

### Potential Issues to Check

**Issue 1: Column Names**
- **Query uses**: `RETURN m.role, m.content, m.timestamp`
- **Code expects**: `row.get_value("m.role")`, `row.get_value("m.content")`, `row.get_value("m.timestamp")`
- **Check**: Does GraphLite preserve full property paths as column names?
- **Fix if needed**: Add aliases to RETURN clause:
  ```rust
  RETURN m.role as role, m.content as content, m.timestamp as timestamp
  ```
  Then update `row.get_value("role")` etc.

**Issue 2: Value Types**
- **Expected**: All fields are `Value::String`
- **Check**: Confirm timestamp is returned as String (RFC3339 format)
- **Alternative**: May be `Value::DateTime` - adjust pattern matching if needed

**Issue 3: Empty Results**
- **Test**: Query with invalid conversation_id
- **Expected**: Returns empty `Vec`, no error

---

## Test 2: find_related_entities

### Objective
Verify that related entities (People and Tasks) are correctly found for a given topic.

### Steps

1. **Create test data**:
   ```
   User: "I'm working on Rust programming with Alice and Bob. Need to finish the documentation task."
   ```

   Expected entities:
   - Topic: "Rust"
   - People: "Alice", "Bob"
   - Task: "finish the documentation"

2. **Verify entity storage**:
   - Check that all entities are created
   - Verify MENTIONED_IN relationships exist

3. **Test retrieval** (requires integration):
   - Call `find_related_entities(session, "Rust")`
   - Currently not called by the application

4. **Expected behavior**:
   - Returns `Vec<String>` containing: `["Alice", "Bob", "finish the documentation"]`
   - People returned by `name` property
   - Tasks returned by `description` property
   - Empty vector if topic not found

### Potential Issues to Check

**Issue 1: CASE Statement Support**
- **Query uses**: GQL CASE statement
  ```gql
  CASE
    WHEN e:Person THEN e.name
    WHEN e:Topic THEN e.name
    WHEN e:Task THEN e.description
  END as entity_name
  ```
- **Check**: Verify GraphLite supports this syntax
- **Alternative**: If not supported, split into two queries (Person and Task separately)

**Issue 2: Column Alias**
- **Code expects**: `row.get_value("entity_name")`
- **Check**: Confirm alias is preserved in results
- **Fix if needed**: Adjust column name in `row.get_value()`

**Issue 3: NULL Handling**
- **Test**: Topic with no related entities
- **Expected**: Returns empty `Vec`, no error

---

## Integration Points

### Where These Functions Should Be Called

**Current Status**: Both functions are marked as "never used" by the compiler.

**Suggested Integration**:

1. **`get_conversation_messages`**:
   - Call in `AgenticMemory::build_context()` (src/agent/memory.rs)
   - Purpose: Include recent conversation history in context
   - Example:
     ```rust
     let recent_messages = self.graph_db.get_conversation_messages(
         &session,
         self.current_conversation.as_ref().unwrap(),
         5  // last 5 messages
     )?;
     ```

2. **`find_related_entities`**:
   - Call in `AgenticMemory::build_context()` or `ContextRetriever`
   - Purpose: Find people/tasks related to mentioned topics
   - Example:
     ```rust
     for topic in &entities.topics {
         let related = self.graph_db.find_related_entities(&session, topic)?;
         // Add to context
     }
     ```

---

## Quick Integration Test Script

Add this temporary code to `src/main.rs` for quick testing:

```rust
// After creating a conversation and sending messages
#[allow(dead_code)]
async fn test_query_parsing(memory: &AgenticMemory) -> Result<()> {
    use graphlite_sdk::Session;

    let session = memory.graph().session("admin", "")?;

    if let Some(conv_id) = memory.current_conversation() {
        println!("\n=== Testing get_conversation_messages ===");
        let messages = memory.graph().get_conversation_messages(&session, conv_id, 10)?;
        println!("Retrieved {} messages:", messages.len());
        for (role, content, timestamp) in messages {
            println!("  [{} @ {}] {}", role, timestamp, content);
        }
    }

    println!("\n=== Testing find_related_entities ===");
    let test_topic = "Rust"; // Use a topic you mentioned
    let entities = memory.graph().find_related_entities(&session, test_topic)?;
    println!("Entities related to '{}': {:?}", test_topic, entities);

    Ok(())
}
```

---

## Debug Tips

### Enable Query Logging

Add debug output to see actual query results:

```rust
// In get_conversation_messages or find_related_entities
let result = session.query(&query)?;
println!("DEBUG: Query returned {} rows", result.rows.len());
println!("DEBUG: Variables: {:?}", result.variables);
if let Some(first_row) = result.rows.first() {
    println!("DEBUG: First row values: {:?}", first_row.values);
}
```

### Check GraphLite SDK Version

If column naming doesn't match:
```bash
grep "graphlite-sdk" Cargo.toml
```

Consult SDK documentation or examples for correct usage.

---

## Future Enhancements

After confirming query parsing works:

1. **Implement ContextRetriever** (src/agent/retrieval.rs):
   - Currently stubbed out
   - Should use `get_conversation_messages` and `find_related_entities`

2. **Add Result Caching**:
   - Cache query results to avoid repeated database queries
   - Invalidate cache on new messages

3. **Improve Query Efficiency**:
   - Add indexes for frequently queried properties
   - Optimize CASE statement if needed

4. **TypedResult Integration**:
   - Consider using GraphLite SDK's `TypedResult` for automatic deserialization
   - Requires defining structs for row types

5. **Better Error Messages**:
   - Add context about which query failed
   - Include query string in error messages for debugging

---

## Completion Checklist

- [ ] Manual test: `get_conversation_messages` returns correct data
- [ ] Verify: Column names match expectations
- [ ] Manual test: `find_related_entities` returns People and Tasks
- [ ] Verify: CASE statement works correctly
- [ ] Integration: Call these functions from `AgenticMemory::build_context()`
- [ ] Remove "never used" warnings
- [ ] Add integration tests (optional)
- [ ] Document any column name adjustments needed
- [ ] Update CLAUDE.md with findings

---

## Notes

- **File**: `src/graph/operations.rs`
- **Lines**: 228-262 (`get_conversation_messages`), 278-299 (`find_related_entities`)
- **Implementation Date**: 2026-01-10
- **Status**: Code complete, awaiting manual testing
