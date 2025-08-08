import './MovieCard.css'

interface Movie {
  id: number
  emoji: string
  gradient: string
}

interface MovieCardProps {
  movie: Movie
}

const MovieCard = ({ movie }: MovieCardProps) => {
  const handleClick = () => {
    console.log('Movie clicked:', movie.id)
    // TODO: Implement movie navigation
  }

  return (
    <div className="movie-card" onClick={handleClick}>
      <div 
        className="movie-poster"
        style={{ background: movie.gradient }}
      >
        <div className="movie-overlay">
          <div className="movie-emoji">
            {movie.emoji}
          </div>
        </div>
      </div>
    </div>
  )
}

export default MovieCard