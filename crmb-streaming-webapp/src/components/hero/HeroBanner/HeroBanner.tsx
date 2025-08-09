import React from 'react'
import './HeroBanner.css'
import { useHeroBanner } from '../../../hooks/useHeroBanner'
import { Play, Plus, ChevronLeft, ChevronRight, Star, RefreshCw } from 'lucide-react'

interface HeroBannerProps {
  contentSource?: 'trending' | 'popular' | 'upcoming'
  contentType?: 'movie' | 'tv' | 'mixed'
  autoRotate?: boolean
  rotationInterval?: number
}

const HeroBanner = ({ 
  contentSource = 'trending',
  contentType = 'mixed',
  autoRotate = true,
  rotationInterval = 10000
}: HeroBannerProps) => {
  const {
    currentContent,
    isLoading,
    error,
    nextContent,
    previousContent,
    refreshContent
  } = useHeroBanner({
    contentSource,
    contentType,
    autoRotate,
    rotationInterval
  })

  const handleStartWatching = () => {
    if (currentContent) {
      console.log('Start watching:', currentContent.title)
      // TODO: Implement navigation to content details/player
    }
  }

  const handleAddToWatchlist = () => {
    if (currentContent) {
      console.log('Add to watchlist:', currentContent.title)
      // TODO: Implement watchlist functionality
    }
  }

  if (isLoading) {
    return (
      <div className="hero-section hero-loading">
        <div className="hero-overlay"></div>
        <div className="container">
          <div className="hero-content">
            <div className="hero-skeleton">
              <div className="skeleton-title"></div>
              <div className="skeleton-description"></div>
              <div className="skeleton-description" style={{ width: '80%' }}></div>
              <div className="skeleton-buttons"></div>
            </div>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="hero-section hero-error">
        <div className="hero-overlay"></div>
        <div className="container">
          <div className="hero-content">
            <h1 className="hero-title">CRUMBLE</h1>
            <p className="hero-description error-message">
              Unable to load featured content. Please check your connection.
            </p>
            <button className="start-watching-btn" onClick={refreshContent}>
              <RefreshCw size={20} />
              Try Again
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (!currentContent) {
    return (
      <div className="hero-section">
        <div className="container">
          <h1 className="hero-title">CRUMBLE</h1>
          <p className="hero-description">
            Stream the latest movies and TV shows in stunning quality. Discover your next 
            favorite series or catch up on blockbuster films.
          </p>
        </div>
      </div>
    )
  }

  const backgroundImage = currentContent.backdrop
  const releaseYear = currentContent.year

  return (
    <div 
      className="hero-section"
      style={{
        backgroundImage: backgroundImage ? `url(${backgroundImage})` : undefined
      }}
    >
      <div className="hero-overlay"></div>
      <div className="container">
        <div className="hero-content">
          <h1 className="hero-title">{currentContent.title}</h1>
          
          <div className="hero-meta">
            {releaseYear && <span className="hero-year">{releaseYear}</span>}
            {currentContent.rating && (
              <span className="hero-rating">
                <Star size={16} style={{ display: 'inline-block', marginRight: '4px' }} />
                {currentContent.rating}
              </span>
            )}
            {currentContent.mediaItem.genres && currentContent.mediaItem.genres.length > 0 && (
              <span className="hero-genres">
                {currentContent.mediaItem.genres.slice(0, 3).join(' • ')}
              </span>
            )}
          </div>
           
           {currentContent.description && (
             <p className="hero-description">
               {currentContent.description.length > 200 
                 ? `${currentContent.description.substring(0, 200)}...` 
                 : currentContent.description
               }
             </p>
           )}
          
          <div className="hero-actions">
            <button className="start-watching-btn primary" onClick={handleStartWatching} aria-label={`Watch ${currentContent.title}`}>
              <Play size={20} />
              {currentContent.mediaItem.type === 'movie' ? 'Watch Now' : 'Start Watching'}
            </button>
            
            <button className="start-watching-btn secondary" onClick={handleAddToWatchlist} aria-label={`Add ${currentContent.title} to watchlist`}>
              <Plus size={20} />
              Add to List
            </button>
          </div>
          
          {autoRotate && (
            <div className="hero-navigation">
              <button 
                className="nav-btn prev" 
                onClick={previousContent}
                aria-label="Previous content"
              >
                <ChevronLeft size={24} />
              </button>
              
              <button 
                className="nav-btn next" 
                onClick={nextContent}
                aria-label="Next content"
              >
                <ChevronRight size={24} />
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default HeroBanner