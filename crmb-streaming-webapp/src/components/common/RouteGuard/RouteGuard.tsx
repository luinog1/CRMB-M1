import { ReactNode, useEffect } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAuth } from '../../../hooks/useAuth'
import { LoadingSpinner } from '../LoadingSpinner/LoadingSpinner'

interface RouteGuardProps {
  children: ReactNode
  requireAuth?: boolean
  requireAdult?: boolean
  redirectTo?: string
  fallback?: ReactNode
}

export function RouteGuard({
  children,
  requireAuth = false,
  requireAdult = false,
  redirectTo = '/login',
  fallback
}: RouteGuardProps) {
  const navigate = useNavigate()
  const location = useLocation()
  const { user, isLoading, preferences } = useAuth()

  useEffect(() => {
    if (isLoading) return

    // Check authentication requirement
    if (requireAuth && !user) {
      navigate(redirectTo, {
        state: { from: location },
        replace: true
      })
      return
    }

    // Check adult content requirement
    if (requireAdult && (!preferences?.adultContent || !user?.isAdult)) {
      navigate('/restricted', {
        state: { reason: 'adult_content_required' },
        replace: true
      })
      return
    }
  }, [user, isLoading, preferences, requireAuth, requireAdult, navigate, location, redirectTo])

  // Show loading state while checking authentication
  if (isLoading) {
    return fallback || <LoadingSpinner />
  }

  // Don't render children if requirements aren't met
  if (requireAuth && !user) {
    return null
  }

  if (requireAdult && (!preferences?.adultContent || !user?.isAdult)) {
    return null
  }

  return <>{children}</>
}

export default RouteGuard