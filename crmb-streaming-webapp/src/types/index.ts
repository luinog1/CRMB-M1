// CRMB Streaming WebApp - Type Definitions
import { ReactNode } from 'react'

// TMDB API Types
export interface TMDBMovie {
  id: number
  title: string
  poster_path: string | null
  backdrop_path: string | null
  overview: string
  release_date: string
  vote_average: number
  vote_count: number
  genre_ids: number[]
  adult: boolean
  original_language: string
  original_title: string
  popularity: number
  video: boolean
}

export interface TMDBTVShow {
  id: number
  name: string
  poster_path: string | null
  backdrop_path: string | null
  overview: string
  first_air_date: string
  vote_average: number
  vote_count: number
  genre_ids: number[]
  origin_country: string[]
  original_language: string
  original_name: string
  popularity: number
}

export interface TVEpisode {
  id: number
  name: string
  overview: string
  vote_average: number
  vote_count: number
  air_date: string
  episode_number: number
  season_number: number
  runtime: number
  still_path: string | null
}

export interface TMDBPerson {
  id: number
  name: string
  profile_path: string | null
  adult: boolean
  known_for: (TMDBMovie | TMDBTVShow)[]
  known_for_department: string
  popularity: number
}

export interface TMDBGenre {
  id: number
  name: string
}

export interface TMDBConfiguration {
  images: {
    base_url: string
    secure_base_url: string
    backdrop_sizes: string[]
    logo_sizes: string[]
    poster_sizes: string[]
    profile_sizes: string[]
    still_sizes: string[]
  }
  change_keys: string[]
}

// Stremio Addon Protocol Types
export interface StremioManifest {
  id: string
  name: string
  description: string
  version: string
  resources: string[]
  types: string[]
  catalogs: StremioAddonCatalog[]
  idPrefixes?: string[]
  background?: string
  logo?: string
  contactEmail?: string
  behaviorHints?: {
    adult?: boolean
    p2p?: boolean
    configurable?: boolean
    configurationRequired?: boolean
  }
}

export interface StremioAddonCatalog {
  type: string
  id: string
  name: string
  extra?: StremioExtra[]
}

export interface StremioExtra {
  name: string
  options?: string[]
  isRequired?: boolean
}

export interface StremioMetaItem {
  id: string
  type: string
  name: string
  poster?: string
  background?: string
  logo?: string
  description?: string
  year?: string
  imdbRating?: string
  director?: string[]
  cast?: string[]
  genre?: string[]
  country?: string
  language?: string
  runtime?: string
  website?: string
  behaviorHints?: {
    defaultVideoId?: string
    hasScheduledVideos?: boolean
  }
  videos?: StremioVideo[]
}

export interface StremioVideo {
  id: string
  title: string
  released: string
  season?: number
  episode?: number
  overview?: string
  thumbnail?: string
  streams?: StremioStream[]
}

export interface StremioStream {
  url: string
  title?: string
  ytId?: string
  infoHash?: string
  fileIdx?: number
  mapHints?: string[]
  behaviorHints?: {
    notWebReady?: boolean
    bingeGroup?: string
    countryWhitelist?: string[]
    videoSize?: number
    videoHash?: string
  }
}

// MDBList Types
export interface MDBListRating {
  imdb_id: string
  tmdb_id?: number
  imdb_rating?: number
  tmdb_rating?: number
  letterboxd_rating?: number
  rt_rating?: number
  metacritic_rating?: number
  trakt_rating?: number
  mal_rating?: number
}

export interface MDBListItem {
  id: number
  title: string
  year: number
  type: 'movie' | 'show'
  imdb_id: string
  tmdb_id?: number
  poster?: string
  backdrop?: string
  rating?: MDBListRating
}

// Application Types
export interface MediaItem {
  id: string
  type: 'movie' | 'tv' | 'person'
  title: string
  poster?: string
  backdrop?: string
  overview?: string
  releaseDate?: string
  rating?: number
  genres?: string[]
  runtime?: number
  status?: 'released' | 'upcoming' | 'in_production'
  source: 'tmdb' | 'stremio' | 'mdblist'
  metadata?: TMDBMovie | TMDBTVShow | TMDBPerson | StremioMetaItem
}

export interface WatchlistItem {
  id: string
  mediaItem: MediaItem
  addedAt: string
  watchedAt?: string
  progress?: number
  rating?: number
  notes?: string
}

export interface UserPreferences {
  theme: 'dark' | 'light'
  language: string
  region: string
  adultContent: boolean
  autoplay: boolean
  quality: 'auto' | 'high' | 'medium' | 'low'
  subtitles: boolean
  notifications: boolean
}

export interface SearchResult {
  query: string
  results: MediaItem[]
  totalResults: number
  page: number
  totalPages: number
  searchTime: number
}

export interface Catalog {
  id: string
  name: string
  type: 'movie' | 'tv' | 'mixed'
  source: 'tmdb' | 'stremio' | 'mdblist' | 'custom'
  items: MediaItem[]
  lastUpdated: string
  isLoading: boolean
  error?: string
}

// API Response Types
export interface APIResponse<T> {
  success: boolean
  data?: T
  error?: string
  message?: string
  timestamp: string
}

export interface PaginatedResponse<T> {
  results: T[]
  page: number
  total_pages: number
  total_results: number
}

// Error Types
export interface AppError {
  code: string
  message: string
  details?: unknown
  timestamp: string
}

// Component Props Types
export interface BaseComponentProps {
  className?: string
  children?: ReactNode
}

export interface LoadingState {
  isLoading: boolean
  error?: string | null
}

// Navigation Types
export interface NavigationItem {
  id: string
  label: string
  path: string
  icon?: string
  badge?: number
  isActive?: boolean
}

// Image Types
export interface ImageConfig {
  baseUrl: string
  secureBaseUrl: string
  sizes: {
    poster: string[]
    backdrop: string[]
    profile: string[]
  }
}

export interface OptimizedImage {
  src: string
  srcSet?: string
  sizes?: string
  alt: string
  loading?: 'lazy' | 'eager'
  placeholder?: string
}

// Environment Types
export interface EnvironmentConfig {
  tmdb: {
    apiKey: string
    baseUrl: string
    imageBaseUrl: string
  }
  api: {
    baseUrl: string
  }
  features: {
    mdblistEnabled: boolean
    webpEnabled: boolean
    avifEnabled: boolean
    analyticsEnabled: boolean
    pwaEnabled: boolean
    offlineEnabled: boolean
  }
  development: {
    devMode: boolean
    logLevel: 'debug' | 'info' | 'warn' | 'error'
  }
}