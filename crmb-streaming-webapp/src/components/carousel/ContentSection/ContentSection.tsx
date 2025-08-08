import { useRef, useEffect } from 'react'
import EpisodeCard from '../EpisodeCard/EpisodeCard'
import MovieCard from '../MovieCard/MovieCard'
import './ContentSection.css'

interface ContentSectionProps {
  title: string
  type: 'episodes' | 'movies'
  showSeeAll?: boolean
}

const ContentSection = ({ title, type, showSeeAll = true }: ContentSectionProps) => {
  const sliderRef = useRef<HTMLDivElement>(null)

  // Mock data for episodes
  const episodesData = [
    {
      id: 1,
      title: 'S01E03. The Beginning',
      description: 'A new journey starts as survivors search for hope...',
      duration: '45 min',
      thumbnail: '🎬',
      gradient: 'linear-gradient(45deg, #1a1a1a, #333)'
    },
    {
      id: 2,
      title: 'S02E05. The Return',
      description: 'An unexpected alliance forms in the wasteland...',
      duration: '52 min',
      thumbnail: '🔥',
      gradient: 'linear-gradient(45deg, #ff4444, #cc0000)',
      badge: '📅 in 3 days'
    },
    {
      id: 3,
      title: 'S01E04. The Storm',
      description: 'Nature unleashes its fury as the group seeks shelter...',
      duration: '48 min',
      thumbnail: '⚡',
      gradient: 'linear-gradient(45deg, #0066cc, #004499)'
    },
    {
      id: 4,
      title: 'S01E05. Night Falls',
      description: 'Darkness brings new threats and unexpected discoveries...',
      duration: '51 min',
      thumbnail: '🌙',
      gradient: 'linear-gradient(45deg, #663399, #441166)'
    }
  ]

  // Mock data for movies
  const moviesData = [
    { id: 1, emoji: '🎭', gradient: 'linear-gradient(45deg, #1a1a1a, #333)' },
    { id: 2, emoji: '🚀', gradient: 'linear-gradient(45deg, #ff4444, #cc0000)' },
    { id: 3, emoji: '⚔️', gradient: 'linear-gradient(45deg, #1a1a1a, #444)' },
    { id: 4, emoji: '🌊', gradient: 'linear-gradient(45deg, #0066cc, #004499)' },
    { id: 5, emoji: '🎯', gradient: 'linear-gradient(45deg, #1a1a1a, #333)' },
    { id: 6, emoji: '🌟', gradient: 'linear-gradient(45deg, #ff4444, #cc0000)' },
    { id: 7, emoji: '🎪', gradient: 'linear-gradient(45deg, #1a1a1a, #444)' }
  ]

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
    // TODO: Implement navigation to full list
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
              episodesData.map((episode) => (
                <EpisodeCard key={episode.id} episode={episode} />
              ))
            ) : (
              moviesData.map((movie) => (
                <MovieCard key={movie.id} movie={movie} />
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default ContentSection