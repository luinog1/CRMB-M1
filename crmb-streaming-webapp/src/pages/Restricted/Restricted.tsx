import { Link, useLocation } from 'react-router-dom'
import { useAuth } from '../../hooks/useAuth'
import './Restricted.css'

interface LocationState {
  from?: string
}

export function Restricted() {
  const { user, updatePreferences } = useAuth()
  const location = useLocation()
  const from = (location.state as LocationState)?.from || '/'

  const handleEnableAdultContent = async () => {
    if (user) {
      await updatePreferences({ adultContent: true })
      // In a real app, this would require additional verification
      window.location.href = from
    }
  }

  return (
    <div className="restricted-page">
      <div className="restricted-container">
        <div className="restricted-icon">
          <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10"/>
            <path d="M4.93 4.93l14.14 14.14"/>
            <path d="M12 6v6"/>
            <path d="M12 16h.01"/>
          </svg>
        </div>
        
        <h1 className="restricted-title">Content Restricted</h1>
        
        <p className="restricted-message">
          This content is restricted and requires adult content verification to access.
        </p>

        {user ? (
          <div className="restricted-actions">
            <p className="restricted-note">
              You need to enable adult content in your account settings to view this content.
            </p>
            
            <div className="restricted-buttons">
              <button 
                className="restricted-button restricted-button--primary"
                onClick={handleEnableAdultContent}
              >
                Enable Adult Content
              </button>
              
              <Link 
                to="/settings" 
                className="restricted-button restricted-button--secondary"
              >
                Go to Settings
              </Link>
            </div>
          </div>
        ) : (
          <div className="restricted-actions">
            <p className="restricted-note">
              Please sign in to your account to access age-restricted content.
            </p>
            
            <div className="restricted-buttons">
              <Link 
                to="/login" 
                state={{ from }}
                className="restricted-button restricted-button--primary"
              >
                Sign In
              </Link>
              
              <Link 
                to="/" 
                className="restricted-button restricted-button--secondary"
              >
                Go Home
              </Link>
            </div>
          </div>
        )}

        <div className="restricted-info">
          <h3>Why is this content restricted?</h3>
          <ul>
            <li>Content may contain mature themes</li>
            <li>Age verification is required by content guidelines</li>
            <li>Helps ensure appropriate content for all users</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default Restricted