#!/usr/bin/env python3
"""
Python asyncio baseline for comparison with Nexus
"""
import asyncio
import time

# Configuration
NUM_AGENTS = 1000
NUM_MESSAGES = 20000

class Agent:
    """Simple agent with bounded mailbox"""
    def __init__(self, agent_id):
        self.id = agent_id
        self.queue = asyncio.Queue(maxsize=1024)
        self.message_count = 0

    async def run(self):
        """Message processing loop"""
        while True:
            try:
                msg = await self.queue.get()
                self.message_count += 1
                # Simulate minimal processing
                await asyncio.sleep(0)
            except asyncio.CancelledError:
                break

async def main():
    print(f"Python asyncio baseline: {NUM_AGENTS} agents, {NUM_MESSAGES} messages")

    # Create agents
    agents = [Agent(i) for i in range(NUM_AGENTS)]
    tasks = [asyncio.create_task(agent.run()) for agent in agents]

    # Warm up
    for i in range(100):
        await agents[i % NUM_AGENTS].queue.put({"msg": i})

    # Benchmark message sending
    start_time = time.perf_counter()

    for i in range(NUM_MESSAGES):
        await agents[i % NUM_AGENTS].queue.put({"msg": i})

    end_time = time.perf_counter()
    elapsed = end_time - start_time

    # Results
    throughput = NUM_MESSAGES / elapsed
    print(f"\nResults:")
    print(f"  Time: {elapsed:.3f} seconds")
    print(f"  Throughput: {throughput:.0f} messages/second")
    print(f"  Latency: {(elapsed / NUM_MESSAGES) * 1000:.3f} ms per message")

    # Cleanup
    for task in tasks:
        task.cancel()

    await asyncio.gather(*tasks, return_exceptions=True)

if __name__ == '__main__':
    asyncio.run(main())
