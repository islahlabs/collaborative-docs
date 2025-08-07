# Integration Tests

This directory contains comprehensive integration tests for the collaborative docs WebSocket CRDT functionality.

## Setup

1. **Install dependencies:**
   ```bash
   pnpm install
   ```

2. **Ensure backend is running:**
   ```bash
   cd ../backend
   cargo run
   ```

3. **Ensure frontend is running (optional):**
   ```bash
   cd ../frontend
   pnpm run dev
   ```

## Running Tests

### All Tests
```bash
pnpm test
```

### Watch Mode
```bash
pnpm test:watch
```

### Specific Test Categories
```bash
# WebSocket tests only
pnpm test:websocket

# CRDT tests only
pnpm test:crdt
```

## Test Structure

### `websocket-crdt.test.ts`
Comprehensive integration tests covering:

- **Backend API Endpoints**: Document CRUD operations
- **WebSocket Connection**: Connection establishment and error handling
- **WebSocket Message Exchange**: Real-time message sending/receiving
- **CRDT Functionality**: Concurrent updates and state consistency
- **Error Handling**: Malformed messages and disconnection scenarios

## Test Features

- **TypeScript**: Full type safety with interfaces matching backend
- **WebSocket Testing**: Real WebSocket connections using `ws` library
- **HTTP Testing**: API endpoint testing using `node-fetch`
- **Concurrent Testing**: Multiple client simulation
- **Error Scenarios**: Comprehensive error handling tests
- **Jest Framework**: Industry-standard testing with proper setup/teardown

## Best Practices Followed

1. **Location**: Project root in dedicated `integration/` folder
2. **Language**: TypeScript for type safety and consistency
3. **Framework**: Jest with proper configuration
4. **Structure**: Separate from unit tests and frontend/backend tests
5. **Configuration**: Proper test configuration and environment setup
6. **Dependencies**: Managed with `pnpm` for consistency

## Test Environment

- **Backend**: `http://localhost:3000`
- **WebSocket**: `ws://localhost:3000/ws/doc/{document_id}`
- **Frontend**: `http://localhost:5173` (optional for full integration)

## Adding New Tests

1. Create new test files in `src/` with `.test.ts` extension
2. Follow the existing pattern with `describe` and `test` blocks
3. Use the `IntegrationTestClient` class for WebSocket testing
4. Add proper setup/teardown in `beforeAll`/`afterAll` hooks
5. Use descriptive test names and proper assertions

## Troubleshooting

- **Connection Errors**: Ensure backend is running on port 3000
- **Type Errors**: Check that interfaces match backend types
- **Timeout Errors**: Increase timeout in `jest.config.js` if needed
- **WebSocket Errors**: Verify WebSocket endpoint is accessible