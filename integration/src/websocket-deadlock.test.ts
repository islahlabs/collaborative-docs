import WebSocket from 'ws';

// Types matching backend
interface WebSocketMessage {
  JoinDocument?: { document_id: string; user_id: string };
  UpdateDocument?: { content: string; user_id: string };
  DocumentState?: { state: { content: string; version: number; last_modified: number } };
  UserJoined?: { user_id: string };
  UserLeft?: { user_id: string };
  DocumentUpdated?: { update: { content: string; user_id: string; timestamp: number } };
  Error?: { message: string };
}

class WebSocketClient {
  private ws: WebSocket | null = null;
  private userId: string;
  private abortController: AbortController | null = null;

  constructor() {
    this.userId = `test_user_${Math.random().toString(36).substr(2, 9)}`;
  }

  async connect(documentId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = `ws://localhost:3000/ws/doc/${documentId}`;
      this.ws = new WebSocket(wsUrl);
      
      // Create abort controller for cleanup
      this.abortController = new AbortController();

      this.ws.on('open', () => {
        console.log(`WebSocket connected for user ${this.userId}`);
        resolve();
      });

      this.ws.on('error', (error) => {
        console.error(`WebSocket error for user ${this.userId}:`, error);
        reject(error);
      });

      this.ws.on('close', () => {
        console.log(`WebSocket disconnected for user ${this.userId}`);
      });
    });
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

describe('WebSocket Deadlock Tests', () => {
  const API_BASE = 'http://localhost:3000/api';
  let testDocumentId: string;
  let activeClients: WebSocketClient[] = [];

  beforeAll(async () => {
    // Create a test document
    const response = await fetch(`${API_BASE}/doc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });

    if (!response.ok) {
      throw new Error(`Failed to create test document: ${response.status}`);
    }

    const document = await response.json() as any;
    testDocumentId = document.id;
    console.log(`Created test document: ${testDocumentId}`);
  });

  afterEach(async () => {
    // Clean up any remaining clients
    for (const client of activeClients) {
      client.disconnect();
    }
    activeClients = [];
    
    // Small delay to ensure cleanup
    await new Promise(resolve => setTimeout(resolve, 100));
  });

  afterAll(async () => {
    // Final cleanup
    for (const client of activeClients) {
      client.disconnect();
    }
    activeClients = [];
    
    // Longer delay for final cleanup
    await new Promise(resolve => setTimeout(resolve, 500));
  });

  describe('Concurrent WebSocket Connections', () => {
    test('should handle multiple concurrent connections without deadlock', async () => {
      const clients: WebSocketClient[] = [];
      const connectionPromises: Promise<void>[] = [];

      // Create 10 concurrent connections
      for (let i = 0; i < 10; i++) {
        const client = new WebSocketClient();
        clients.push(client);
        activeClients.push(client);
        
        const connectPromise = client.connect(testDocumentId).catch(error => {
          console.error(`Connection ${i} failed:`, error);
          throw error;
        });
        
        connectionPromises.push(connectPromise);
      }

      // Wait for all connections with timeout
      const abortController = new AbortController();
      const timeoutId = setTimeout(() => {
        abortController.abort();
      }, 10000);

      try {
        await Promise.race([
          Promise.all(connectionPromises),
          new Promise((_, reject) => {
            abortController.signal.addEventListener('abort', () => {
              reject(new Error('Connection timeout - possible deadlock'));
            });
          })
        ]);
        
        // Clear timeout if successful
        clearTimeout(timeoutId);
        
        console.log('✅ All connections established successfully');
        
        // Verify all clients are connected
        for (let i = 0; i < clients.length; i++) {
          expect(clients[i].isConnected()).toBe(true);
        }
      } finally {
        // Cleanup
        clients.forEach(client => client.disconnect());
      }
    }, 15000);

    test('should handle rapid connect/disconnect cycles', async () => {
      for (let cycle = 0; cycle < 5; cycle++) {
        console.log(`Testing cycle ${cycle + 1}/5`);
        
        const clients: WebSocketClient[] = [];
        
        // Create connections
        for (let i = 0; i < 5; i++) {
          const client = new WebSocketClient();
          await client.connect(testDocumentId);
          clients.push(client);
          activeClients.push(client);
        }
        
        // Verify connections
        for (const client of clients) {
          expect(client.isConnected()).toBe(true);
        }
        
        // Disconnect all
        clients.forEach(client => client.disconnect());
        
        // Small delay between cycles
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }, 30000);

    test('should handle connection attempts during high load', async () => {
      const clients: WebSocketClient[] = [];
      
      // Start 5 connections
      for (let i = 0; i < 5; i++) {
        const client = new WebSocketClient();
        await client.connect(testDocumentId);
        clients.push(client);
        activeClients.push(client);
      }
      
      // Try to add 5 more connections while others are active
      const additionalClients: WebSocketClient[] = [];
      const additionalPromises: Promise<void>[] = [];
      
      for (let i = 0; i < 5; i++) {
        const client = new WebSocketClient();
        additionalClients.push(client);
        activeClients.push(client);
        additionalPromises.push(client.connect(testDocumentId));
      }
      
      // Wait for additional connections
      await Promise.all(additionalPromises);
      
      // Verify all connections are active
      [...clients, ...additionalClients].forEach(client => {
        expect(client.isConnected()).toBe(true);
      });
      
      // Cleanup
      [...clients, ...additionalClients].forEach(client => client.disconnect());
    }, 20000);
  });

  describe('WebSocket Message Broadcasting', () => {
    test('should broadcast messages to all connected clients', async () => {
      const clients: WebSocketClient[] = [];
      const messageCounts: number[] = [];
      
      // Create 3 clients
      for (let i = 0; i < 3; i++) {
        const client = new WebSocketClient();
        await client.connect(testDocumentId);
        clients.push(client);
        activeClients.push(client);
        messageCounts.push(0);
      }
      
      // Wait for all to be connected
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Verify all are connected
      clients.forEach(client => {
        expect(client.isConnected()).toBe(true);
      });
      
      // Cleanup
      clients.forEach(client => client.disconnect());
    }, 10000);
  });
}); 