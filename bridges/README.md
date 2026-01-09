# Nexus Language Bridges

This directory contains language bridges that allow agents written in different programming languages to integrate seamlessly with the Nexus agent kernel.

## Available Bridges

### Python Bridge (`python/nexus_bridge.py`)

Allows Python code to run as Nexus agents using a simple protocol over stdin/stdout.

**Features:**
- Async/await support with asyncio
- Full message routing
- Type-safe message handling
- Easy-to-use `NexusAgent` base class

**Example:**
```python
from nexus_bridge import NexusAgent, run_agent

class MyAgent(NexusAgent):
    async def handle_message(self, message):
        return {"result": "processed"}

agent = MyAgent("PythonAgent-1")
run_agent(agent)
```

**Requirements:**
- Python 3.7+
- No external dependencies (uses only stdlib)

### JavaScript/Node.js Bridge (`javascript/nexus-bridge.js`)

Allows JavaScript/Node.js code to run as Nexus agents.

**Features:**
- Promise-based async handling
- ES6+ support
- Full message routing
- Easy-to-use `NexusAgent` base class

**Example:**
```javascript
const { NexusAgent, runAgent } = require('./nexus-bridge');

class MyAgent extends NexusAgent {
  async handleMessage(message) {
    return { result: 'processed' };
  }
}

const agent = new MyAgent('JSAgent-1');
runAgent(agent);
```

**Requirements:**
- Node.js 14+
- No external dependencies

## Protocol

All bridges use a simple JSON-based protocol over stdin/stdout:

### Registration
When an agent starts, it sends:
```json
{
  "type": "register",
  "payload": {
    "id": "uuid-here",
    "name": "agent-name"
  }
}
```

### Incoming Messages
The kernel sends messages in this format:
```json
{
  "type": "message",
  "payload": {
    "from": "sender-uuid",
    "to": "receiver-uuid",
    "kind": "Command",
    "payload": { "custom": "data" },
    "timestamp": "2026-01-09T12:00:00Z"
  }
}
```

### Responses
Agents send responses like:
```json
{
  "type": "response",
  "payload": {
    "from": "agent-uuid",
    "to": "original-sender-uuid",
    "kind": "Response",
    "payload": { "result": "data" },
    "timestamp": "2026-01-09T12:00:01Z"
  }
}
```

### Shutdown
The kernel sends shutdown signal:
```json
{
  "type": "shutdown",
  "payload": {}
}
```

## Creating Your Own Bridge

To create a bridge for another language:

1. Implement the protocol (stdin/stdout JSON exchange)
2. Provide an agent base class or interface
3. Handle async message processing
4. Implement graceful shutdown
5. Add examples

## Testing

Test your bridge with the echo agent:

**Python:**
```bash
python bridges/python/nexus_bridge.py
```

**JavaScript:**
```bash
node bridges/javascript/nexus-bridge.js
```

Then send test messages via stdin.

## Examples

See the `examples/` directory for complete working examples:
- `python_agent_example.py` - Calculator agent
- `javascript_agent_example.js` - Data processor agent

## Performance

Bridge communication uses JSON over pipes, which provides:
- Low latency (< 1ms for small messages)
- High throughput (thousands of messages/second)
- Language independence
- Easy debugging (human-readable protocol)

## Future Bridges

Planned language bridges:
- Rust (native integration)
- Go
- Ruby
- Java/Kotlin

## Contributing

To contribute a new language bridge:

1. Follow the protocol specification
2. Add comprehensive examples
3. Include tests
4. Update this README
5. Submit a pull request

## License

All bridges are licensed under MIT, same as the Nexus project.
