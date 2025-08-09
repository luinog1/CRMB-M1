import { useNavigate } from 'react-router-dom'
import './NotFound.css'

const NotFound = () => {
  const navigate = useNavigate()

  return (
    <div className="not-found">
      <div className="not-found__content">
        <h1 className="not-found__title">404</h1>
        <h2 className="not-found__subtitle">Page Not Found</h2>
        <p className="not-found__description">
          The page you're looking for doesn't exist or has been moved.
        </p>
        
        <div className="not-found__actions">
          <button 
            onClick={() => navigate(-1)} 
            className="btn btn-secondary not-found__btn"
          >
            Go Back
          </button>
          <button 
            onClick={() => navigate('/')} 
            className="btn btn-primary not-found__btn"
          >
            Go Home
          </button>
        </div>
      </div>
    </div>
  )
}

export default NotFound