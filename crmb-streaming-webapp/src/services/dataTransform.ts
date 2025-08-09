// Data Transformation Service
// Converts TMDB API responses to unified MediaItem format

import { TMDBMovie, TMDBTVShow, TMDBPerson, MediaItem } from '../types'
import { tmdbService } from './tmdb'

class DataTransformService {
  // Transform TMDB Movie to MediaItem
  transformMovie(movie: TMDBMovie): MediaItem {
    return {
      id: `tmdb-movie-${movie.id}`,
      type: 'movie',
      title: movie.title,
      poster: movie.poster_path ? tmdbService.getImageUrl(movie.poster_path, 'w500', 'poster') : undefined,
      backdrop: movie.backdrop_path ? tmdbService.getImageUrl(movie.backdrop_path, 'w1280', 'backdrop') : undefined,
      overview: movie.overview || undefined,
      releaseDate: movie.release_date || undefined,
      rating: movie.vote_average || undefined,
      genres: [], // Will be populated when we fetch genre details
      status: this.getMovieStatus(movie.release_date),
      source: 'tmdb',
      metadata: movie
    }
  }

  // Transform TMDB TV Show to MediaItem
  transformTVShow(tvShow: TMDBTVShow): MediaItem {
    return {
      id: `tmdb-tv-${tvShow.id}`,
      type: 'tv',
      title: tvShow.name,
      poster: tvShow.poster_path ? tmdbService.getImageUrl(tvShow.poster_path, 'w500', 'poster') : undefined,
      backdrop: tvShow.backdrop_path ? tmdbService.getImageUrl(tvShow.backdrop_path, 'w1280', 'backdrop') : undefined,
      overview: tvShow.overview || undefined,
      releaseDate: tvShow.first_air_date || undefined,
      rating: tvShow.vote_average || undefined,
      genres: [], // Will be populated when we fetch genre details
      status: 'released', // TV shows are typically released when they appear in TMDB
      source: 'tmdb',
      metadata: tvShow
    }
  }

  // Transform TMDB Person to MediaItem
  transformPerson(person: TMDBPerson): MediaItem {
    return {
      id: `tmdb-person-${person.id}`,
      type: 'person',
      title: person.name,
      poster: person.profile_path ? tmdbService.getImageUrl(person.profile_path, 'w185', 'profile') : undefined,
      overview: `Known for: ${person.known_for_department}`,
      rating: person.popularity,
      source: 'tmdb',
      metadata: person
    }
  }

  // Transform mixed TMDB results
  transformMixedResult(item: TMDBMovie | TMDBTVShow | TMDBPerson): MediaItem {
    // Check if it's a movie
    if ('title' in item && 'release_date' in item) {
      return this.transformMovie(item as TMDBMovie)
    }
    
    // Check if it's a TV show
    if ('name' in item && 'first_air_date' in item) {
      return this.transformTVShow(item as TMDBTVShow)
    }
    
    // Must be a person
    return this.transformPerson(item as TMDBPerson)
  }

  // Transform array of TMDB items
  transformMovieArray(movies: TMDBMovie[]): MediaItem[] {
    return movies.map(movie => this.transformMovie(movie))
  }

  transformTVShowArray(tvShows: TMDBTVShow[]): MediaItem[] {
    return tvShows.map(tvShow => this.transformTVShow(tvShow))
  }

  transformMixedArray(items: (TMDBMovie | TMDBTVShow | TMDBPerson)[]): MediaItem[] {
    return items.map(item => this.transformMixedResult(item))
  }

  // Helper methods
  private getMovieStatus(releaseDate?: string): 'released' | 'upcoming' | 'in_production' {
    if (!releaseDate) return 'in_production'
    
    const release = new Date(releaseDate)
    const now = new Date()
    
    if (release > now) {
      return 'upcoming'
    }
    
    return 'released'
  }

  // Create episode data for ContentSection (temporary mock data with TMDB styling)
  createEpisodeData(tvShow?: MediaItem): Array<{
    id: number
    title: string
    description: string
    duration: string
    thumbnail: string
    gradient: string
    badge?: string
  }> {
    const baseTitle = tvShow?.title || 'Unknown Series'
    
    return [
      {
        id: 1,
        title: `${baseTitle} - S01E01`,
        description: tvShow?.overview?.substring(0, 60) + '...' || 'Episode description not available',
        duration: '45 min',
        thumbnail: tvShow?.poster || '🎬',
        gradient: 'linear-gradient(45deg, #1a1a1a, #333)'
      },
      {
        id: 2,
        title: `${baseTitle} - S01E02`,
        description: 'The story continues with new developments...',
        duration: '52 min',
        thumbnail: tvShow?.poster || '🔥',
        gradient: 'linear-gradient(45deg, #ff4444, #cc0000)',
        badge: '📅 New'
      },
      {
        id: 3,
        title: `${baseTitle} - S01E03`,
        description: 'Tensions rise as characters face new challenges...',
        duration: '48 min',
        thumbnail: tvShow?.poster || '⚡',
        gradient: 'linear-gradient(45deg, #0066cc, #004499)'
      },
      {
        id: 4,
        title: `${baseTitle} - S01E04`,
        description: 'A turning point in the series unfolds...',
        duration: '51 min',
        thumbnail: tvShow?.poster || '🌙',
        gradient: 'linear-gradient(45deg, #663399, #441166)'
      }
    ]
  }

  // Create movie card data for ContentSection
  createMovieCardData(movies: MediaItem[]): Array<{
    id: number
    emoji: string
    gradient: string
    title?: string
    poster?: string
    mediaItem?: MediaItem
  }> {
    const gradients = [
      'linear-gradient(45deg, #1a1a1a, #333)',
      'linear-gradient(45deg, #ff4444, #cc0000)',
      'linear-gradient(45deg, #0066cc, #004499)',
      'linear-gradient(45deg, #663399, #441166)',
      'linear-gradient(45deg, #ff6b35, #f7931e)',
      'linear-gradient(45deg, #00d4aa, #00a085)',
      'linear-gradient(45deg, #8e44ad, #3498db)'
    ]

    const emojis = ['🎭', '🚀', '⚔️', '🌊', '🎯', '🌟', '🎪', '🔮', '🎨', '🎵']

    return movies.slice(0, 10).map((movie, index) => ({
      id: parseInt(movie.id.split('-')[2]) || index + 1,
      emoji: emojis[index % emojis.length],
      gradient: gradients[index % gradients.length],
      title: movie.title,
      poster: movie.poster,
      mediaItem: movie
    }))
  }

  // Format rating for display
  formatRating(rating?: number): string {
    if (!rating) return 'N/A'
    return rating.toFixed(1)
  }

  // Format release date for display
  formatReleaseDate(dateString?: string): string {
    if (!dateString) return 'TBA'
    
    try {
      const date = new Date(dateString)
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      })
    } catch {
      return dateString
    }
  }

  // Format year from release date
  formatYear(dateString?: string): string {
    if (!dateString) return 'TBA'
    
    try {
      const date = new Date(dateString)
      return date.getFullYear().toString()
    } catch {
      return 'TBA'
    }
  }

  // Get content type display name
  getContentTypeDisplayName(type: string): string {
    const typeMap: Record<string, string> = {
      'movie': 'Movie',
      'tv': 'TV Show',
      'person': 'Person'
    }
    
    return typeMap[type] || type
  }

  // Create hero banner data from MediaItem
  createHeroBannerData(mediaItem: MediaItem): {
    title: string
    description: string
    backdrop: string
    year: string
    rating: string
    type: string
  } {
    return {
      title: mediaItem.title,
      description: mediaItem.overview || 'No description available.',
      backdrop: mediaItem.backdrop || tmdbService.getPlaceholderImage('backdrop'),
      year: this.formatYear(mediaItem.releaseDate),
      rating: this.formatRating(mediaItem.rating),
      type: this.getContentTypeDisplayName(mediaItem.type)
    }
  }
}

// Export singleton instance
export const dataTransformService = new DataTransformService()
export default dataTransformService