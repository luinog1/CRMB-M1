import React, { useState, useEffect, useRef } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import tmdbService from '../../../services/tmdb';
import './MovieCarousel.css';

interface Movie {
  id: number;
  title: string;
  release_date: string;
  vote_average: number;
  poster_path: string | null;
  backdrop_path: string | null;
  overview: string;
  genre_ids: number[];
}

interface TVShow {
  id: number;
  name: string;
  first_air_date: string;
  vote_average: number;
  poster_path: string | null;
  backdrop_path: string | null;
  overview: string;
  genre_ids: number[];
}

interface MovieCarouselProps {
  title: string;
  type: 'movie' | 'tv';
  category: 'popular' | 'top_rated' | 'upcoming' | 'now_playing' | 'trending';
  maxItems?: number;
}

const MovieCarousel: React.FC<MovieCarouselProps> = ({
  title,
  type,
  category,
  maxItems = 20
}) => {
  const [items, setItems] = useState<(Movie | TVShow)[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(true);
  
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadContent();
  }, [type, category]);

  const loadContent = async () => {
    try {
      setIsLoading(true);
      setError(null);

      let response;

      if (category === 'trending') {
        response = await tmdbService.getTrending(type, 'week');
      } else {
        if (type === 'movie') {
          switch (category) {
            case 'popular':
              response = await tmdbService.getPopularMovies();
              break;
            case 'top_rated':
              response = await tmdbService.getTopRatedMovies();
              break;
            case 'upcoming':
              response = await tmdbService.getUpcomingMovies();
              break;
            case 'now_playing':
              response = await tmdbService.getNowPlayingMovies();
              break;
            default:
              response = await tmdbService.getPopularMovies();
          }
        } else {
          switch (category) {
            case 'popular':
              response = await tmdbService.getPopularTVShows();
              break;
            case 'top_rated':
              response = await tmdbService.getTopRatedTVShows();
              break;
            default:
              response = await tmdbService.getPopularTVShows();
          }
        }
      }

      const content = (response.results || []).slice(0, maxItems);
      setItems(content);

    } catch (err) {
      console.error('Failed to load carousel content:', err);
      setError('Unable to load content');
    } finally {
      setIsLoading(false);
    }
  };

  const handleScroll = () => {
    if (scrollRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
      setCanScrollLeft(scrollLeft > 0);
      setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 1);
    }
  };

  const scrollLeft = () => {
    if (scrollRef.current) {
      const scrollAmount = scrollRef.current.clientWidth * 0.8;
      scrollRef.current.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
    }
  };

  const scrollRight = () => {
    if (scrollRef.current) {
      const scrollAmount = scrollRef.current.clientWidth * 0.8;
      scrollRef.current.scrollBy({ left: scrollAmount, behavior: 'smooth' });
    }
  };

  const getImageUrl = (path: string | null) => {
    if (!path) return '/placeholder-poster.jpg';
    return `https://image.tmdb.org/t/p/w500${path}`;
  };

  const getYear = (date: string) => {
    return new Date(date).getFullYear();
  };

  const formatRating = (rating: number) => {
    return Math.round(rating * 10) / 10;
  };

  const getTitle = (item: Movie | TVShow) => {
    return 'title' in item ? item.title : item.name;
  };

  if (isLoading) {
    return (
      <div className="carousel-container">
        <div className="carousel-header">
          <h2 className="carousel-title">{title}</h2>
        </div>
        <div className="carousel-loading">
          <div className="carousel-skeleton">
            {Array.from({ length: 6 }).map((_, index) => (
              <div key={index} className="skeleton-card" />
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (error || items.length === 0) {
    return null;
  }

  return (
    <div className="carousel-container">
      <div className="carousel-header">
        <h2 className="carousel-title">{title}</h2>
        {items.length > 4 && (
          <div className="carousel-controls">
            <button
              className={`carousel-btn ${!canScrollLeft ? 'disabled' : ''}`}
              onClick={scrollLeft}
              disabled={!canScrollLeft}
              aria-label="Scroll left"
            >
              <ChevronLeft size={20} />
            </button>
            <button
              className={`carousel-btn ${!canScrollRight ? 'disabled' : ''}`}
              onClick={scrollRight}
              disabled={!canScrollRight}
              aria-label="Scroll right"
            >
              <ChevronRight size={20} />
            </button>
          </div>
        )}
      </div>

      <div className="carousel-wrapper">
        <div
          ref={scrollRef}
          className="carousel-scroll"
          onScroll={handleScroll}
          role="list"
          aria-label={`${title} carousel`}
        >
          {items.map((item) => (
            <div
              key={item.id}
              className="movie-card"
              role="listitem"
              onClick={() => console.log('Navigate to:', getTitle(item))}
            >
              <div className="movie-poster">
                <img
                  src={getImageUrl(item.poster_path)}
                  alt={getTitle(item)}
                  loading="lazy"
                  onError={(e) => {
                    (e.target as HTMLImageElement).src = '/placeholder-poster.jpg';
                  }}
                />
                <div className="movie-overlay">
                  <div className="movie-info">
                    <h3 className="movie-title">{getTitle(item)}</h3>
                    <div className="movie-meta">
                      <span className="movie-year">
                        {getYear('release_date' in item ? item.release_date : item.first_air_date)}
                      </span>
                      <span className="movie-rating">
                        {formatRating(item.vote_average)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default MovieCarousel;