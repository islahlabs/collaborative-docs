import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Editor from './components/Editor';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { FileText, ArrowRight, LogOut, User } from 'lucide-react';
import { api } from '@/services/api';
import { AuthProvider, useAuth } from '@/contexts/AuthContext';
import { AuthPage } from '@/components/AuthPage';
import { GitHubIcon } from '@/components/ui/github-icon';

function AppContent() {
  const { user, token, logout, isLoading, redirectToAuth } = useAuth();

  const handleSave = (content: string) => {
    console.log('Saving content:', content);
  };

  const handleCreateDocument = async () => {
    try {
      const response = await api.createDocument(token || undefined);
      window.location.href = `/doc/${response.id}`;
    } catch (error: any) {
      console.error('Failed to create document:', error);
      
      // If user is not authenticated, redirect to auth page
      if (error.status === 401) {
        redirectToAuth(window.location.pathname);
        return;
      }
      
      // Show error message to user
      alert(error.message || 'Failed to create document. Please try again.');
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
          <p className="mt-2 text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  return (
    <Router>
      <div className="min-h-screen bg-gray-50">
        {/* Header with user info and logout */}
        <header className="bg-white shadow-sm border-b">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between items-center h-16">
              <div className="flex items-center">
                <FileText className="h-8 w-8 text-primary mr-3" />
                <h1 className="text-xl font-semibold">Collaborative Docs</h1>
              </div>
              <div className="flex items-center space-x-4">
                {/* GitHub Link */}
                <a
                  href="https://github.com/islahlabs/collaborative-docs"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center space-x-2 text-muted-foreground hover:text-foreground transition-colors"
                  title="View source on GitHub"
                >
                  <GitHubIcon className="h-5 w-5" />
                  <span className="hidden sm:inline">Source Code</span>
                </a>

                {user ? (
                  <>
                    <div className="flex items-center space-x-2">
                      <User className="h-4 w-4 text-muted-foreground" />
                      <span className="text-sm text-muted-foreground">{user.email}</span>
                      <span className="text-xs bg-primary/10 text-primary px-2 py-1 rounded">
                        {user.role_name}
                      </span>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={logout}
                      className="flex items-center space-x-2"
                    >
                      <LogOut className="h-4 w-4" />
                      <span>Logout</span>
                    </Button>
                  </>
                                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => redirectToAuth()}
                      className="flex items-center space-x-2"
                    >
                      <User className="h-4 w-4" />
                      <span>Sign In</span>
                    </Button>
                  )}
              </div>
            </div>
          </div>
        </header>

        <Routes>
          <Route path="/doc/:id" element={<Editor onSave={handleSave} />} />
          <Route path="/auth" element={<AuthPage />} />
          <Route path="/" element={
            <div className="min-h-screen flex items-center justify-center p-6">
              <Card className="w-full max-w-2xl">
                <CardHeader className="text-center">
                  <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                    <FileText className="h-8 w-8 text-primary" />
                  </div>
                  <CardTitle className="text-3xl">
                    {user ? `Welcome, ${user.email}!` : 'Welcome to Collaborative Docs!'}
                  </CardTitle>
                  <p className="text-muted-foreground">
                    {user ? 'Create and edit your collaborative documents.' : 'Create and edit collaborative documents.'}
                  </p>
                </CardHeader>
                <CardContent className="text-center space-y-4">
                  {user ? (
                    <>
                      <div className="text-sm text-muted-foreground">
                        <p>Your role: <span className="font-medium text-primary">{user.role_name}</span></p>
                        {user.role_name === 'user' && (
                          <p className="mt-2 text-orange-600">
                            ⚠️ You need document creation permission to create new documents.
                          </p>
                        )}
                      </div>
                      <Button 
                        onClick={handleCreateDocument} 
                        className="inline-flex items-center gap-2"
                        disabled={user.role_name === 'user'}
                      >
                        Create New Document
                        <ArrowRight className="h-4 w-4" />
                      </Button>
                    </>
                  ) : (
                    <>
                      <div className="text-sm text-muted-foreground">
                        <p>You can view and edit documents shared with you, or sign in to create new ones.</p>
                      </div>
                      <div className="flex flex-col sm:flex-row gap-3 justify-center">
                        <Button 
                          onClick={() => redirectToAuth()}
                          className="inline-flex items-center gap-2"
                        >
                          Sign In
                          <ArrowRight className="h-4 w-4" />
                        </Button>
                        <Button 
                          variant="outline"
                          onClick={() => redirectToAuth()}
                          className="inline-flex items-center gap-2"
                        >
                          Create Account
                        </Button>
                        <Button 
                          variant="secondary"
                          onClick={handleCreateDocument}
                          className="inline-flex items-center gap-2"
                        >
                          Try Create Document
                          <ArrowRight className="h-4 w-4" />
                        </Button>
                      </div>
                    </>
                  )}
                </CardContent>
              </Card>
              
            </div>
          } />
        </Routes>
      </div>
    </Router>
  );
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}

export default App;
