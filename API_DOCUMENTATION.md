# API Documentation

This document provides comprehensive documentation for the Collaborative Docs API.

## 🔗 API Base URL

- **Development**: `http://localhost:3000`
- **Production**: `https://api.example.com`

## 📚 Interactive Documentation

Access the interactive Swagger UI at:
- **Development**: `http://localhost:3000/swagger-ui`
- **Production**: `https://api.example.com/swagger-ui`

## 🔐 Authentication

The API uses JWT (JSON Web Token) authentication. Include the token in the `Authorization` header:

```
Authorization: Bearer <your_jwt_token>
```

## 📋 API Endpoints

### Authentication Endpoints

#### POST /api/auth/signup
Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "role_id": 2,
    "role_name": "user",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### POST /api/auth/login
Authenticate with existing credentials.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:** Same as signup response.

### Document Endpoints

#### POST /api/doc
Create a new document (requires authentication and `document_creator` role).

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### GET /api/doc/{id}
Get a document by ID (public endpoint).

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Hello, World!",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### PUT /api/doc/{id}
Update a document's content (public endpoint).

**Request Body:**
```json
{
  "content": "Updated content here"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Updated content here",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

#### GET /api/doc/{id}/history
Get document version history (public endpoint).

**Response:**
```json
[
  {
    "timestamp": "2024-01-01T12:00:00Z",
    "ip_address": "192.168.1.1",
    "content": "Previous version content"
  },
  {
    "timestamp": "2024-01-01T11:00:00Z",
    "ip_address": "192.168.1.1",
    "content": "Original content"
  }
]
```

#### GET /api/doc/{id}/stats
Get document statistics (public endpoint).

**Response:**
```json
{
  "history_count": 5,
  "last_updated": "2024-01-01T12:00:00Z"
}
```

#### GET /api/search?q=query
Search documents by content (public endpoint).

**Query Parameters:**
- `q` (required): Search query string

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Document containing search term",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

### CRDT Endpoints (Real-time Collaboration)

#### GET /api/doc/{id}/crdt/state
Get the current CRDT state for real-time collaboration (public endpoint).

**Response:**
```json
{
  "content": "Current document content",
  "version": 1,
  "last_updated": "2024-01-01T12:00:00Z"
}
```

#### POST /api/doc/{id}/crdt/update
Apply a CRDT update from another client (public endpoint).

**Request Body:**
```json
{
  "operation": "insert",
  "position": 5,
  "content": "new text",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Update applied successfully"
}
```

### Admin Endpoints

#### PUT /api/admin/users/{user_id}/role
Update a user's role (requires admin authentication).

**Headers:**
```
Authorization: Bearer <admin_jwt_token>
```

**Request Body:**
```json
{
  "role_name": "document_creator"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "role_id": 3,
  "role_name": "document_creator",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

### WebSocket Endpoints

#### GET /ws/doc/{document_id}
WebSocket endpoint for real-time document collaboration.

**Protocol:** `ws://` or `wss://`

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:3000/ws/doc/document-id');
```

**Messages:**
```json
// Join document
{
  "JoinDocument": {
    "document_id": "document-id",
    "user_id": "user-id"
  }
}

// Update document
{
  "UpdateDocument": {
    "content": "new content",
    "user_id": "user-id"
  }
}
```

#### GET /ws/info/{document_id}
Get WebSocket connection information (HTTP endpoint).

**Response:**
```json
{
  "message": "WebSocket endpoint",
  "document_id": "document-id",
  "endpoint": "/ws/doc/document-id",
  "protocol": "ws:// or wss://",
  "active_users_count": 3
}
```

## 🔒 User Roles

The system supports the following roles:

1. **admin** - Full access to all features, can manage other users
2. **user** - Basic user access (cannot create documents)
3. **document_creator** - Can create new documents

## 📊 Error Responses

### Validation Error (400)
```json
{
  "error": "Validation error: Invalid email format",
  "status": 400
}
```

### Authentication Error (401)
```json
{
  "error": "Authentication error: Invalid token",
  "status": 401
}
```

### Authorization Error (403)
```json
{
  "error": "Authorization error: Insufficient permissions",
  "status": 403
}
```

### Not Found Error (404)
```json
{
  "error": "Document not found: 550e8400-e29b-41d4-a716-446655440000",
  "status": 404
}
```

### Conflict Error (409)
```json
{
  "error": "User already exists: user@example.com",
  "status": 409
}
```

### Internal Server Error (500)
```json
{
  "error": "Internal server error",
  "status": 500
}
```

## 🚀 Usage Examples

### cURL Examples

#### Create a User
```bash
curl -X POST http://localhost:3000/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

#### Login
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

#### Create a Document
```bash
curl -X POST http://localhost:3000/api/doc \
  -H "Authorization: Bearer <your_jwt_token>"
```

#### Update a Document
```bash
curl -X PUT http://localhost:3000/api/doc/document-id \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Updated content"
  }'
```

#### Search Documents
```bash
curl "http://localhost:3000/api/search?q=hello"
```

### JavaScript Examples

#### WebSocket Connection
```javascript
const ws = new WebSocket('ws://localhost:3000/ws/doc/document-id');

ws.onopen = () => {
  console.log('Connected to document');
  
  // Join the document
  ws.send(JSON.stringify({
    JoinDocument: {
      document_id: 'document-id',
      user_id: 'user-id'
    }
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

#### API Client
```javascript
class CollaborativeDocsAPI {
  constructor(baseURL = 'http://localhost:3000') {
    this.baseURL = baseURL;
    this.token = null;
  }

  async signup(email, password) {
    const response = await fetch(`${this.baseURL}/api/auth/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });
    const data = await response.json();
    this.token = data.token;
    return data;
  }

  async createDocument() {
    const response = await fetch(`${this.baseURL}/api/doc`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json'
      }
    });
    return response.json();
  }

  async updateDocument(id, content) {
    const response = await fetch(`${this.baseURL}/api/doc/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content })
    });
    return response.json();
  }
}
```

## 🔧 Rate Limiting

The API implements rate limiting to prevent abuse:
- **Authentication endpoints**: 5 requests per minute per IP
- **Document endpoints**: 100 requests per minute per IP
- **Search endpoints**: 50 requests per minute per IP

## 📝 Notes

- All timestamps are in ISO 8601 format (UTC)
- Document IDs are UUIDs
- Content is limited to 100,000 characters
- Passwords must be at least 6 characters long
- Email addresses must be valid format
- JWT tokens expire after 24 hours 