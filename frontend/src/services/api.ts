const API_BASE_URL = 'http://localhost:3000/api';

export interface Document {
  id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface DocumentHistory {
  timestamp: string;
  ip_address: string;
  content: string;
}

export interface CreateDocumentResponse {
  id: string;
}

export interface UpdateDocumentRequest {
  content: string;
}

class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

export const api = {
  // Create a new document
  async createDocument() {
    const response = await fetch(`${API_BASE_URL}/doc`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new ApiError(response.status, `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<CreateDocumentResponse>;
  },

  // Get document by ID
  async getDocument(id: string) {
    const response = await fetch(`${API_BASE_URL}/doc/${id}`, {
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new ApiError(response.status, `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<Document>;
  },

  // Update document content
  async updateDocument(id: string, content: string) {
    const response = await fetch(`${API_BASE_URL}/doc/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ content }),
    });

    if (!response.ok) {
      throw new ApiError(response.status, `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<Document>;
  },

  // Get document history
  async getDocumentHistory(id: string) {
    const response = await fetch(`${API_BASE_URL}/doc/${id}/history`, {
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new ApiError(response.status, `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<DocumentHistory[]>;
  },
}; 