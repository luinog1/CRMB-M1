import { useState, useEffect, createContext, useContext } from 'react'
import { ReactNode } from 'react'
import { UserPreferences } from '../types'

// Mock user type for now - will be replaced with actual auth implementation
interface User {
  id: string
  email: string
  name: string
  isAdult: boolean
  avatar?: string
  createdAt: string
}

interface AuthContextType {
  user: User | null
  preferences: UserPreferences | null
  isLoading: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => void
  updatePreferences: (preferences: Partial<UserPreferences>) => Promise<void>
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

// Default preferences
const defaultPreferences: UserPreferences = {
  theme: 'dark',
  language: 'en',
  region: 'US',
  adultContent: false,
  autoplay: true,
  quality: 'auto',
  subtitles: false,
  notifications: true
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [preferences, setPreferences] = useState<UserPreferences | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Simulate checking for existing session
    const checkAuth = async () => {
      try {
        const savedUser = localStorage.getItem('crmb_user')
        const savedPreferences = localStorage.getItem('crmb_preferences')
        
        if (savedUser) {
          setUser(JSON.parse(savedUser))
        }
        
        if (savedPreferences) {
          setPreferences(JSON.parse(savedPreferences))
        } else {
          setPreferences(defaultPreferences)
        }
      } catch (error) {
        console.error('Error checking auth:', error)
        setPreferences(defaultPreferences)
      } finally {
        setIsLoading(false)
      }
    }

    checkAuth()
  }, [])

  const login = async (email: string, password: string) => {
    setIsLoading(true)
    try {
      // Mock login - replace with actual API call
      const mockUser: User = {
        id: '1',
        email,
        name: email.split('@')[0],
        isAdult: true,
        createdAt: new Date().toISOString()
      }
      
      setUser(mockUser)
      localStorage.setItem('crmb_user', JSON.stringify(mockUser))
      
      if (!preferences) {
        setPreferences(defaultPreferences)
        localStorage.setItem('crmb_preferences', JSON.stringify(defaultPreferences))
      }
    } catch (error) {
      console.error('Login error:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const logout = () => {
    setUser(null)
    localStorage.removeItem('crmb_user')
    localStorage.removeItem('crmb_preferences')
  }

  const updatePreferences = async (newPreferences: Partial<UserPreferences>) => {
    if (!preferences) return
    
    const updated = { ...preferences, ...newPreferences }
    setPreferences(updated)
    localStorage.setItem('crmb_preferences', JSON.stringify(updated))
  }

  const value: AuthContextType = {
    user,
    preferences,
    isLoading,
    login,
    logout,
    updatePreferences
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}

export default useAuth