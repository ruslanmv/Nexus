#!/usr/bin/env node
/**
 * Example JavaScript agent for Nexus
 *
 * This demonstrates how to create a JavaScript/Node.js agent that integrates
 * with the Nexus agent kernel.
 */

const path = require('path');
const { NexusAgent, MessageKind, runAgent } = require(
  path.join(__dirname, '../bridges/javascript/nexus-bridge.js')
);

/**
 * A simple data processor agent that transforms and filters data.
 */
class DataProcessorAgent extends NexusAgent {
  constructor() {
    super('JSDataProcessor');
    this.processedCount = 0;
  }

  async handleMessage(message) {
    this.processedCount++;

    const { payload } = message;

    // Handle different command types
    if (message.kind === MessageKind.COMMAND) {
      const { action, data } = payload;

      switch (action) {
        case 'transform':
          return this.transformData(data);

        case 'filter':
          return this.filterData(data, payload.condition);

        case 'aggregate':
          return this.aggregateData(data);

        case 'stats':
          return this.getStats();

        default:
          return {
            error: `Unknown action: ${action}`,
            agent: this.name,
            available_actions: ['transform', 'filter', 'aggregate', 'stats']
          };
      }
    }

    // Echo for non-commands
    return {
      echo: payload,
      agent: this.name,
      message: 'Send a command with action and data'
    };
  }

  transformData(data) {
    if (!Array.isArray(data)) {
      return { error: 'Data must be an array' };
    }

    // Transform: uppercase strings, double numbers
    const transformed = data.map(item => {
      if (typeof item === 'string') {
        return item.toUpperCase();
      } else if (typeof item === 'number') {
        return item * 2;
      }
      return item;
    });

    return {
      result: transformed,
      operation: 'transform',
      count: transformed.length,
      agent: this.name
    };
  }

  filterData(data, condition) {
    if (!Array.isArray(data)) {
      return { error: 'Data must be an array' };
    }

    let filtered;
    switch (condition) {
      case 'numbers':
        filtered = data.filter(item => typeof item === 'number');
        break;
      case 'strings':
        filtered = data.filter(item => typeof item === 'string');
        break;
      case 'positive':
        filtered = data.filter(item => typeof item === 'number' && item > 0);
        break;
      default:
        return { error: `Unknown condition: ${condition}` };
    }

    return {
      result: filtered,
      operation: 'filter',
      condition: condition,
      original_count: data.length,
      filtered_count: filtered.length,
      agent: this.name
    };
  }

  aggregateData(data) {
    if (!Array.isArray(data)) {
      return { error: 'Data must be an array' };
    }

    const numbers = data.filter(item => typeof item === 'number');

    if (numbers.length === 0) {
      return { error: 'No numbers to aggregate' };
    }

    const sum = numbers.reduce((a, b) => a + b, 0);
    const avg = sum / numbers.length;
    const min = Math.min(...numbers);
    const max = Math.max(...numbers);

    return {
      result: {
        sum,
        average: avg,
        min,
        max,
        count: numbers.length
      },
      operation: 'aggregate',
      agent: this.name
    };
  }

  getStats() {
    return {
      result: {
        processed_messages: this.processedCount,
        agent_id: this.id,
        agent_name: this.name,
        uptime: process.uptime()
      },
      operation: 'stats',
      agent: this.name
    };
  }

  async onStart() {
    console.error(`JavaScript Data Processor Agent '${this.name}' started (ID: ${this.id})`);
  }

  async onStop() {
    console.error(`JavaScript Data Processor Agent '${this.name}' stopped. Processed ${this.processedCount} messages`);
  }
}

// Run the agent
if (require.main === module) {
  const agent = new DataProcessorAgent();
  runAgent(agent);
}

module.exports = DataProcessorAgent;
