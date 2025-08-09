// TMDB API Service
// Implements comprehensive TMDB integration with rate limiting and error handling

import { TMDBMovie, TMDBTVShow, TMDBPerson, TMDBConfiguration, PaginatedResponse, APIResponse } from '../types'

class TMDBService {
  private baseUrl: string
  private imageBaseUrl: string
  private apiKey: string
  private requestQueue: Array<() => Promise<any>> = []
  private isProcessing = false
  private lastRequestTime = 0
  private requestCount = 0
  private readonly RATE_LIMIT = 40 // requests per 10 seconds
  private readonly RATE_WINDOW = 10000 // 10 seconds in milliseconds
  private configuration: TMDBConfiguration | null = null

  constructor() {
    this.baseUrl = import.meta.env.VITE_TMDB_BASE_URL || 'https://api.themoviedb.org/3'
    this.imageBaseUrl = import.meta.env.VITE_TMDB_IMAGE_BASE_URL || 'https://image.tmdb.org/t/p/'
    this.apiKey = import.meta.env.VITE_TMDB_API_KEY || ''
    
    if (!this.apiKey) {
      console.warn('TMDB API key not found. Please set VITE_TMDB_API_KEY environment variable.')
    }

    // Initialize configuration
    this.initializeConfiguration()
  }

  private async initializeConfiguration(): Promise<void> {
    try {
      this.configuration = await this.getConfiguration()
    } catch (error) {
      console.error('Failed to initialize TMDB configuration:', error)
    }
  }

  private async rateLimitedRequest<T>(url: string, options?: RequestInit): Promise<T> {
    return new Promise((resolve, reject) => {
      this.requestQueue.push(async () => {
        try {
          const response = await this.makeRequest<T>(url, options)
          resolve(response)
        } catch (error) {
          reject(error)
        }
      })
      
      this.processQueue()
    })
  }

  private async processQueue(): Promise<void> {
    if (this.isProcessing || this.requestQueue.length === 0) {
      return
    }

    this.isProcessing = true

    while (this.requestQueue.length > 0) {
      const now = Date.now()
      
      // Reset counter if window has passed
      if (now - this.lastRequestTime >= this.RATE_WINDOW) {
        this.requestCount = 0
        this.lastRequestTime = now
      }

      // Check if we can make a request
      if (this.requestCount >= this.RATE_LIMIT) {
        const waitTime = this.RATE_WINDOW - (now - this.lastRequestTime)
        await new Promise(resolve => setTimeout(resolve, waitTime))
        continue
      }

      const request = this.requestQueue.shift()
      if (request) {
        this.requestCount++
        await request()
        
        // Small delay between requests to be respectful
        await new Promise(resolve => setTimeout(resolve, 250))
      }
    }

    this.isProcessing = false
  }

  private async makeRequest<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = new URL(endpoint, this.baseUrl)
    url.searchParams.append('api_key', this.apiKey)

    const response = await fetch(url.toString(), {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new Error(`TMDB API Error: ${response.status} - ${errorData.status_message || response.statusText}`)
    }

    return response.json()
  }

  // Configuration
  async getConfiguration(): Promise<TMDBConfiguration> {
    if (this.configuration) {
      return this.configuration
    }

    const config = await this.rateLimitedRequest<TMDBConfiguration>('/configuration')
    this.configuration = config
    return config
  }

  // Image URL helpers
  getImageUrl(path: string | null, size: string = 'original', type: 'poster' | 'backdrop' | 'profile' = 'poster'): string {
    if (!path) {
      return this.getPlaceholderImage(type)
    }

    return `${this.imageBaseUrl}${size}${path}`
  }

  getPlaceholderImage(type: 'poster' | 'backdrop' | 'profile'): string {
    const placeholders = {
      poster: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzAwIiBoZWlnaHQ9IjQ1MCIgdmlld0JveD0iMCAwIDMwMCA0NTAiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHdpZHRoPSIzMDAiIGhlaWdodD0iNDUwIiBmaWxsPSIjMzMzIi8+Cjx0ZXh0IHg9IjE1MCIgeT0iMjI1IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0iQXJpYWwiIGZvbnQtc2l6ZT0iMTQiPk5vIEltYWdlPC90ZXh0Pgo8L3N2Zz4=',
      backdrop: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4MCIgaGVpZ2h0PSI3MjAiIHZpZXdCb3g9IjAgMCAxMjgwIDcyMCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjEyODAiIGhlaWdodD0iNzIwIiBmaWxsPSIjMzMzIi8+Cjx0ZXh0IHg9IjY0MCIgeT0iMzYwIiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0iQXJpYWwiIGZvbnQtc2l6ZT0iMjQiPk5vIEJhY2tkcm9wPC90ZXh0Pgo8L3N2Zz4=',
      profile: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTg1IiBoZWlnaHQ9IjI3OCIgdmlld0JveD0iMCAwIDE4NSAyNzgiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHdpZHRoPSIxODUiIGhlaWdodD0iMjc4IiBmaWxsPSIjMzMzIi8+Cjx0ZXh0IHg9IjkyLjUiIHk9IjEzOSIgZmlsbD0iIzY2NiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjEyIj5ObyBQcm9maWxlPC90ZXh0Pgo8L3N2Zz4='
    }
    return placeholders[type]
  }

  // Movies
  async getPopularMovies(page: number = 1): Promise<PaginatedResponse<TMDBMovie>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie>>(`/movie/popular?page=${page}`)
  }

  async getUpcomingMovies(page: number = 1): Promise<PaginatedResponse<TMDBMovie>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie>>(`/movie/upcoming?page=${page}`)
  }

  async getNowPlayingMovies(page: number = 1): Promise<PaginatedResponse<TMDBMovie>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie>>(`/movie/now_playing?page=${page}`)
  }

  async getTopRatedMovies(page: number = 1): Promise<PaginatedResponse<TMDBMovie>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie>>(`/movie/top_rated?page=${page}`)
  }

  async getMovieDetails(movieId: number): Promise<TMDBMovie> {
    return this.rateLimitedRequest<TMDBMovie>(`/movie/${movieId}`)
  }

  // TV Shows
  async getPopularTVShows(page: number = 1): Promise<PaginatedResponse<TMDBTVShow>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBTVShow>>(`/tv/popular?page=${page}`)
  }

  async getTopRatedTVShows(page: number = 1): Promise<PaginatedResponse<TMDBTVShow>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBTVShow>>(`/tv/top_rated?page=${page}`)
  }

  async getOnTheAirTVShows(page: number = 1): Promise<PaginatedResponse<TMDBTVShow>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBTVShow>>(`/tv/on_the_air?page=${page}`)
  }

  async getTVShowDetails(tvId: number): Promise<TMDBTVShow> {
    return this.rateLimitedRequest<TMDBTVShow>(`/tv/${tvId}`)
  }

  // Trending
  async getTrending(mediaType: 'all' | 'movie' | 'tv' | 'person' = 'all', timeWindow: 'day' | 'week' = 'week', page: number = 1): Promise<PaginatedResponse<TMDBMovie | TMDBTVShow | TMDBPerson>> {
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie | TMDBTVShow | TMDBPerson>>(`/trending/${mediaType}/${timeWindow}?page=${page}`)
  }

  // Search
  async searchMulti(query: string, page: number = 1): Promise<PaginatedResponse<TMDBMovie | TMDBTVShow | TMDBPerson>> {
    const encodedQuery = encodeURIComponent(query)
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie | TMDBTVShow | TMDBPerson>>(`/search/multi?query=${encodedQuery}&page=${page}`)
  }

  async searchMovies(query: string, page: number = 1): Promise<PaginatedResponse<TMDBMovie>> {
    const encodedQuery = encodeURIComponent(query)
    return this.rateLimitedRequest<PaginatedResponse<TMDBMovie>>(`/search/movie?query=${encodedQuery}&page=${page}`)
  }

  async searchTVShows(query: string, page: number = 1): Promise<PaginatedResponse<TMDBTVShow>> {
    const encodedQuery = encodeURIComponent(query)
    return this.rateLimitedRequest<PaginatedResponse<TMDBTVShow>>(`/search/tv?query=${encodedQuery}&page=${page}`)
  }

  async searchPeople(query: string, page: number = 1): Promise<PaginatedResponse<TMDBPerson>> {
    const encodedQuery = encodeURIComponent(query)
    return this.rateLimitedRequest<PaginatedResponse<TMDBPerson>>(`/search/person?query=${encodedQuery}&page=${page}`)
  }

  // TV Episodes
  async getTVEpisodeDetails(tvId: number, seasonNumber: number, episodeNumber: number): Promise<TVEpisode> {
    return this.rateLimitedRequest<TVEpisode>(`/tv/${tvId}/season/${seasonNumber}/episode/${episodeNumber}`)
  }

  // Utility methods
  isValidImagePath(path: string | null): boolean {
    return path !== null && path.length > 0 && path.startsWith('/')
  }

  getOptimalImageSize(containerWidth: number, type: 'poster' | 'backdrop' | 'profile' = 'poster'): string {
    const sizes = {
      poster: ['w92', 'w154', 'w185', 'w342', 'w500', 'w780', 'original'],
      backdrop: ['w300', 'w780', 'w1280', 'original'],
      profile: ['w45', 'w185', 'h632', 'original']
    }

    const availableSizes = sizes[type]
    
    for (const size of availableSizes) {
      if (size === 'original') return size
      const width = parseInt(size.substring(1))
      if (width >= containerWidth * 1.5) { // 1.5x for retina displays
        return size
      }
    }

    return 'original'
  }
}

// Export singleton instance
export const tmdbService = new TMDBService()
export default tmdbService