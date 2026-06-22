import { LogOut, Menu, X, type LucideIcon } from 'lucide-react';
import { useState } from 'react';
import { NavLink, Outlet, useNavigate } from 'react-router-dom';

import { cmd } from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';

export interface NavItem {
  label: string;
  path: string;
  icon: LucideIcon;
  description?: string;
}

interface AppShellProps {
  title: string;
  subtitle: string;
  navItems: NavItem[];
  profileItem?: NavItem;
}

const navClass = ({ isActive }: { isActive: boolean }) =>
  [
    'focus-ring flex items-center gap-3 rounded px-3 py-2 text-sm transition',
    isActive ? 'bg-white/12 text-white shadow-inner' : 'text-white/75 hover:bg-white/8 hover:text-white'
  ].join(' ');

export const AppShell = ({ title, subtitle, navItems, profileItem }: AppShellProps) => {
  const [isNavOpen, setIsNavOpen] = useState(false);
  const navigate = useNavigate();
  const { displayName, sessionId, clearSession } = useSessionStore();

  const handleLogout = async () => {
    if (sessionId) {
      await cmd<void>('logout', { sessionId }).catch(() => undefined);
    }
    clearSession();
    navigate('/', { replace: true });
  };

  return (
    <div className="flex h-screen min-w-0 overflow-hidden bg-background">
      {isNavOpen && (
        <button
          aria-label="Close navigation"
          className="fixed inset-0 z-40 bg-black/45 lg:hidden"
          onClick={() => setIsNavOpen(false)}
          type="button"
        />
      )}
      <aside
        className={[
          'fixed inset-y-0 left-0 z-50 w-60 shrink-0 flex-col bg-secondary text-white transition-transform duration-200 ease-out lg:static lg:flex lg:translate-x-0',
          isNavOpen ? 'flex translate-x-0' : 'hidden -translate-x-full lg:flex'
        ].join(' ')}
      >
        <div className="flex items-start justify-between gap-3 border-b border-white/10 p-4">
          <div className="min-w-0">
            <img
              alt="UEP logo"
              className="mb-3 h-12 w-12 object-contain drop-shadow-sm"
              src="/uep-logo.png"
            />
            <p className="text-sm font-semibold">{title}</p>
            <p className="text-xs text-white/60">{subtitle}</p>
          </div>
          <button
            aria-label="Close navigation"
            className="focus-ring -mr-1 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded text-white/80 hover:bg-white/10 lg:hidden"
            onClick={() => setIsNavOpen(false)}
            type="button"
          >
            <X size={18} />
          </button>
        </div>
        <nav className="flex-1 space-y-1 overflow-y-auto p-3">
          {navItems.map((item) => (
            <NavLink
              className={navClass}
              key={item.path}
              onClick={() => setIsNavOpen(false)}
              title={item.description ?? item.label}
              to={item.path}
            >
              <item.icon size={17} />
              {item.label}
            </NavLink>
          ))}
        </nav>
        <div className="border-t border-white/10 p-3">
          <p className="mb-2 truncate px-3 text-xs text-white/60">{displayName}</p>
          {profileItem && (
            <NavLink
              className={navClass}
              onClick={() => setIsNavOpen(false)}
              title={profileItem.description ?? profileItem.label}
              to={profileItem.path}
            >
              <profileItem.icon size={17} />
              {profileItem.label}
            </NavLink>
          )}
          <button
            className="focus-ring mt-1 flex w-full items-center gap-3 rounded px-3 py-2 text-left text-sm text-white/80 hover:bg-white/10"
            onClick={handleLogout}
            type="button"
          >
            <LogOut size={17} />
            Logout
          </button>
        </div>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex min-h-16 items-center gap-3 border-b border-border bg-white px-4 lg:hidden">
          <button
            aria-label="Open navigation"
            className="icon-btn shrink-0"
            onClick={() => setIsNavOpen(true)}
            type="button"
          >
            <Menu size={18} />
          </button>
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold text-secondary">{title}</p>
            <p className="truncate text-xs text-muted">{subtitle}</p>
          </div>
        </header>
        <main className="min-w-0 flex-1 overflow-y-auto p-4 lg:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
};
