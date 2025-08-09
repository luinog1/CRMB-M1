import { useParams, useNavigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import tmdbService from '../../services/tmdb'
import { Movie, TVShow } from '../../types'
import './MovieDetail.css'

const MovieDetail = () => {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [content, setContent] = useState<Movie | TVShow | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchContent = async () => {
      if (!id) return

      try {
        setLoading(true)
        // Try to get movie details first
        const movieData = await tmdbService.getMovieDetails(parseInt(id))
        if (movieData) {
          setContent(movieData)
        } else {
          // If not found as movie, try TV show
          const tvData = await tmdbService.getTVShowDetails(parseInt(id))
          setContent(tvData)
        }
      } catch (err) {
        setError('Failed to load content details')
        console.error('Error fetching content:', err)
      } finally {
        setLoading(false)
      }
    }

    fetchContent()
  }, [id])

  if (loading) {
    return (
      <div className="movie-detail__loading">
        <div className="loading-spinner"></div>
        <p>Loading content...</p>
      </div>
    )
  }

  if (error || !content) {
    return (
      <div className="movie-detail__error">
        <h2>Content Not Found</h2>
        <p>{error || 'The requested content could not be found.'}</p>
        <button onClick={() => navigate('/')} className="btn btn-primary">
          Go Back Home
        </button>
      </div>
    )
  }

  const isMovie = 'title' in content
  const title = isMovie ? content.title : content.name
  const releaseDate = isMovie ? content.release_date : content.first_air_date
  const year = releaseDate ? new Date(releaseDate).getFullYear() : 'N/A'

  return (
    <div className="movie-detail">
      <div 
        className="movie-detail__backdrop"
        style={{
          backgroundImage: `url(https://image.tmdb.org/t/p/original${content.backdrop_path})`
        }}
      >
        <div className="movie-detail__backdrop-overlay"></div>
      </div>

      <div className="movie-detail__content">
        <div className="movie-detail__header">
          <button 
            onClick={() => navigate(-1)} 
            className="movie-detail__back-btn"
          >
            ← Back
          </button>
        </div>

        <div className="movie-detail__info">
          <div className="movie-detail__poster">
            <img 
              src={`https://image.tmdb.org/t/p/w500${content.poster_path}`}
              alt={title}
              className="movie-detail__poster-image"
            />
          </div>

          <div className="movie-detail__details">
            <h1 className="movie-detail__title">{title}</h1>
            <p className="movie-detail__year">{year}</p>
            
            <div className="movie-detail__meta">
              <span className="movie-detail__rating">
                ⭐ {content.vote_average.toFixed(1)}
              </span>
              <span className="movie-detail__runtime">
                {isMovie && 'runtime' in content && content.runtime 
                  ? `${content.runtime} min` 
                  : 'TV Series'}
              </span>
            </div>

            <p className="movie-detail__overview">{content.overview}</p>

            <div className="movie-detail__actions">
              <button className="btn btn-primary movie-detail__play-btn">
                ▶ Play
              </button>
              <button className="btn btn-secondary movie-detail__watchlist-btn">
                + Add to Watchlist
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MovieDetail