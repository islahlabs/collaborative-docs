import React, { useState } from 'react';
import { Login } from './Login';
import { Signup } from './Signup';
import { FileText } from 'lucide-react';

export const AuthPage: React.FC = () => {
  const [isLogin, setIsLogin] = useState(true);

  return (
    <div className="min-h-screen flex items-center justify-center p-6 bg-gray-50">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
            <FileText className="h-8 w-8 text-primary" />
          </div>
          <h1 className="text-3xl font-bold">Collaborative Docs</h1>
          <p className="text-muted-foreground mt-2">
            {isLogin ? 'Welcome back!' : 'Create your account'}
          </p>
        </div>

        {isLogin ? (
          <Login onSwitchToSignup={() => setIsLogin(false)} />
        ) : (
          <Signup onSwitchToLogin={() => setIsLogin(true)} />
        )}

        <div className="text-center mt-4">
          <button
            onClick={() => window.location.href = '/'}
            className="text-sm text-muted-foreground hover:text-primary"
          >
            ← Back to Home
          </button>
        </div>
      </div>
    </div>
  );
}; 