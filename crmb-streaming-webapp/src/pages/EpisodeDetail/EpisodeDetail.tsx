import { useParams, useNavigate } from 'react-router-dom'
import { useEffect, useState } from 'react'
import tmdbService from '../../services/tmdb'
import { TVEpisode } from '../../types'
import './EpisodeDetail.css'

const EpisodeDetail = () => {
  const { tvId, seasonNumber, episodeNumber } = useParams<{
    tvId: string
    seasonNumber: string
    episodeNumber: string
  }>()
  const navigate = useNavigate()
  const [episode, setEpisode] = useState<TVEpisode | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchEpisode = async () => {
      if (!tvId || !seasonNumber || !episodeNumber) return

      try {
        setLoading(true)
        const episodeData = await tmdbService.getTVEpisodeDetails(
          parseInt(tvId),
          parseInt(seasonNumber),
          parseInt(episodeNumber)
        )
        setEpisode(episodeData)
      } catch (err) {
        setError('Failed to load episode details')
        console.error('Error fetching episode:', err)
      } finally {
        setLoading(false)
      }
    }

    fetchEpisode()
  }, [tvId, seasonNumber, episodeNumber])

  if (loading) {
    return (
      <div className="episode-detail__loading">
        <div className="loading-spinner"></div>
        <p>Loading episode...</p>
      </div>
    )
  }

  if (error || !episode) {
    return (
      <div className="episode-detail__error">
        <h2>Episode Not Found</h2>
        <p>{error || 'The requested episode could not be found.'}</p>
        <button onClick={() => navigate('/')} className="btn btn-primary">
          Go Back Home
        </button>
      </div>
    )
  }

  return (
    <div className="episode-detail">
      <div 
        className="episode-detail__backdrop"
        style={{
          backgroundImage: `url(https://image.tmdb.org/t/p/original${episode.still_path})`
        }}
      >
        <div className="episode-detail__backdrop-overlay"></div>
      </div>

      <div className="episode-detail__content">
        <div className="episode-detail__header">
          <button 
            onClick={() => navigate(-1)} 
            className="episode-detail__back-btn"
          >
            ← Back
          </button>
        </div>

        <div className="episode-detail__info">
          <div className="episode-detail__still">
            <img 
              src={`https://image.tmdb.org/t/p/w500${episode.still_path}`}
              alt={episode.name}
              className="episode-detail__still-image"
            />
          </div>

          <div className="episode-detail__details">
            <h1 className="episode-detail__title">{episode.name}</h1>
            <p className="episode-detail__meta">
              Season {episode.season_number} • Episode {episode.episode_number}
            </p>
            
            <div className="episode-detail__stats">
              <span className="episode-detail__rating">
                ⭐ {episode.vote_average.toFixed(1)}
              </span>
              <span className="episode-detail__air-date">
                {episode.air_date && new Date(episode.air_date).toLocaleDateString()}
              </span>
            </div>

            <p className="episode-detail__overview">{episode.overview}</p>

            <div className="episode-detail__actions">
              <button className="btn btn-primary episode-detail__play-btn">
                ▶ Play Episode
              </button>
              <button className="btn btn-secondary episode-detail__watchlist-btn">
                + Add to Watchlist
              </button>
            </div>

            {episode.runtime && (
              <div className="episode-detail__runtime">
                <strong>Runtime:</strong> {episode.runtime} minutes
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default EpisodeDetail