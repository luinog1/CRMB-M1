import { Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider } from './hooks/useAuth'
import ErrorBoundary from './components/common/ErrorBoundary'
import ProtectedRoute from './components/common/ProtectedRoute'
import Layout from './components/Layout/Layout'
import Home from './pages/Home/Home'
import Search from './pages/Search/Search'
import Settings from './pages/Settings/Settings'
import Watchlist from './pages/Watchlist/Watchlist'
import MovieDetail from './pages/MovieDetail/MovieDetail'
import EpisodeDetail from './pages/EpisodeDetail/EpisodeDetail'
import Login from './pages/Login/Login'
import Restricted from './pages/Restricted/Restricted'
import NotFound from './pages/NotFound/NotFound'
import './App.css'

function App() {
  return (
    <ErrorBoundary>
      <AuthProvider>
        <Routes>
          {/* Auth Routes - outside of main layout */}
          <Route path="/login" element={<Login />} />
          <Route path="/restricted" element={<Restricted />} />
          
          <Route path="/" element={<Layout />}>
            {/* Public Routes */}
            <Route index element={<Home />} />
            <Route path="search" element={<Search />} />
            
            {/* Protected Routes - require authentication */}
            <Route 
              path="watchlist" 
              element={
                <ProtectedRoute requireAuth={true}>
                  <Watchlist />
                </ProtectedRoute>
              } 
            />
            <Route 
              path="settings" 
              element={
                <ProtectedRoute requireAuth={true}>
                  <Settings />
                </ProtectedRoute>
              } 
            />
            
            {/* Content Detail Routes */}
            <Route path="movie/:movieId" element={<MovieDetail />} />
            <Route path="tv/:tvId" element={<MovieDetail />} />
            
            {/* Adult Content Routes - require adult verification */}
            <Route 
              path="tv/:tvId/season/:seasonNumber/episode/:episodeNumber" 
              element={
                <ProtectedRoute requireAdult={true}>
                  <EpisodeDetail />
                </ProtectedRoute>
              } 
            />
            
            {/* Redirect old routes */}
            <Route path="movies/:movieId" element={<Navigate to="/movie/:movieId" replace />} />
            <Route path="shows/:tvId" element={<Navigate to="/tv/:tvId" replace />} />
            
            {/* 404 Catch-all */}
            <Route path="*" element={<NotFound />} />
          </Route>
        </Routes>
      </AuthProvider>
    </ErrorBoundary>
  )
}

export default App