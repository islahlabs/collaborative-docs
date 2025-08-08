const API_BASE_URL = `${import.meta.env.VITE_API_URL || 'http://localhost:3000'}/api`;

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

// Authentication interfaces
export interface User {
  id: string;
  email: string;
  role_id: number;
  role_name: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface SignupRequest {
  email: string;
  password: string;
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
  // Authentication methods
  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await fetch(`${API_BASE_URL}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(response.status, errorData.error || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<AuthResponse>;
  },

  async signup(credentials: SignupRequest): Promise<AuthResponse> {
    const response = await fetch(`${API_BASE_URL}/auth/signup`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(response.status, errorData.error || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json() as Promise<AuthResponse>;
  },

  // Create a new document (requires authentication)
  async createDocument(token?: string) {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE_URL}/doc`, {
      method: 'POST',
      headers,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      let errorMessage = errorData.error || `HTTP ${response.status}: ${response.statusText}`;
      
      // Provide more user-friendly error messages
      if (response.status === 401) {
        errorMessage = 'Authentication required. Please sign in to create documents.';
      } else if (response.status === 403) {
        errorMessage = 'You do not have permission to create documents. Contact an administrator.';
      }
      
      throw new ApiError(response.status, errorMessage);
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