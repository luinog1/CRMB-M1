import './Navigation.css'

export const Navigation = () => {
  const navigationItems = [
    { id: 'home', label: 'Home', path: '/', icon: '🏠' },
    { id: 'search', label: 'Search', path: '/search', icon: '🔍' },
    { id: 'watchlist', label: 'Watchlist', path: '/watchlist', icon: '📚' },
    { id: 'settings', label: 'Settings', path: '/settings', icon: '⚙️' }
  ]

  return (
    <nav className="navigation">
      <div className="navigation__container">
        <ul className="navigation__list">
          {navigationItems.map(item => (
            <li key={item.id} className="navigation__item">
              <a 
                href={item.path}
                className="navigation__link"
                aria-label={item.label}
              >
                <span className="navigation__icon">{item.icon}</span>
                <span className="navigation__label">{item.label}</span>
              </a>
            </li>
          ))}
        </ul>
      </div>
    </nav>
  )
}