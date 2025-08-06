import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Editor from './components/Editor';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { FileText, ArrowRight } from 'lucide-react';
import { api } from '@/services/api';

function App() {
  const handleSave = (content: string) => {
    console.log('Saving content:', content);
  };

  const handleCreateDocument = async () => {
    try {
      const response = await api.createDocument();
      window.location.href = `/doc/${response.id}`;
    } catch (error) {
      console.error('Failed to create document:', error);
    }
  };

  return (
    <Router>
      <div className="min-h-screen bg-gray-50">
        <Routes>
          <Route path="/doc/:id" element={<Editor onSave={handleSave} />} />
          <Route path="/" element={
            <div className="min-h-screen flex items-center justify-center p-6">
              <Card className="w-full max-w-2xl">
                <CardHeader className="text-center">
                  <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                    <FileText className="h-8 w-8 text-primary" />
                  </div>
                  <CardTitle className="text-3xl">Collaborative Docs</CardTitle>
                  <p className="text-muted-foreground">Welcome to your collaborative document editor!</p>
                </CardHeader>
                <CardContent className="text-center space-y-4">
                  <p className="text-sm text-muted-foreground">
                    Navigate to <code className="bg-muted px-2 py-1 rounded font-mono text-sm">/doc/your-document-id</code> to start editing.
                  </p>
                  <Button onClick={handleCreateDocument} className="inline-flex items-center gap-2">
                    Create New Document
                    <ArrowRight className="h-4 w-4" />
                  </Button>
                </CardContent>
              </Card>
            </div>
          } />
        </Routes>
      </div>
    </Router>
  );
}

export default App;
