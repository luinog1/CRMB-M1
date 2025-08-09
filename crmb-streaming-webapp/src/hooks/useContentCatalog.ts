// useContentCatalog Hook
// Manages content catalogs for different sections with TMDB integration

import { useState, useEffect, useCallback, useRef } from 'react'
import { MediaItem, Catalog } from '../types'
import { tmdbService } from '../services/tmdb'
import { dataTransformService } from '../services/dataTransform'

type CatalogType = 
  | 'popular_movies'
  | 'upcoming_movies'
  | 'top_rated_movies'
  | 'now_playing_movies'
  | 'popular_tv'
  | 'top_rated_tv'
  | 'on_the_air_tv'
  | 'trending_all'
  | 'trending_movies'
  | 'trending_tv'

interface UseContentCatalogOptions {
  catalogType: CatalogType
  autoRefresh?: boolean
  refreshInterval?: number
  pageSize?: number
}

interface UseContentCatalogReturn {
  catalog: Catalog | null
  items: MediaItem[]
  isLoading: boolean
  error: string | null
  hasMore: boolean
  currentPage: number
  totalPages: number
  loadMore: () => Promise<void>
  refresh: () => Promise<void>
  retry: () => Promise<void>
}

export const useContentCatalog = (options: UseContentCatalogOptions): UseContentCatalogReturn => {
  const {
    catalogType,
    autoRefresh = false,
    refreshInterval = 300000, // 5 minutes
    pageSize = 20
  } = options

  // State
  const [catalog, setCatalog] = useState<Catalog | null>(null)
  const [items, setItems] = useState<MediaItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(0)
  const [hasMore, setHasMore] = useState(false)

  // Refs
  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const abortControllerRef = useRef<AbortController | null>(null)
  const isInitializedRef = useRef(false)

  // Get catalog metadata
  const getCatalogMetadata = useCallback((type: CatalogType) => {
    const catalogMap = {
      popular_movies: { name: 'Popular Movies', type: 'movie' as const, source: 'tmdb' as const },
      upcoming_movies: { name: 'Upcoming Movies', type: 'movie' as const, source: 'tmdb' as const },
      top_rated_movies: { name: 'Top Rated Movies', type: 'movie' as const, source: 'tmdb' as const },
      now_playing_movies: { name: 'Now Playing', type: 'movie' as const, source: 'tmdb' as const },
      popular_tv: { name: 'Popular TV Shows', type: 'tv' as const, source: 'tmdb' as const },
      top_rated_tv: { name: 'Top Rated TV Shows', type: 'tv' as const, source: 'tmdb' as const },
      on_the_air_tv: { name: 'On The Air', type: 'tv' as const, source: 'tmdb' as const },
      trending_all: { name: 'Trending Now', type: 'mixed' as const, source: 'tmdb' as const },
      trending_movies: { name: 'Trending Movies', type: 'movie' as const, source: 'tmdb' as const },
      trending_tv: { name: 'Trending TV Shows', type: 'tv' as const, source: 'tmdb' as const }
    }

    return catalogMap[type]
  }, [])

  // Fetch content from TMDB
  const fetchContent = useCallback(async (page: number = 1) => {
    // Cancel previous request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }

    abortControllerRef.current = new AbortController()

    try {
      let response
      let transformedItems: MediaItem[]

      switch (catalogType) {
        case 'popular_movies':
          response = await tmdbService.getPopularMovies(page)
          transformedItems = dataTransformService.transformMovieArray(response.results)
          break

        case 'upcoming_movies':
          response = await tmdbService.getUpcomingMovies(page)
          transformedItems = dataTransformService.transformMovieArray(response.results)
          break

        case 'top_rated_movies':
          response = await tmdbService.getTopRatedMovies(page)
          transformedItems = dataTransformService.transformMovieArray(response.results)
          break

        case 'now_playing_movies':
          response = await tmdbService.getNowPlayingMovies(page)
          transformedItems = dataTransformService.transformMovieArray(response.results)
          break

        case 'popular_tv':
          response = await tmdbService.getPopularTVShows(page)
          transformedItems = dataTransformService.transformTVShowArray(response.results)
          break

        case 'top_rated_tv':
          response = await tmdbService.getTopRatedTVShows(page)
          transformedItems = dataTransformService.transformTVShowArray(response.results)
          break

        case 'on_the_air_tv':
          response = await tmdbService.getOnTheAirTVShows(page)
          transformedItems = dataTransformService.transformTVShowArray(response.results)
          break

        case 'trending_all':
          response = await tmdbService.getTrending('all', 'week', page)
          transformedItems = dataTransformService.transformMixedArray(response.results)
          break

        case 'trending_movies':
          response = await tmdbService.getTrending('movie', 'week', page)
          transformedItems = dataTransformService.transformMixedArray(response.results)
          break

        case 'trending_tv':
          response = await tmdbService.getTrending('tv', 'week', page)
          transformedItems = dataTransformService.transformMixedArray(response.results)
          break

        default:
          throw new Error(`Unknown catalog type: ${catalogType}`)
      }

      return {
        items: transformedItems,
        page: response.page,
        totalPages: response.total_pages,
        totalResults: response.total_results
      }

    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error(`Error fetching ${catalogType}:`, err)
        throw err
      }
      throw err
    }
  }, [catalogType])

  // Load content
  const loadContent = useCallback(async (page: number = 1, append: boolean = false) => {
    try {
      setIsLoading(true)
      if (!append) {
        setError(null)
      }

      const result = await fetchContent(page)
      
      // Update items
      if (append) {
        setItems(prev => [...prev, ...result.items])
      } else {
        setItems(result.items)
      }

      // Update pagination
      setCurrentPage(result.page)
      setTotalPages(result.totalPages)
      setHasMore(result.page < result.totalPages)

      // Update catalog metadata
      const metadata = getCatalogMetadata(catalogType)
      setCatalog({
        id: catalogType,
        name: metadata.name,
        type: metadata.type,
        source: metadata.source,
        items: append ? [...items, ...result.items] : result.items,
        lastUpdated: new Date().toISOString(),
        isLoading: false
      })

    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error(`Error loading ${catalogType}:`, err)
        setError(err.message || 'Failed to load content')
        
        if (!append) {
          setItems([])
          setHasMore(false)
        }

        // Update catalog with error
        const metadata = getCatalogMetadata(catalogType)
        setCatalog({
          id: catalogType,
          name: metadata.name,
          type: metadata.type,
          source: metadata.source,
          items: append ? items : [],
          lastUpdated: new Date().toISOString(),
          isLoading: false,
          error: err.message
        })
      }
    } finally {
      setIsLoading(false)
    }
  }, [catalogType, fetchContent, getCatalogMetadata, items])

  // Load more content
  const loadMore = useCallback(async () => {
    if (hasMore && !isLoading) {
      await loadContent(currentPage + 1, true)
    }
  }, [hasMore, isLoading, currentPage, loadContent])

  // Refresh content
  const refresh = useCallback(async () => {
    setCurrentPage(1)
    await loadContent(1, false)
  }, [loadContent])

  // Retry on error
  const retry = useCallback(async () => {
    await loadContent(currentPage, false)
  }, [loadContent, currentPage])

  // Setup auto-refresh
  useEffect(() => {
    if (autoRefresh && !isLoading && !error) {
      refreshIntervalRef.current = setInterval(() => {
        refresh()
      }, refreshInterval)
    }

    return () => {
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current)
        refreshIntervalRef.current = null
      }
    }
  }, [autoRefresh, isLoading, error, refreshInterval, refresh])

  // Initial load
  useEffect(() => {
    if (!isInitializedRef.current) {
      isInitializedRef.current = true
      loadContent(1, false)
    }
  }, [loadContent])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current)
      }
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [])

  // Reset when catalog type changes
  useEffect(() => {
    if (isInitializedRef.current) {
      setItems([])
      setCurrentPage(1)
      setTotalPages(0)
      setHasMore(false)
      setError(null)
      loadContent(1, false)
    }
  }, [catalogType, loadContent])

  return {
    catalog,
    items,
    isLoading,
    error,
    hasMore,
    currentPage,
    totalPages,
    loadMore,
    refresh,
    retry
  }
}

export default useContentCatalog