import { useState } from 'react';
import { NavLink, Outlet, useNavigate } from 'react-router-dom';
import { Home, FolderOpen, MapPin, BarChart3, Menu, X, LogOut } from 'lucide-react';
import { useAuth } from '../hooks/useAuth';

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [menuOpen, setMenuOpen] = useState(false);

  const handleLogout = () => {
    logout();
    navigate('/login');
    setMenuOpen(false);
  };

  const closeMenu = () => setMenuOpen(false);

  const navLinkClass = ({ isActive }: { isActive: boolean }) =>
    `flex items-center gap-2 ${isActive ? 'active' : ''}`;

  return (
    <div className="flex flex-col" style={{ minHeight: '100vh' }}>
      <header className="app-header">
        <div className="header-inner">
          <NavLink to="/dashboard" className="header-logo">
            <Home size={22} />
            <span>小管家</span>
          </NavLink>

          <button
            className="mobile-menu-btn"
            onClick={() => setMenuOpen(!menuOpen)}
            aria-label="Toggle menu"
          >
            {menuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>

          <nav className={`header-nav ${menuOpen ? 'open' : ''}`}>
            <NavLink to="/dashboard" className={navLinkClass} onClick={closeMenu}>
              <Home size={18} />
              <span>首页</span>
            </NavLink>
            <NavLink to="/categories" className={navLinkClass} onClick={closeMenu}>
              <FolderOpen size={18} />
              <span>分类</span>
            </NavLink>
            <NavLink to="/locations" className={navLinkClass} onClick={closeMenu}>
              <MapPin size={18} />
              <span>地点</span>
            </NavLink>
            {user?.role === 'admin' && (
              <NavLink to="/stats" className={navLinkClass} onClick={closeMenu}>
                <BarChart3 size={18} />
                <span>统计</span>
              </NavLink>
            )}
          </nav>

          <div className={`header-user ${menuOpen ? '' : ''}`}>
            <span className="header-username">{user?.username}</span>
            <button className="btn btn-ghost btn-sm" onClick={handleLogout}>
              <LogOut size={16} />
              <span>退出</span>
            </button>
          </div>
        </div>
      </header>

      <main className="flex-1">
        <Outlet />
      </main>

      <footer className="app-footer">
        <p>&copy; {new Date().getFullYear()} 小管家 - 家庭物品管理系统</p>
      </footer>
    </div>
  );
}
