import './EpisodeCard.css'

interface Episode {
  id: number
  title: string
  description: string
  duration: string
  thumbnail: string
  gradient: string
  badge?: string
}

interface EpisodeCardProps {
  episode: Episode
}

const EpisodeCard = ({ episode }: EpisodeCardProps) => {
  const handleClick = () => {
    console.log('Episode clicked:', episode.title)
    // TODO: Implement episode navigation
  }

  return (
    <div className="episode-card" onClick={handleClick}>
      <div className="episode-thumbnail">
        {episode.badge && (
          <div className="days-badge">
            {episode.badge}
          </div>
        )}
        <div 
          className="episode-thumbnail-content"
          style={{ background: episode.gradient }}
        >
          {episode.thumbnail}
        </div>
      </div>
      <div className="episode-info">
        <div className="episode-title">{episode.title}</div>
        <div className="episode-description">{episode.description}</div>
        <div className="episode-duration">{episode.duration}</div>
      </div>
    </div>
  )
}

export default EpisodeCard