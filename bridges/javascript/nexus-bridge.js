#!/usr/bin/env node
/**
 * Nexus JavaScript Bridge
 *
 * This module provides JavaScript/Node.js bindings for the Nexus agent kernel,
 * allowing JavaScript code to run as first-class agents in the Nexus system.
 *
 * @example
 * const { NexusAgent, runAgent } = require('./nexus-bridge');
 *
 * class MyAgent extends NexusAgent {
 *   async handleMessage(message) {
 *     console.log('Received:', message);
 *     return { status: 'processed', result: 42 };
 *   }
 * }
 *
 * const agent = new MyAgent('JSAgent-1');
 * runAgent(agent);
 */

const { v4: uuidv4 } = require('crypto').randomUUID ? { v4: () => require('crypto').randomUUID() } : require('uuid');
const readline = require('readline');

/**
 * Message types in Nexus
 */
const MessageKind = {
  COMMAND: 'Command',
  EVENT: 'Event',
  QUERY: 'Query',
  RESPONSE: 'Response'
};

/**
 * Nexus message structure
 */
class Message {
  constructor({ from, to, kind, payload, timestamp, conversation_id, trace_id }) {
    this.from = from;
    this.to = to;
    this.kind = kind;
    this.payload = payload;
    this.timestamp = timestamp;
    this.conversation_id = conversation_id;
    this.trace_id = trace_id;
  }

  /**
   * Create a reply to this message
   */
  reply(senderId, payload) {
    return new Message({
      from: senderId,
      to: this.from,
      kind: MessageKind.RESPONSE,
      payload: payload,
      timestamp: new Date().toISOString(),
      conversation_id: this.conversation_id,
      trace_id: this.trace_id
    });
  }

  toJSON() {
    const result = {
      from: this.from,
      to: this.to,
      kind: this.kind,
      payload: this.payload,
      timestamp: this.timestamp
    };
    if (this.conversation_id) result.conversation_id = this.conversation_id;
    if (this.trace_id) result.trace_id = this.trace_id;
    return result;
  }
}

/**
 * Base class for JavaScript agents in Nexus
 *
 * Subclass this and implement handleMessage to create your agent.
 */
class NexusAgent {
  /**
   * @param {string} name - Human-readable name for the agent
   * @param {string} [id] - UUID for the agent (generated if not provided)
   */
  constructor(name, id) {
    this.name = name;
    this.id = id || uuidv4();
    this._running = false;
  }

  /**
   * Handle an incoming message
   *
   * @param {Message} message - The received message
   * @returns {Promise<Object|null>} Response payload or null if no response needed
   */
  async handleMessage(message) {
    throw new Error('handleMessage must be implemented by subclass');
  }

  /**
   * Called when the agent starts
   */
  async onStart() {}

  /**
   * Called when the agent stops
   */
  async onStop() {}
}

/**
 * Protocol handler for communication with Nexus kernel
 */
class BridgeProtocol {
  constructor(agent) {
    this.agent = agent;
    this.rl = null;
  }

  /**
   * Send a message to the kernel
   */
  sendMessage(msgType, payload) {
    const message = {
      type: msgType,
      payload: payload
    };
    process.stdout.write(JSON.stringify(message) + '\n');
  }

  /**
   * Main protocol loop
   */
  async run() {
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stderr,  // Use stderr for protocol debug output
      terminal: false
    });

    // Send registration
    this.sendMessage('register', {
      id: this.agent.id,
      name: this.agent.name
    });

    await this.agent.onStart();

    // Process incoming messages
    for await (const line of this.rl) {
      try {
        const data = JSON.parse(line);
        const msgType = data.type;
        const payload = data.payload || {};

        if (msgType === 'message') {
          // Handle incoming message
          const message = new Message(payload);

          try {
            const responsePayload = await this.agent.handleMessage(message);

            if (responsePayload !== null && responsePayload !== undefined) {
              const response = message.reply(this.agent.id, responsePayload);
              this.sendMessage('response', response.toJSON());
            }
          } catch (error) {
            process.stderr.write(`Error handling message: ${error.message}\n`);
            const errorResponse = message.reply(this.agent.id, {
              error: error.message
            });
            this.sendMessage('response', errorResponse.toJSON());
          }
        } else if (msgType === 'shutdown') {
          break;
        }
      } catch (error) {
        process.stderr.write(`Error processing line: ${error.message}\n`);
      }
    }

    await this.agent.onStop();
  }
}

/**
 * Run an agent with the bridge protocol
 *
 * @param {NexusAgent} agent - The agent instance to run
 */
async function runAgent(agent) {
  const protocol = new BridgeProtocol(agent);

  try {
    await protocol.run();
  } catch (error) {
    process.stderr.write(`Fatal error: ${error.message}\n`);
    process.exit(1);
  }
}

/**
 * Example echo agent for testing
 */
class EchoAgent extends NexusAgent {
  async handleMessage(message) {
    return {
      echo: message.payload,
      agent: this.name,
      processed_at: new Date().toISOString()
    };
  }
}

// Export classes and functions
module.exports = {
  NexusAgent,
  Message,
  MessageKind,
  runAgent,
  EchoAgent
};

// Run echo agent if executed directly
if (require.main === module) {
  const agent = new EchoAgent('JSEchoAgent');
  runAgent(agent);
}
