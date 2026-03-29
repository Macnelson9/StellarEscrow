import { useState } from 'react';
import { Routes, Route, NavLink } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import TradeDetail from './pages/TradeDetail';
import CreateTrade from './pages/CreateTrade';
import { ErrorBoundary } from './ErrorBoundary';
import './App.css';

export default function App() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  return (
    <ErrorBoundary>
      <div className="app">
        <nav className="nav">
          <span className="nav-brand">StellarEscrow</span>
          <button 
            className="nav-mobile-toggle" 
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            aria-label="Toggle navigation"
            aria-expanded={isMenuOpen}
          >
            ☰
          </button>
          <div className={`nav-links ${isMenuOpen ? 'nav-links-open' : ''}`}>
            <NavLink to="/" end onClick={() => setIsMenuOpen(false)}>Dashboard</NavLink>
            <NavLink to="/trades/new" onClick={() => setIsMenuOpen(false)}>New Trade</NavLink>
          </div>
        </nav>
        <main className="main">
          <ErrorBoundary>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/trades/new" element={<CreateTrade />} />
              <Route path="/trades/:id" element={<TradeDetail />} />
            </Routes>
          </ErrorBoundary>
        </main>
      </div>
    </ErrorBoundary>
  );
}
