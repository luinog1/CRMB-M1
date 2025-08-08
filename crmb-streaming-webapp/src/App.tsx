import { Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout/Layout'
import { Home } from './pages/Home/Home'
import { Search } from './pages/Search/Search'
import { Watchlist } from './pages/Watchlist/Watchlist'
import { Settings } from './pages/Settings/Settings'
import './styles/App.css'

function App() {
  return (
    <div className="app">
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Home />} />
          <Route path="search" element={<Search />} />
          <Route path="watchlist" element={<Watchlist />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </div>
  )
}

export default App