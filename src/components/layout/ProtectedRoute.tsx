import type { ReactNode } from 'react';
import { Navigate } from 'react-router-dom';

import type { Role } from '../../types';
import { useSessionStore } from '../../store/sessionStore';

interface ProtectedRouteProps {
  role: Role;
  children: ReactNode;
}

export const ProtectedRoute = ({ role, children }: ProtectedRouteProps) => {
  const userRole = useSessionStore((state) => state.role);
  const sessionId = useSessionStore((state) => state.sessionId);

  if (!sessionId || userRole !== role) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
};
