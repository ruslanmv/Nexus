#!/usr/bin/env python3
"""
Example Python agent for Nexus

This demonstrates how to create a Python agent that integrates
with the Nexus agent kernel.
"""

import sys
import os

# Add bridge to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../bridges/python'))

from nexus_bridge import NexusAgent, Message, MessageKind, run_agent


class CalculatorAgent(NexusAgent):
    """
    A simple calculator agent that performs arithmetic operations.

    This agent responds to commands with arithmetic operations in the payload.
    """

    def __init__(self):
        super().__init__("PythonCalculator")
        self.operation_count = 0

    async def handle_message(self, message: Message):
        """Handle incoming calculation requests"""
        self.operation_count += 1

        payload = message.payload

        # Check if this is a calculation command
        if message.kind == MessageKind.COMMAND.value:
            operation = payload.get('operation')
            a = payload.get('a')
            b = payload.get('b')

            if operation and a is not None and b is not None:
                # Perform calculation
                result = self._calculate(operation, a, b)

                if result is not None:
                    return {
                        'result': result,
                        'operation': operation,
                        'operands': [a, b],
                        'agent': self.name,
                        'operations_performed': self.operation_count
                    }
                else:
                    return {
                        'error': f'Unknown operation: {operation}',
                        'agent': self.name
                    }

        # Echo back if not a calculation
        return {
            'echo': payload,
            'agent': self.name,
            'message': 'Send a calculation command with operation, a, and b'
        }

    def _calculate(self, operation: str, a: float, b: float):
        """Perform the arithmetic operation"""
        operations = {
            'add': lambda x, y: x + y,
            'subtract': lambda x, y: x - y,
            'multiply': lambda x, y: x * y,
            'divide': lambda x, y: x / y if y != 0 else None,
            'power': lambda x, y: x ** y,
        }

        op_func = operations.get(operation)
        if op_func:
            try:
                return op_func(float(a), float(b))
            except (ValueError, ZeroDivisionError) as e:
                print(f"Calculation error: {e}", file=sys.stderr)
                return None
        return None

    async def on_start(self):
        """Called when agent starts"""
        print(f"Python Calculator Agent '{self.name}' started (ID: {self.id})", file=sys.stderr)

    async def on_stop(self):
        """Called when agent stops"""
        print(f"Python Calculator Agent '{self.name}' stopped. Performed {self.operation_count} operations", file=sys.stderr)


if __name__ == '__main__':
    agent = CalculatorAgent()
    run_agent(agent)
