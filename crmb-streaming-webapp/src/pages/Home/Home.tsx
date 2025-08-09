import HeroBanner from '../../components/hero/HeroBanner/HeroBanner'
import MovieCarousel from '../../components/carousel/MovieCarousel/MovieCarousel'
import './Home.css'

const Home = () => {
  return (
    <div className="home">
      <HeroBanner />
      
      <div className="content-sections">
        <MovieCarousel 
          title="Trending Now" 
          type="movie" 
          category="trending" 
        />
        
        <MovieCarousel 
          title="Popular Movies" 
          type="movie" 
          category="popular" 
        />
        
        <MovieCarousel 
          title="Top Rated Movies" 
          type="movie" 
          category="top_rated" 
        />
        
        <MovieCarousel 
          title="Popular TV Shows" 
          type="tv" 
          category="popular" 
        />
        
        <MovieCarousel 
          title="Upcoming Movies" 
          type="movie" 
          category="upcoming" 
        />
        
        <MovieCarousel 
          title="Now Playing" 
          type="movie" 
          category="now_playing" 
        />
      </div>
    </div>
  )
}

export default Home