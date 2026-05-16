import { LogIn, Shield } from 'lucide-react';
import { Link, Outlet } from 'react-router-dom';

export const GuestLayout = () => (
  <div className="flex h-screen flex-col bg-background">
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-secondary px-6 text-white">
      <div className="flex items-center gap-3">
        <img alt="UEP logo" className="h-10 w-10 rounded-full bg-white object-contain p-1" src="/uep-logo.png" />
        <div>
          <p className="text-sm font-semibold leading-tight">OVPSPEE Filing and Tracking System</p>
          <p className="text-xs text-white/70">Staff/Head Viewer</p>
        </div>
      </div>
      <Link
        className="focus-ring inline-flex items-center gap-2 rounded border border-white/20 px-3 py-2 text-sm hover:bg-white/10"
        to="/login"
      >
        <LogIn size={16} />
        Login
      </Link>
    </header>
    <main className="flex-1 overflow-y-auto p-6">
      <div className="mb-5 flex items-center gap-2 text-sm text-muted">
        <Shield size={16} />
        Authorized OVPSPEE staff and heads only
      </div>
      <Outlet />
    </main>
  </div>
);
