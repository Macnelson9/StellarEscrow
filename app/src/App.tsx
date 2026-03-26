import { Routes, Route, NavLink } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import TradeDetail from './pages/TradeDetail';
import CreateTrade from './pages/CreateTrade';
import './App.css';

export default function App() {
  return (
    <div className="app">
      <nav className="nav">
        <span className="nav-brand">StellarEscrow</span>
        <NavLink to="/" end>Dashboard</NavLink>
        <NavLink to="/trades/new">New Trade</NavLink>
      </nav>
      <main className="main">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/trades/new" element={<CreateTrade />} />
          <Route path="/trades/:id" element={<TradeDetail />} />
        </Routes>
      </main>
    </div>
  );
}
