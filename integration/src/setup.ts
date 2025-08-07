// Jest setup file for integration tests
import { config } from 'dotenv';

// Load environment variables if .env file exists
config();

// Global test timeout
jest.setTimeout(15000);

// Global test utilities
global.console = {
  ...console,
  // Suppress console.log during tests unless explicitly needed
  log: jest.fn(),
  debug: jest.fn(),
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
}; 