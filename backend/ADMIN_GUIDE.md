# Admin Guide

This guide explains how to manage user roles and permissions in the collaborative documents system.

## Initial Setup

### 1. Create an Admin User

First, you need to create an admin user who can manage other users' roles:

```bash
# Set your database URL
export DATABASE_URL="postgresql://collaborative_user:collaborative_password@localhost:5432/collaborative_docs"

# Create an admin user
cargo run --bin create_admin admin@example.com adminpassword123
```

This will:
- Create a new user with the specified email and password
- Automatically assign the "admin" role to this user
- Display the user ID for future reference

### 2. Login as Admin

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "adminpassword123"
  }'
```

Save the JWT token from the response.

## Managing User Roles

### Available Roles

1. **admin** - Full access to all features, can manage other users
2. **user** - Basic user access (cannot create documents)
3. **document_creator** - Can create new documents

### Grant Document Creation Permission

To allow a user to create documents, update their role to "document_creator":

```bash
# Replace with actual values
ADMIN_TOKEN="your_admin_jwt_token"
USER_ID="user_uuid_here"

curl -X PUT http://localhost:3000/api/admin/users/$USER_ID/role \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "document_creator"
  }'
```

### Promote User to Admin

```bash
curl -X PUT http://localhost:3000/api/admin/users/$USER_ID/role \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "admin"
  }'
```

### Demote User to Basic User

```bash
curl -X PUT http://localhost:3000/api/admin/users/$USER_ID/role \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "user"
  }'
```

## Finding User IDs

To find a user's ID, you can query the database directly:

```sql
SELECT id, email, role_name FROM users JOIN roles ON users.role_id = roles.id;
```

## Security Notes

- Only users with the "admin" role can update user roles
- The admin endpoint requires a valid JWT token
- Role names are case-sensitive
- Invalid role names will return a validation error

## Example Workflow

1. **Create admin user:**
   ```bash
   cargo run --bin create_admin admin@example.com securepassword123
   ```

2. **Login as admin:**
   ```bash
   curl -X POST http://localhost:3000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "admin@example.com", "password": "securepassword123"}'
   ```

3. **Create a regular user (via signup endpoint):**
   ```bash
   curl -X POST http://localhost:3000/api/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"email": "user@example.com", "password": "userpassword123"}'
   ```

4. **Grant document creation permission:**
   ```bash
   # Use the admin token and user ID from previous steps
   curl -X PUT http://localhost:3000/api/admin/users/USER_ID_HERE/role \
     -H "Authorization: Bearer ADMIN_TOKEN_HERE" \
     -H "Content-Type: application/json" \
     -d '{"role_name": "document_creator"}'
   ```

5. **User can now create documents:**
   ```bash
   # Login as the user
   curl -X POST http://localhost:3000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "user@example.com", "password": "userpassword123"}'
   
   # Create a document with the user's token
   curl -X POST http://localhost:3000/api/doc \
     -H "Authorization: Bearer USER_TOKEN_HERE"
   ``` 