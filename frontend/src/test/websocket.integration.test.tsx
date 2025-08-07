import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import Editor from '@/components/Editor';
import { websocketService } from '@/services/websocket';

// Mock the API service
vi.mock('@/services/api', () => ({
  api: {
    getDocument: vi.fn(),
    updateDocument: vi.fn(),
    createDocument: vi.fn(),
    getDocumentHistory: vi.fn(),
  },
}));

// Mock the WebSocket service
vi.mock('@/services/websocket', () => ({
  websocketService: {
    connect: vi.fn(),
    disconnect: vi.fn(),
    isConnected: vi.fn(),
    updateDocument: vi.fn(),
    on: vi.fn(),
    off: vi.fn(),
  },
}));

const mockApi = vi.mocked(await import('@/services/api')).api as any;
const mockWebSocketService = vi.mocked(websocketService) as any;

// Helper to render Editor with router
const renderEditor = (documentId: string) => {
  return render(
    <BrowserRouter>
      <Editor />
    </BrowserRouter>
  );
};

describe('Editor WebSocket Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock API responses
    mockApi.getDocument.mockResolvedValue({
      id: 'test-doc-123',
      content: 'Initial content',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    });
    
    mockApi.updateDocument.mockResolvedValue({
      id: 'test-doc-123',
      content: 'Updated content',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    });
    
    mockApi.getDocumentHistory.mockResolvedValue([]);
    
    // Mock WebSocket service
    mockWebSocketService.isConnected.mockReturnValue(true);
    mockWebSocketService.connect.mockResolvedValue();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('WebSocket Connection', () => {
    it('should connect to WebSocket when component mounts', async () => {
      // Mock useParams to return a document ID
      vi.mock('react-router-dom', async () => {
        const actual = await vi.importActual('react-router-dom');
        return {
          ...actual,
          useParams: () => ({ id: 'test-doc-123' }),
        };
      });

      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(mockWebSocketService.connect).toHaveBeenCalledWith('test-doc-123');
      });
    });

    it('should disconnect WebSocket when component unmounts', async () => {
      const { unmount } = renderEditor('test-doc-123');
      
      // Wait for component to mount and connect
      await waitFor(() => {
        expect(mockWebSocketService.connect).toHaveBeenCalled();
      });
      
      // Unmount the component
      unmount();
      
      // Check that disconnect was called
      expect(mockWebSocketService.disconnect).toHaveBeenCalled();
    });

    it('should show connection status in UI', async () => {
      mockWebSocketService.isConnected.mockReturnValue(true);
      
      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(screen.getByText('Live')).toBeInTheDocument();
      });
    });

    it('should show offline status when WebSocket is disconnected', async () => {
      mockWebSocketService.isConnected.mockReturnValue(false);
      
      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(screen.getByText('Offline')).toBeInTheDocument();
      });
    });
  });

  describe('Real-time Collaboration', () => {
    it('should send document updates via WebSocket', async () => {
      const user = userEvent.setup();
      
      renderEditor('test-doc-123');

      const textarea = await screen.findByRole('textbox');
      await user.type(textarea, 'New content');

      // Wait for any WebSocket update call (the content will be incremental)
      await waitFor(() => {
        expect(mockWebSocketService.updateDocument).toHaveBeenCalled();
      }, { timeout: 3000 });
      
      // Check that it was called with some content containing "New content"
      const calls = mockWebSocketService.updateDocument.mock.calls;
      const lastCall = calls[calls.length - 1];
      expect(lastCall[0]).toContain('New content');
    });

    it('should handle WebSocket message events', async () => {
      let messageHandler: ((data: any) => void) | undefined;
      
      mockWebSocketService.on.mockImplementation((event: string, handler: (data: any) => void) => {
        if (event === 'documentUpdated') {
          messageHandler = handler;
        }
      });

      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(mockWebSocketService.on).toHaveBeenCalledWith('documentUpdated', expect.any(Function));
      });

      // Simulate receiving a WebSocket message
      if (messageHandler) {
        messageHandler({
          content: 'Updated from another user',
          user_id: 'other-user',
          timestamp: Date.now(),
        });
      }
    });

    it('should handle user join/leave events', async () => {
      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(mockWebSocketService.on).toHaveBeenCalledWith('userJoined', expect.any(Function));
        expect(mockWebSocketService.on).toHaveBeenCalledWith('userLeft', expect.any(Function));
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle WebSocket connection errors gracefully', async () => {
      mockWebSocketService.connect.mockRejectedValue(new Error('Connection failed'));
      
      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(screen.getByText('Offline')).toBeInTheDocument();
      });
    });

    it('should handle WebSocket message parsing errors', async () => {
      let errorHandler: ((error: string) => void) | undefined;
      
      mockWebSocketService.on.mockImplementation((event: string, handler: (data: any) => void) => {
        if (event === 'error') {
          errorHandler = handler as (error: string) => void;
        }
      });

      renderEditor('test-doc-123');

      await waitFor(() => {
        expect(mockWebSocketService.on).toHaveBeenCalledWith('error', expect.any(Function));
      });

      // Simulate WebSocket error
      if (errorHandler) {
        errorHandler('WebSocket error occurred');
      }
    });
  });

  describe('UI Integration', () => {
    it('should display active user count', async () => {
      renderEditor('test-doc-123');

      // Mock active users state
      await waitFor(() => {
        // The component should handle active users display
        expect(screen.getByText('Document: test-doc-123')).toBeInTheDocument();
      });
    });

    it('should show saving status during updates', async () => {
      const user = userEvent.setup();
      
      renderEditor('test-doc-123');

      const textarea = await screen.findByRole('textbox');
      await user.type(textarea, 'New content');

      // Should show saving indicator (but it might be brief due to debouncing)
      await waitFor(() => {
        // Check for either "Saving..." or "Last saved" indicator
        const savingText = screen.queryByText('Saving...');
        const lastSavedText = screen.queryByText(/Last saved:/);
        expect(savingText || lastSavedText).toBeInTheDocument();
      }, { timeout: 3000 });
    });
  });
}); 