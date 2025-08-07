export interface WebSocketMessage {
  // Client -> Server
  JoinDocument?: { document_id: string; user_id: string };
  UpdateDocument?: { content: string; user_id: string };
  
  // Server -> Client
  DocumentState?: { 
    state: { 
      content: string; 
      version: number; 
      last_modified: number; 
    } 
  };
  UserJoined?: { user_id: string };
  UserLeft?: { user_id: string };
  DocumentUpdated?: { 
    update: { 
      content: string; 
      user_id: string; 
      timestamp: number; 
    } 
  };
  Error?: { message: string };
}

export class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();
  private documentId: string | null = null;
  private userId: string;

  constructor() {
    this.userId = this.generateUserId();
  }

  private generateUserId(): string {
    return `user_${Math.random().toString(36).substr(2, 9)}`;
  }

  connect(documentId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const wsUrl = `ws://localhost:3000/ws/doc/${documentId}`;
        console.log('Creating WebSocket connection to:', wsUrl);
        this.ws = new WebSocket(wsUrl);
        this.documentId = documentId;

        // Add connection timeout
        const connectionTimeout = setTimeout(() => {
          if (this.ws && this.ws.readyState === WebSocket.CONNECTING) {
            console.error('WebSocket connection timeout');
            this.ws.close();
            reject(new Error('Connection timeout'));
          }
        }, 5000); // 5 second timeout

        this.ws.onopen = () => {
          console.log('✅ WebSocket connected for document:', documentId);
          clearTimeout(connectionTimeout); // Clear the timeout
          this.reconnectAttempts = 0;
          
          // Join the document
          console.log('Sending JoinDocument message');
          this.send({
            JoinDocument: {
              document_id: documentId,
              user_id: this.userId
            }
          });
          
          resolve();
        };

        this.ws.onmessage = (event) => {
          console.log('📨 Received WebSocket message:', event.data);
          try {
            const message: WebSocketMessage = JSON.parse(event.data);
            this.handleMessage(message);
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        };

        this.ws.onclose = (event) => {
          console.log('🔌 WebSocket disconnected:', event.code, event.reason);
          this.attemptReconnect();
        };

        this.ws.onerror = (error) => {
          console.error('❌ WebSocket error:', error);
          clearTimeout(connectionTimeout); // Clear the timeout
          reject(error);
        };
      } catch (error) {
        console.error('❌ Failed to create WebSocket:', error);
        reject(error);
      }
    });
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts && this.documentId) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      
      setTimeout(() => {
        this.connect(this.documentId!).catch(console.error);
      }, this.reconnectDelay * this.reconnectAttempts);
    }
  }

  private handleMessage(message: WebSocketMessage) {
    if (message.DocumentState) {
      this.emit('documentState', message.DocumentState.state);
    } else if (message.DocumentUpdated) {
      this.emit('documentUpdated', message.DocumentUpdated.update);
    } else if (message.UserJoined) {
      this.emit('userJoined', message.UserJoined.user_id);
    } else if (message.UserLeft) {
      this.emit('userLeft', message.UserLeft.user_id);
    } else if (message.Error) {
      this.emit('error', message.Error.message);
    }
  }

  send(message: WebSocketMessage) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected');
    }
  }

  updateDocument(content: string) {
    if (!this.documentId) {
      console.warn('No document connected');
      return;
    }

    this.send({
      UpdateDocument: {
        content,
        user_id: this.userId
      }
    });
  }

  on(event: string, callback: (data: any) => void) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  off(event: string, callback: (data: any) => void) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.delete(callback);
    }
  }

  private emit(event: string, data: any) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.forEach(callback => callback(data));
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.documentId = null;
    this.listeners.clear();
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

// Create a singleton instance
export const websocketService = new WebSocketService(); 