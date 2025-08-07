import { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Save, Clock, History, User, Wifi, WifiOff } from 'lucide-react';
import { api } from '@/services/api';
import { websocketService } from '@/services/websocket';
import type { DocumentHistory } from '@/services/api';

interface EditorProps {
  onSave?: (content: string) => void;
}

export default function Editor({ onSave }: EditorProps) {
  const [content, setContent] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [history, setHistory] = useState<DocumentHistory[]>([]);
  const [isLoadingHistory, setIsLoadingHistory] = useState(false);
  const [isWebSocketConnected, setIsWebSocketConnected] = useState(false);
  const [activeUsers, setActiveUsers] = useState<string[]>([]);
  const { id } = useParams<{ id: string }>();

  // Load document history
  const loadHistory = useCallback(async () => {
    if (!id || id === 'new-document') return;
    
    try {
      setIsLoadingHistory(true);
      const historyData = await api.getDocumentHistory(id);
      setHistory(historyData);
    } catch (error) {
      console.error('Failed to load history:', error);
    } finally {
      setIsLoadingHistory(false);
    }
  }, [id]);

  // Debounced save function
  const debouncedSave = useCallback(
    (() => {
      let timeoutId: number;
      return (text: string) => {
        clearTimeout(timeoutId);
        timeoutId = window.setTimeout(async () => {
          setIsSaving(true);
          try {
            await api.updateDocument(id!, text);
            onSave?.(text);
            setLastSaved(new Date());
            // Refresh history after saving
            await loadHistory();
          } catch (error) {
            console.error('Failed to save document:', error);
          } finally {
            setIsSaving(false);
          }
        }, 2000); // 2 second delay
      };
    })(),
    [id, onSave, loadHistory]
  );

  // Handle content changes
  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    setContent(newContent);
    
    // Send update via WebSocket for real-time collaboration
    if (isWebSocketConnected) {
      websocketService.updateDocument(newContent);
    }
    
    // Also save to database (debounced)
    debouncedSave(newContent);
  };

  // WebSocket integration
  useEffect(() => {
    if (!id || id === 'new-document') return;

    // Connect to WebSocket
    const connectWebSocket = async () => {
      try {
        await websocketService.connect(id);
        setIsWebSocketConnected(true);
        
        // Set up event listeners
        websocketService.on('documentState', (state) => {
          console.log('Received document state:', state);
          setContent(state.content);
        });

        websocketService.on('documentUpdated', (update) => {
          console.log('Document updated by another user:', update);
          setContent(update.content);
          setLastSaved(new Date());
        });

        websocketService.on('userJoined', (userId) => {
          console.log('User joined:', userId);
          setActiveUsers(prev => [...prev, userId]);
        });

        websocketService.on('userLeft', (userId) => {
          console.log('User left:', userId);
          setActiveUsers(prev => prev.filter(id => id !== userId));
        });

        websocketService.on('error', (error) => {
          console.error('WebSocket error:', error);
          setError(`WebSocket error: ${error}`);
        });
      } catch (error) {
        console.error('Failed to connect WebSocket:', error);
        setError('Failed to connect to real-time collaboration');
      }
    };

    connectWebSocket();

    // Cleanup on unmount
    return () => {
      websocketService.disconnect();
      setIsWebSocketConnected(false);
      setActiveUsers([]);
    };
  }, [id]);

  // Load document content and history on mount
  useEffect(() => {
    const loadDocument = async () => {
      try {
        setIsLoading(true);
        setError(null);
        const document = await api.getDocument(id!);
        setContent(document.content);
        // Load history after document loads
        await loadHistory();
      } catch (error) {
        console.error('Failed to load document:', error);
        setError('Failed to load document. It may not exist yet.');
        // If document doesn't exist, we'll create it when user starts typing
      } finally {
        setIsLoading(false);
      }
    };

    if (id && id !== 'new-document') {
      loadDocument();
    } else {
      setIsLoading(false);
    }
  }, [id, loadHistory]);

  // Format history entry for display
  const formatHistoryEntry = (entry: DocumentHistory) => {
    const date = new Date(entry.timestamp);
    const timeAgo = getTimeAgo(date);
    const contentPreview = entry.content.length > 100 
      ? entry.content.substring(0, 100) + '...' 
      : entry.content;
    
    return {
      timeAgo,
      contentPreview,
      ipAddress: entry.ip_address,
      fullContent: entry.content
    };
  };

  // Helper function to get relative time
  const getTimeAgo = (date: Date) => {
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
    
    if (diffInSeconds < 60) return `${diffInSeconds}s ago`;
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`;
    if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="max-w-6xl mx-auto p-6 h-screen flex flex-col">
      <Card className="mb-6">
        <CardHeader className="pb-3">
          <div className="flex justify-between items-center">
            <CardTitle className="text-2xl">Document: {id}</CardTitle>
            <div className="flex items-center gap-3">
              {/* WebSocket Status */}
              <Badge 
                variant={isWebSocketConnected ? "default" : "secondary"} 
                className="flex items-center gap-1"
              >
                {isWebSocketConnected ? (
                  <>
                    <Wifi className="h-3 w-3" />
                    Live
                  </>
                ) : (
                  <>
                    <WifiOff className="h-3 w-3" />
                    Offline
                  </>
                )}
              </Badge>
              
              {/* Active Users */}
              {activeUsers.length > 0 && (
                <Badge variant="outline" className="flex items-center gap-1">
                  <User className="h-3 w-3" />
                  {activeUsers.length} active
                </Badge>
              )}
              
              {isSaving && (
                <Badge variant="secondary" className="flex items-center gap-1">
                  <Save className="h-3 w-3" />
                  Saving...
                </Badge>
              )}
              {lastSaved && !isSaving && (
                <Badge variant="outline" className="flex items-center gap-1 text-green-600">
                  <Clock className="h-3 w-3" />
                  Last saved: {lastSaved.toLocaleTimeString()}
                </Badge>
              )}
            </div>
          </div>
        </CardHeader>
      </Card>
      
      <div className="flex-1 flex gap-6">
        {/* Editor Section */}
        <Card className="flex-1">
          <CardContent className="p-0">
            {isLoading ? (
              <div className="flex items-center justify-center min-h-[500px]">
                <div className="text-muted-foreground">Loading document...</div>
              </div>
            ) : error ? (
              <div className="flex items-center justify-center min-h-[500px]">
                <div className="text-destructive">{error}</div>
              </div>
            ) : (
              <Textarea
                className="min-h-[500px] border-0 resize-none text-base leading-relaxed font-mono focus-visible:ring-0 focus-visible:ring-offset-0"
                value={content}
                onChange={handleContentChange}
                placeholder="Start typing your document..."
                autoFocus
              />
            )}
          </CardContent>
        </Card>

        {/* History Section */}
        <Card className="w-80">
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <History className="h-4 w-4" />
              <CardTitle className="text-lg">Version History</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="p-4">
            {isLoadingHistory ? (
              <div className="text-center text-muted-foreground py-4">
                Loading history...
              </div>
            ) : history.length === 0 ? (
              <div className="text-center text-muted-foreground py-4">
                No history yet
              </div>
            ) : (
              <div className="space-y-3 max-h-[500px] overflow-y-auto">
                {history.map((entry, index) => {
                  const formatted = formatHistoryEntry(entry);
                  return (
                    <div key={index} className="border rounded-lg p-3 bg-muted/30">
                      <div className="flex items-center gap-2 mb-2">
                        <User className="h-3 w-3 text-muted-foreground" />
                        <span className="text-xs text-muted-foreground">
                          {formatted.ipAddress}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {formatted.timeAgo}
                        </span>
                      </div>
                      <p className="text-sm text-foreground">
                        {formatted.contentPreview}
                      </p>
                    </div>
                  );
                })}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
} 