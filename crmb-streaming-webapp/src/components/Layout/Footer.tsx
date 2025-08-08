import './Footer.css'

export const Footer = () => {
  const currentYear = new Date().getFullYear()
  
  return (
    <footer className="footer">
      <div className="footer__container">
        <div className="footer__content">
          <div className="footer__section">
            <h3 className="footer__title">CRMB Streaming</h3>
            <p className="footer__description">
              Your premium destination for movies and TV shows.
            </p>
          </div>
          
          <div className="footer__section">
            <h4 className="footer__subtitle">Quick Links</h4>
            <ul className="footer__links">
              <li><a href="/" className="footer__link">Home</a></li>
              <li><a href="/search" className="footer__link">Search</a></li>
              <li><a href="/watchlist" className="footer__link">Watchlist</a></li>
              <li><a href="/settings" className="footer__link">Settings</a></li>
            </ul>
          </div>
          
          <div className="footer__section">
            <h4 className="footer__subtitle">Support</h4>
            <ul className="footer__links">
              <li><a href="/help" className="footer__link">Help Center</a></li>
              <li><a href="/contact" className="footer__link">Contact Us</a></li>
              <li><a href="/privacy" className="footer__link">Privacy Policy</a></li>
              <li><a href="/terms" className="footer__link">Terms of Service</a></li>
            </ul>
          </div>
        </div>
        
        <div className="footer__bottom">
          <p className="footer__copyright">
            © {currentYear} CRMB Streaming. All rights reserved.
          </p>
          <div className="footer__powered">
            <span>Powered by TMDB & Stremio</span>
          </div>
        </div>
      </div>
    </footer>
  )
}