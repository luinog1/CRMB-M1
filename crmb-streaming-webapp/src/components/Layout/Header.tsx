import './Header.css'

export const Header = () => {
  return (
    <header className="header">
      <div className="header__container">
        <div className="header__brand">
          <h1 className="header__logo">CRMB</h1>
          <span className="header__tagline">Streaming</span>
        </div>
        
        <div className="header__search">
          <div className="search-bar">
            <input 
              type="text" 
              placeholder="Search movies, TV shows..."
              className="search-bar__input"
            />
            <button className="search-bar__button" type="button">
              <span className="search-bar__icon">🔍</span>
            </button>
          </div>
        </div>
        
        <div className="header__actions">
          <button className="header__action" type="button">
            <span className="header__action-icon">🔔</span>
          </button>
          <button className="header__action" type="button">
            <span className="header__action-icon">👤</span>
          </button>
        </div>
      </div>
    </header>
  )
}