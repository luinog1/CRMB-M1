import React from 'react'
import './HeroBanner.css'
import { useHeroBanner } from '../../../hooks/useHeroBanner'

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
        <div className="container">
          <div className="hero-skeleton">
            <div className="skeleton-title"></div>
            <div className="skeleton-description"></div>
            <div className="skeleton-buttons"></div>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="hero-section hero-error">
        <div className="container">
          <h1 className="hero-title">CRUMBLE</h1>
          <p className="hero-description error-message">
            Unable to load featured content. Please check your connection.
          </p>
          <button className="start-watching-btn" onClick={refreshContent}>
            <svg fill="currentColor" viewBox="0 0 24 24">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            Try Again
          </button>
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
                 ⭐ {currentContent.rating}
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
            <button className="start-watching-btn primary" onClick={handleStartWatching}>
              <svg fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z"/>
              </svg>
              {currentContent.mediaItem.type === 'movie' ? 'Watch Now' : 'Start Watching'}
            </button>
            
            <button className="start-watching-btn secondary" onClick={handleAddToWatchlist}>
              <svg fill="currentColor" viewBox="0 0 24 24">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
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
                <svg fill="currentColor" viewBox="0 0 24 24">
                  <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
                </svg>
              </button>
              
              <button 
                className="nav-btn next" 
                onClick={nextContent}
                aria-label="Next content"
              >
                <svg fill="currentColor" viewBox="0 0 24 24">
                  <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
                </svg>
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default HeroBanner