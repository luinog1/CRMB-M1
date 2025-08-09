import { Routes, Route, Navigate } from 'react-router-dom'
import Layout from './components/Layout/Layout'
import Home from './pages/Home/Home'
import Search from './pages/Search/Search'
import Settings from './pages/Settings/Settings'
import Watchlist from './pages/Watchlist/Watchlist'
import MovieDetail from './pages/MovieDetail/MovieDetail'
import EpisodeDetail from './pages/EpisodeDetail/EpisodeDetail'
import NotFound from './pages/NotFound/NotFound'
import './App.css'

function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="search" element={<Search />} />
        <Route path="watchlist" element={<Watchlist />} />
        <Route path="settings" element={<Settings />} />
        
        {/* Content Detail Routes */}
        <Route path="movie/:movieId" element={<MovieDetail />} />
        <Route path="tv/:tvId" element={<MovieDetail />} />
        <Route path="tv/:tvId/season/:seasonNumber/episode/:episodeNumber" element={<EpisodeDetail />} />
        
        {/* Redirect old routes */}
        <Route path="movies/:movieId" element={<Navigate to="/movie/:movieId" replace />} />
        <Route path="shows/:tvId" element={<Navigate to="/tv/:tvId" replace />} />
        
        {/* 404 Catch-all */}
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  )
}

export default App