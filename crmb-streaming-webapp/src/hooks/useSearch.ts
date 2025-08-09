// useSearch Hook
// Provides debounced search functionality with TMDB integration

import { useState, useEffect, useCallback, useRef } from 'react'
import { MediaItem, SearchResult } from '../types'
import { tmdbService } from '../services/tmdb'
import { dataTransformService } from '../services/dataTransform'

interface UseSearchOptions {
  debounceMs?: number
  minQueryLength?: number
  searchType?: 'multi' | 'movie' | 'tv' | 'person'
}

interface UseSearchReturn {
  query: string
  results: MediaItem[]
  isLoading: boolean
  error: string | null
  totalResults: number
  hasMore: boolean
  searchTime: number
  setQuery: (query: string) => void
  clearSearch: () => void
  loadMore: () => Promise<void>
  retry: () => Promise<void>
}

export const useSearch = (options: UseSearchOptions = {}): UseSearchReturn => {
  const {
    debounceMs = 300,
    minQueryLength = 2,
    searchType = 'multi'
  } = options

  // State
  const [query, setQueryState] = useState('')
  const [results, setResults] = useState<MediaItem[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [totalResults, setTotalResults] = useState(0)
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(0)
  const [searchTime, setSearchTime] = useState(0)

  // Refs
  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const abortControllerRef = useRef<AbortController | null>(null)
  const lastQueryRef = useRef('')

  // Perform search
  const performSearch = useCallback(async (searchQuery: string, page: number = 1, append: boolean = false) => {
    if (searchQuery.length < minQueryLength) {
      setResults([])
      setTotalResults(0)
      setTotalPages(0)
      setError(null)
      return
    }

    // Cancel previous request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }

    // Create new abort controller
    abortControllerRef.current = new AbortController()

    try {
      setIsLoading(true)
      setError(null)
      
      const startTime = performance.now()
      
      let response
      
      switch (searchType) {
        case 'movie':
          response = await tmdbService.searchMovies(searchQuery, page)
          break
        case 'tv':
          response = await tmdbService.searchTVShows(searchQuery, page)
          break
        case 'person':
          response = await tmdbService.searchPeople(searchQuery, page)
          break
        default:
          response = await tmdbService.searchMulti(searchQuery, page)
      }
      
      const endTime = performance.now()
      setSearchTime(endTime - startTime)

      // Transform results
      let transformedResults: MediaItem[]
      
      if (searchType === 'movie') {
        transformedResults = dataTransformService.transformMovieArray(response.results as any[])
      } else if (searchType === 'tv') {
        transformedResults = dataTransformService.transformTVShowArray(response.results as any[])
      } else {
        transformedResults = dataTransformService.transformMixedArray(response.results as any[])
      }

      // Update state
      if (append) {
        setResults(prev => [...prev, ...transformedResults])
      } else {
        setResults(transformedResults)
      }
      
      setTotalResults(response.total_results)
      setTotalPages(response.total_pages)
      setCurrentPage(response.page)
      
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error('Search error:', err)
        setError(err.message || 'Search failed. Please try again.')
        
        if (!append) {
          setResults([])
          setTotalResults(0)
          setTotalPages(0)
        }
      }
    } finally {
      setIsLoading(false)
    }
  }, [minQueryLength, searchType])

  // Debounced search
  const debouncedSearch = useCallback((searchQuery: string) => {
    // Clear existing timeout
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current)
    }

    // Set new timeout
    debounceTimeoutRef.current = setTimeout(() => {
      if (searchQuery !== lastQueryRef.current) {
        lastQueryRef.current = searchQuery
        setCurrentPage(1)
        performSearch(searchQuery, 1, false)
      }
    }, debounceMs)
  }, [debounceMs, performSearch])

  // Set query with debouncing
  const setQuery = useCallback((newQuery: string) => {
    setQueryState(newQuery)
    
    if (newQuery.trim() === '') {
      // Clear results immediately for empty query
      setResults([])
      setTotalResults(0)
      setTotalPages(0)
      setError(null)
      setCurrentPage(1)
      
      // Clear any pending debounced search
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current)
      }
    } else {
      debouncedSearch(newQuery.trim())
    }
  }, [debouncedSearch])

  // Clear search
  const clearSearch = useCallback(() => {
    setQuery('')
    setResults([])
    setTotalResults(0)
    setTotalPages(0)
    setError(null)
    setCurrentPage(1)
    setSearchTime(0)
    
    // Cancel any pending requests
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }
    
    // Clear debounce timeout
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current)
    }
  }, [setQuery])

  // Load more results
  const loadMore = useCallback(async () => {
    if (currentPage < totalPages && !isLoading && query.trim()) {
      await performSearch(query.trim(), currentPage + 1, true)
    }
  }, [currentPage, totalPages, isLoading, query, performSearch])

  // Retry search
  const retry = useCallback(async () => {
    if (query.trim()) {
      await performSearch(query.trim(), 1, false)
    }
  }, [query, performSearch])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current)
      }
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [])

  // Computed values
  const hasMore = currentPage < totalPages

  return {
    query,
    results,
    isLoading,
    error,
    totalResults,
    hasMore,
    searchTime,
    setQuery,
    clearSearch,
    loadMore,
    retry
  }
}

export default useSearch