import { useRef, useEffect } from 'react'
import EpisodeCard from '../EpisodeCard/EpisodeCard'
import MovieCard from '../MovieCard/MovieCard'
import { useContentCatalog } from '../../../hooks/useContentCatalog'
import { dataTransformService } from '../../../services/dataTransform'
import './ContentSection.css'

type CatalogType = 
  | 'popular_movies'
  | 'upcoming_movies'
  | 'top_rated_movies'
  | 'now_playing_movies'
  | 'popular_tv'
  | 'top_rated_tv'
  | 'on_the_air_tv'
  | 'trending_all'
  | 'trending_movies'
  | 'trending_tv'

interface ContentSectionProps {
  title: string
  type: 'episodes' | 'movies'
  catalogType: CatalogType
  showSeeAll?: boolean
  autoRefresh?: boolean
}

const ContentSection = ({ 
  title, 
  type, 
  catalogType, 
  showSeeAll = true, 
  autoRefresh = false 
}: ContentSectionProps) => {
  const sliderRef = useRef<HTMLDivElement>(null)
  
  // Use TMDB content catalog
  const { 
    items, 
    isLoading, 
    error, 
    hasMore, 
    loadMore, 
    retry 
  } = useContentCatalog({ 
    catalogType, 
    autoRefresh,
    pageSize: 20 
  })

  // Transform TMDB data for UI components
  const episodesData = type === 'episodes' 
    ? dataTransformService.createEpisodeData(items[0])
    : []
    
  const moviesData = type === 'movies'
    ? dataTransformService.createMovieCardData(items.slice(0, 10))
    : []

  useEffect(() => {
    const slider = sliderRef.current
    if (!slider) return

    let isDown = false
    let startX: number
    let scrollLeft: number

    const handleMouseDown = (e: MouseEvent) => {
      isDown = true
      slider.classList.add('active')
      startX = e.pageX - slider.offsetLeft
      scrollLeft = slider.scrollLeft
    }

    const handleMouseLeave = () => {
      isDown = false
      slider.classList.remove('active')
    }

    const handleMouseUp = () => {
      isDown = false
      slider.classList.remove('active')
    }

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDown) return
      e.preventDefault()
      const x = e.pageX - slider.offsetLeft
      const walk = (x - startX) * 2
      slider.scrollLeft = scrollLeft - walk
    }

    slider.addEventListener('mousedown', handleMouseDown)
    slider.addEventListener('mouseleave', handleMouseLeave)
    slider.addEventListener('mouseup', handleMouseUp)
    slider.addEventListener('mousemove', handleMouseMove)

    return () => {
      slider.removeEventListener('mousedown', handleMouseDown)
      slider.removeEventListener('mouseleave', handleMouseLeave)
      slider.removeEventListener('mouseup', handleMouseUp)
      slider.removeEventListener('mousemove', handleMouseMove)
    }
  }, [])

  const handleSeeAllClick = () => {
    console.log(`See all ${title} clicked`)
    // TODO: Implement navigation to full list with catalogType
  }

  const handleLoadMore = () => {
    if (hasMore && !isLoading) {
      loadMore()
    }
  }

  const handleRetry = () => {
    retry()
  }

  // Show loading state
  if (isLoading && items.length === 0) {
    return (
      <div className={`content-section ${type === 'movies' ? 'movies-section' : ''}`}>
        <div className="container">
          <div className="section-header">
            <h2 className="section-title">{title}</h2>
          </div>
          <div className="loading-state">
            <div className="loading-spinner"></div>
            <p>Loading {title.toLowerCase()}...</p>
          </div>
        </div>
      </div>
    )
  }

  // Show error state
  if (error && items.length === 0) {
    return (
      <div className={`content-section ${type === 'movies' ? 'movies-section' : ''}`}>
        <div className="container">
          <div className="section-header">
            <h2 className="section-title">{title}</h2>
          </div>
          <div className="error-state">
            <p>Failed to load {title.toLowerCase()}</p>
            <button className="retry-button" onClick={handleRetry}>
              Try Again
            </button>
          </div>
        </div>
      </div>
    )
  }

  // Show empty state
  if (!isLoading && items.length === 0) {
    return (
      <div className={`content-section ${type === 'movies' ? 'movies-section' : ''}`}>
        <div className="container">
          <div className="section-header">
            <h2 className="section-title">{title}</h2>
          </div>
          <div className="empty-state">
            <p>No {title.toLowerCase()} available</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={`content-section ${type === 'movies' ? 'movies-section' : ''}`}>
      <div className="container">
        <div className="section-header">
          <h2 className="section-title">{title}</h2>
          {showSeeAll && (
            <button className="see-all" onClick={handleSeeAllClick}>
              See All &gt;
            </button>
          )}
        </div>
        
        <div className={`${type}-container`}>
          <div 
            ref={sliderRef}
            className={`${type}-slider`}
          >
            {type === 'episodes' ? (
              episodesData.map((episode: any) => (
                <EpisodeCard key={episode.id} episode={episode} />
              ))
            ) : (
              moviesData.map((movie: any) => (
                <MovieCard key={movie.id} movie={movie} />
              ))
            )}
            
            {/* Load more button for horizontal scroll */}
            {hasMore && (
              <div className="load-more-card">
                <button 
                  className="load-more-button" 
                  onClick={handleLoadMore}
                  disabled={isLoading}
                >
                  {isLoading ? 'Loading...' : 'Load More'}
                </button>
              </div>
            )}
          </div>
        </div>
        
        {/* Loading indicator for additional content */}
        {isLoading && items.length > 0 && (
          <div className="loading-more">
            <div className="loading-spinner small"></div>
          </div>
        )}
      </div>
    </div>
  )
}

export default ContentSection