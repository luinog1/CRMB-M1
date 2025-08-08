import './HeroBanner.css'

const HeroBanner = () => {
  const handleStartWatching = () => {
    console.log('Start watching clicked')
    // TODO: Implement start watching functionality
  }

  return (
    <div className="hero-section">
      <div className="container">
        <h1 className="hero-title">CRUMBLE</h1>
        <p className="hero-description">
          Stream the latest movies and TV shows in stunning quality. Discover your next 
          favorite series or catch up on blockbuster films.
        </p>
        <button className="start-watching-btn" onClick={handleStartWatching}>
          <svg fill="currentColor" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z"/>
          </svg>
          Start Watching
        </button>
      </div>
    </div>
  )
}

export default HeroBanner