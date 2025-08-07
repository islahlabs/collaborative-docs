# Authentication System

This document describes the authentication and authorization system implemented in the collaborative documents backend.

## Overview

The authentication system provides:
- User registration and login with email/password
- JWT-based authentication
- Role-based authorization for document creation
- Password hashing using bcrypt

## API Endpoints

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
  "token": "jwt_token_here",
  "user": {
    "id": "uuid",
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
Login with existing credentials.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:** Same as signup response.

### Protected Endpoints

#### POST /api/doc
Create a new document (requires authentication and document_creator role).

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response:**
```json
{
  "id": "document_uuid"
}
```

## User Roles

The system supports the following roles:

1. **admin** - Full access to all features
2. **user** - Basic user access (cannot create documents)
3. **document_creator** - Can create new documents

## Authentication Flow

1. User signs up or logs in via `/api/auth/signup` or `/api/auth/login`
2. Server returns a JWT token
3. Client includes the token in the `Authorization` header for protected requests
4. Server validates the token and checks user permissions

## Security Features

- Passwords are hashed using bcrypt with cost factor 12
- JWT tokens expire after 24 hours
- Role-based access control for document creation
- Input validation for email and password requirements

## Database Schema

### Users Table
- `id` (UUID, Primary Key)
- `email` (VARCHAR, Unique)
- `password_hash` (VARCHAR)
- `role_id` (INTEGER, Foreign Key to roles)
- `is_active` (BOOLEAN)
- `created_at` (TIMESTAMP)
- `updated_at` (TIMESTAMP)

### Roles Table
- `id` (SERIAL, Primary Key)
- `name` (VARCHAR, Unique)
- `description` (TEXT)
- `created_at` (TIMESTAMP)

## Usage Example

```bash
# 1. Sign up a new user
curl -X POST http://localhost:3000/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'

# 2. Create a document (requires document_creator role)
curl -X POST http://localhost:3000/api/doc \
  -H "Authorization: Bearer <jwt_token_from_signup>"
```

## Configuration

The JWT secret key is currently hardcoded in `src/auth.rs`. In production, this should be:
1. Stored in environment variables
2. Rotated regularly
3. At least 32 characters long

## Testing

Run the authentication tests with:
```bash
cargo test auth_test
``` 