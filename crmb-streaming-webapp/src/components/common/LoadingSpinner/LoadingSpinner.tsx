import './LoadingSpinner.css'

interface LoadingSpinnerProps {
  size?: 'small' | 'medium' | 'large'
  message?: string
  className?: string
}

export function LoadingSpinner({ 
  size = 'medium', 
  message = 'Loading...', 
  className = '' 
}: LoadingSpinnerProps) {
  return (
    <div className={`loading-spinner loading-spinner--${size} ${className}`}>
      <div className="loading-spinner__circle">
        <div className="loading-spinner__inner"></div>
      </div>
      {message && (
        <p className="loading-spinner__message">{message}</p>
      )}
    </div>
  )
}

export default LoadingSpinner