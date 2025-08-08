import './Home.css'

export const Home = () => {
  return (
    <div className="home">
      <section className="hero">
        <div className="hero__content">
          <h1 className="hero__title">Welcome to CRMB Streaming</h1>
          <p className="hero__subtitle">
            Discover and stream your favorite movies and TV shows
          </p>
          <div className="hero__actions">
            <button className="btn btn--primary" type="button">
              Browse Movies
            </button>
            <button className="btn btn--secondary" type="button">
              Browse TV Shows
            </button>
          </div>
        </div>
        <div className="hero__background">
          <div className="hero__gradient"></div>
        </div>
      </section>
      
      <section className="content-section">
        <h2 className="section-title">Trending Now</h2>
        <div className="media-grid">
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
        </div>
      </section>
      
      <section className="content-section">
        <h2 className="section-title">Popular Movies</h2>
        <div className="media-grid">
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
          <div className="media-card media-card--placeholder">
            <div className="media-card__poster"></div>
            <div className="media-card__info">
              <h3 className="media-card__title">Loading...</h3>
              <p className="media-card__meta">Content loading</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}