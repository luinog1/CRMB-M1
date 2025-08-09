import { ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { RouteGuard } from '../RouteGuard/RouteGuard'
import ErrorBoundary from '../ErrorBoundary'
import { useAuth } from '../../../hooks/useAuth'

interface ProtectedRouteProps {
  children: ReactNode
  requireAuth?: boolean
  requireAdult?: boolean
  fallbackPath?: string
  errorFallback?: ReactNode
}

export function ProtectedRoute({
  children,
  requireAuth = false,
  requireAdult = false,
  fallbackPath = '/login',
  errorFallback
}: ProtectedRouteProps) {
  const { user, isLoading } = useAuth()
  const location = useLocation()

  // Show loading while checking authentication
  if (isLoading) {
    return (
      <RouteGuard
        requireAuth={requireAuth}
        requireAdult={requireAdult}
        redirectTo={fallbackPath}
      >
        {children}
      </RouteGuard>
    )
  }

  // Redirect to login if authentication is required but user is not logged in
  if (requireAuth && !user) {
    return (
      <Navigate 
        to={fallbackPath} 
        state={{ from: location.pathname }} 
        replace 
      />
    )
  }

  // Redirect if adult content is required but user doesn't have access
  if (requireAdult && (!user || !user.isAdult)) {
    return (
      <Navigate 
        to="/restricted" 
        state={{ from: location.pathname }} 
        replace 
      />
    )
  }

  // Wrap in ErrorBoundary for error handling
  return (
    <ErrorBoundary fallback={errorFallback}>
      <RouteGuard
        requireAuth={requireAuth}
        requireAdult={requireAdult}
        redirectTo={fallbackPath}
      >
        {children}
      </RouteGuard>
    </ErrorBoundary>
  )
}

export default ProtectedRoute