import type { LucideIcon } from 'lucide-react';
import { LogOut } from 'lucide-react';
import { NavLink, Outlet, useNavigate } from 'react-router-dom';

import { logout } from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';

export interface NavItem {
  label: string;
  path: string;
  icon: LucideIcon;
}

interface AppShellProps {
  title: string;
  subtitle: string;
  navItems: NavItem[];
}

export const AppShell = ({ title, subtitle, navItems }: AppShellProps) => {
  const navigate = useNavigate();
  const { displayName, sessionId, clearSession } = useSessionStore();

  const handleLogout = async () => {
    if (sessionId) {
      await logout(sessionId).catch(() => undefined);
    }
    clearSession();
    navigate('/', { replace: true });
  };

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <aside className="flex w-60 shrink-0 flex-col bg-secondary text-white">
        <div className="border-b border-white/10 p-4">
          <div className="mb-3 flex h-10 w-10 items-center justify-center rounded bg-primary text-lg font-bold">
            O
          </div>
          <p className="text-sm font-semibold">{title}</p>
          <p className="text-xs text-white/60">{subtitle}</p>
        </div>
        <nav className="flex-1 space-y-1 p-3">
          {navItems.map((item) => (
            <NavLink
              className={({ isActive }) =>
                [
                  'focus-ring flex items-center gap-3 rounded px-3 py-2 text-sm transition',
                  isActive ? 'bg-white/12 text-white shadow-inner' : 'text-white/75 hover:bg-white/8 hover:text-white'
                ].join(' ')
              }
              key={item.path}
              to={item.path}
            >
              <item.icon size={17} />
              {item.label}
            </NavLink>
          ))}
        </nav>
        <div className="border-t border-white/10 p-3">
          <p className="mb-2 truncate px-3 text-xs text-white/60">{displayName}</p>
          <button
            className="focus-ring flex w-full items-center gap-3 rounded px-3 py-2 text-left text-sm text-white/80 hover:bg-white/10"
            onClick={handleLogout}
            type="button"
          >
            <LogOut size={17} />
            Logout
          </button>
        </div>
      </aside>
      <main className="flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
    </div>
  );
};
