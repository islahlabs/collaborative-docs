import { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Save, Clock } from 'lucide-react';

interface EditorProps {
  onSave?: (content: string) => void;
}

export default function Editor({ onSave }: EditorProps) {
  const [content, setContent] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const { id } = useParams<{ id: string }>();

  // Debounced save function
  const debouncedSave = useCallback(
    (() => {
      let timeoutId: number;
      return (text: string) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          setIsSaving(true);
          // TODO: Call API to save document
          console.log('Saving document:', id, text);
          onSave?.(text);
          setLastSaved(new Date());
          setIsSaving(false);
        }, 2000); // 2 second delay
      };
    })(),
    [id, onSave]
  );

  // Handle content changes
  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    setContent(newContent);
    debouncedSave(newContent);
  };

  // Load document content on mount
  useEffect(() => {
    // TODO: Load document content from API
    console.log('Loading document:', id);
  }, [id]);

  return (
    <div className="max-w-6xl mx-auto p-6 h-screen flex flex-col">
      <Card className="mb-6">
        <CardHeader className="pb-3">
          <div className="flex justify-between items-center">
            <CardTitle className="text-2xl">Document: {id}</CardTitle>
            <div className="flex items-center gap-3">
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
      
      <Card className="flex-1">
        <CardContent className="p-0">
          <Textarea
            className="min-h-[500px] border-0 resize-none text-base leading-relaxed font-mono focus-visible:ring-0 focus-visible:ring-offset-0"
            value={content}
            onChange={handleContentChange}
            placeholder="Start typing your document..."
            autoFocus
          />
        </CardContent>
      </Card>
    </div>
  );
} 