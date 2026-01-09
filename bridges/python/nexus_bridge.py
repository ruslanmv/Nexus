#!/usr/bin/env python3
"""
Nexus Python Bridge

This module provides Python bindings for the Nexus agent kernel,
allowing Python code to run as first-class agents in the Nexus system.

Example:
    ```python
    from nexus_bridge import NexusAgent, run_agent

    class MyAgent(NexusAgent):
        async def handle_message(self, message):
            print(f"Received: {message}")
            return {"status": "processed", "result": 42}

    if __name__ == "__main__":
        agent = MyAgent("PythonAgent-1")
        run_agent(agent)
    ```
"""

import asyncio
import json
import sys
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass, asdict
from datetime import datetime
from typing import Optional, Dict, Any
from enum import Enum


class MessageKind(Enum):
    """Message types in Nexus"""
    COMMAND = "Command"
    EVENT = "Event"
    QUERY = "Query"
    RESPONSE = "Response"


@dataclass
class Message:
    """Nexus message structure"""
    from_id: str
    to_id: str
    kind: str
    payload: Dict[str, Any]
    timestamp: str
    conversation_id: Optional[str] = None
    trace_id: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Message':
        """Create message from dictionary"""
        return cls(
            from_id=data['from'],
            to_id=data['to'],
            kind=data['kind'],
            payload=data['payload'],
            timestamp=data['timestamp'],
            conversation_id=data.get('conversation_id'),
            trace_id=data.get('trace_id')
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert message to dictionary"""
        result = {
            'from': self.from_id,
            'to': self.to_id,
            'kind': self.kind,
            'payload': self.payload,
            'timestamp': self.timestamp
        }
        if self.conversation_id:
            result['conversation_id'] = self.conversation_id
        if self.trace_id:
            result['trace_id'] = self.trace_id
        return result

    def reply(self, sender_id: str, payload: Dict[str, Any]) -> 'Message':
        """Create a reply to this message"""
        return Message(
            from_id=sender_id,
            to_id=self.from_id,
            kind=MessageKind.RESPONSE.value,
            payload=payload,
            timestamp=datetime.utcnow().isoformat() + 'Z',
            conversation_id=self.conversation_id,
            trace_id=self.trace_id
        )


class NexusAgent(ABC):
    """
    Base class for Python agents in Nexus.

    Subclass this and implement handle_message to create your agent.
    """

    def __init__(self, name: str, agent_id: Optional[str] = None):
        """
        Initialize the agent.

        Args:
            name: Human-readable name for the agent
            agent_id: UUID for the agent (generated if not provided)
        """
        self.name = name
        self.id = agent_id or str(uuid.uuid4())
        self._running = False

    @abstractmethod
    async def handle_message(self, message: Message) -> Optional[Dict[str, Any]]:
        """
        Handle an incoming message.

        Args:
            message: The received message

        Returns:
            Optional response payload (None if no response needed)
        """
        pass

    async def on_start(self):
        """Called when the agent starts"""
        pass

    async def on_stop(self):
        """Called when the agent stops"""
        pass


class BridgeProtocol:
    """Protocol handler for communication with Nexus kernel"""

    def __init__(self, agent: NexusAgent):
        self.agent = agent
        self.reader = None
        self.writer = None

    async def connect_stdio(self):
        """Connect using stdin/stdout"""
        self.reader = asyncio.StreamReader()
        read_protocol = asyncio.StreamReaderProtocol(self.reader)

        loop = asyncio.get_event_loop()
        await loop.connect_read_pipe(lambda: read_protocol, sys.stdin)

        write_transport, write_protocol = await loop.connect_write_pipe(
            asyncio.Protocol, sys.stdout
        )
        self.writer = asyncio.StreamWriter(
            write_transport, write_protocol, self.reader, loop
        )

    async def send_message(self, msg_type: str, payload: Dict[str, Any]):
        """Send a message to the kernel"""
        message = {
            'type': msg_type,
            'payload': payload
        }
        data = json.dumps(message) + '\n'
        self.writer.write(data.encode('utf-8'))
        await self.writer.drain()

    async def receive_message(self) -> Optional[Dict[str, Any]]:
        """Receive a message from the kernel"""
        try:
            line = await self.reader.readline()
            if not line:
                return None
            return json.loads(line.decode('utf-8'))
        except Exception as e:
            sys.stderr.write(f"Error receiving message: {e}\n")
            return None

    async def run(self):
        """Main protocol loop"""
        await self.connect_stdio()

        # Send registration
        await self.send_message('register', {
            'id': self.agent.id,
            'name': self.agent.name
        })

        await self.agent.on_start()

        try:
            while True:
                data = await self.receive_message()
                if data is None:
                    break

                msg_type = data.get('type')
                payload = data.get('payload', {})

                if msg_type == 'message':
                    # Handle incoming message
                    message = Message.from_dict(payload)
                    try:
                        response_payload = await self.agent.handle_message(message)

                        if response_payload is not None:
                            response = message.reply(self.agent.id, response_payload)
                            await self.send_message('response', response.to_dict())

                    except Exception as e:
                        sys.stderr.write(f"Error handling message: {e}\n")
                        error_response = message.reply(
                            self.agent.id,
                            {'error': str(e)}
                        )
                        await self.send_message('response', error_response.to_dict())

                elif msg_type == 'shutdown':
                    break

        finally:
            await self.agent.on_stop()


def run_agent(agent: NexusAgent):
    """
    Run an agent with the bridge protocol.

    Args:
        agent: The agent instance to run
    """
    protocol = BridgeProtocol(agent)

    try:
        asyncio.run(protocol.run())
    except KeyboardInterrupt:
        pass
    except Exception as e:
        sys.stderr.write(f"Fatal error: {e}\n")
        sys.exit(1)


# Example agent for testing
class EchoAgent(NexusAgent):
    """Simple echo agent for testing"""

    async def handle_message(self, message: Message) -> Optional[Dict[str, Any]]:
        """Echo back the received message"""
        return {
            'echo': message.payload,
            'agent': self.name,
            'processed_at': datetime.utcnow().isoformat()
        }


if __name__ == "__main__":
    # Run echo agent if executed directly
    agent = EchoAgent("PythonEchoAgent")
    run_agent(agent)
