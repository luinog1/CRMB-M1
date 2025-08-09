// useHeroBanner Hook
// Manages hero banner content with TMDB integration and rotation

import { useState, useEffect, useCallback, useRef } from 'react'
import { MediaItem } from '../types'
import { tmdbService } from '../services/tmdb'
import { dataTransformService } from '../services/dataTransform'

interface HeroBannerData {
  title: string
  description: string
  backdrop: string
  year: string
  rating: string
  type: string
  mediaItem: MediaItem
}

interface UseHeroBannerOptions {
  autoRotate?: boolean
  rotationInterval?: number
  contentSource?: 'trending' | 'popular' | 'upcoming'
  contentType?: 'movie' | 'tv' | 'mixed'
}

interface UseHeroBannerReturn {
  currentContent: HeroBannerData | null
  availableContent: HeroBannerData[]
  isLoading: boolean
  error: string | null
  currentIndex: number
  totalItems: number
  nextContent: () => void
  previousContent: () => void
  selectContent: (index: number) => void
  refreshContent: () => Promise<void>
  pauseRotation: () => void
  resumeRotation: () => void
  isRotationPaused: boolean
}

export const useHeroBanner = (options: UseHeroBannerOptions = {}): UseHeroBannerReturn => {
  const {
    autoRotate = true,
    rotationInterval = 8000, // 8 seconds
    contentSource = 'trending',
    contentType = 'mixed'
  } = options

  // State
  const [availableContent, setAvailableContent] = useState<HeroBannerData[]>([])
  const [currentIndex, setCurrentIndex] = useState(0)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isRotationPaused, setIsRotationPaused] = useState(false)

  // Refs
  const rotationIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const isInitializedRef = useRef(false)

  // Fetch content from TMDB
  const fetchContent = useCallback(async (): Promise<MediaItem[]> => {
    try {
      let response
      
      switch (contentSource) {
        case 'trending':
          if (contentType === 'movie') {
            response = await tmdbService.getTrending('movie', 'week', 1)
          } else if (contentType === 'tv') {
            response = await tmdbService.getTrending('tv', 'week', 1)
          } else {
            response = await tmdbService.getTrending('all', 'week', 1)
          }
          break
          
        case 'popular':
          if (contentType === 'movie') {
            response = await tmdbService.getPopularMovies(1)
          } else if (contentType === 'tv') {
            response = await tmdbService.getPopularTVShows(1)
          } else {
            // Mix popular movies and TV shows
            const [moviesResponse, tvResponse] = await Promise.all([
              tmdbService.getPopularMovies(1),
              tmdbService.getPopularTVShows(1)
            ])
            
            const movies = dataTransformService.transformMovieArray(moviesResponse.results.slice(0, 10))
            const tvShows = dataTransformService.transformTVShowArray(tvResponse.results.slice(0, 10))
            
            // Interleave movies and TV shows
            const mixed: MediaItem[] = []
            const maxLength = Math.max(movies.length, tvShows.length)
            
            for (let i = 0; i < maxLength; i++) {
              if (i < movies.length) mixed.push(movies[i])
              if (i < tvShows.length) mixed.push(tvShows[i])
            }
            
            return mixed.slice(0, 20)
          }
          break
          
        case 'upcoming':
          if (contentType === 'tv') {
            response = await tmdbService.getOnTheAirTVShows(1)
          } else {
            response = await tmdbService.getUpcomingMovies(1)
          }
          break
          
        default:
          response = await tmdbService.getTrending('all', 'week', 1)
      }

      // Transform based on content type
      if (contentType === 'movie') {
        return dataTransformService.transformMovieArray(response.results as any[])
      } else if (contentType === 'tv') {
        return dataTransformService.transformTVShowArray(response.results as any[])
      } else {
        return dataTransformService.transformMixedArray(response.results as any[])
      }
      
    } catch (err: any) {
      console.error('Failed to fetch hero banner content:', err)
      throw err
    }
  }, [contentSource, contentType])

  // Transform MediaItem to HeroBannerData
  const transformToHeroBannerData = useCallback((mediaItems: MediaItem[]): HeroBannerData[] => {
    return mediaItems
      .filter(item => item.backdrop || item.poster) // Only items with images
      .slice(0, 10) // Limit to 10 items for performance
      .map(item => ({
        ...dataTransformService.createHeroBannerData(item),
        mediaItem: item
      }))
  }, [])

  // Load content
  const loadContent = useCallback(async () => {
    try {
      setIsLoading(true)
      setError(null)
      
      const mediaItems = await fetchContent()
      const heroBannerData = transformToHeroBannerData(mediaItems)
      
      if (heroBannerData.length === 0) {
        throw new Error('No suitable content found for hero banner')
      }
      
      setAvailableContent(heroBannerData)
      
      // Reset to first item if current index is out of bounds
      if (currentIndex >= heroBannerData.length) {
        setCurrentIndex(0)
      }
      
    } catch (err: any) {
      console.error('Error loading hero banner content:', err)
      setError(err.message || 'Failed to load content')
      setAvailableContent([])
    } finally {
      setIsLoading(false)
    }
  }, [fetchContent, transformToHeroBannerData, currentIndex])

  // Navigation functions
  const nextContent = useCallback(() => {
    if (availableContent.length > 0) {
      setCurrentIndex(prev => (prev + 1) % availableContent.length)
    }
  }, [availableContent.length])

  const previousContent = useCallback(() => {
    if (availableContent.length > 0) {
      setCurrentIndex(prev => (prev - 1 + availableContent.length) % availableContent.length)
    }
  }, [availableContent.length])

  const selectContent = useCallback((index: number) => {
    if (index >= 0 && index < availableContent.length) {
      setCurrentIndex(index)
    }
  }, [availableContent.length])

  // Rotation control
  const pauseRotation = useCallback(() => {
    setIsRotationPaused(true)
    if (rotationIntervalRef.current) {
      clearInterval(rotationIntervalRef.current)
      rotationIntervalRef.current = null
    }
  }, [])

  const resumeRotation = useCallback(() => {
    setIsRotationPaused(false)
  }, [])

  // Refresh content
  const refreshContent = useCallback(async () => {
    await loadContent()
  }, [loadContent])

  // Setup auto-rotation
  useEffect(() => {
    if (autoRotate && !isRotationPaused && availableContent.length > 1 && !isLoading) {
      rotationIntervalRef.current = setInterval(() => {
        nextContent()
      }, rotationInterval)
    }

    return () => {
      if (rotationIntervalRef.current) {
        clearInterval(rotationIntervalRef.current)
        rotationIntervalRef.current = null
      }
    }
  }, [autoRotate, isRotationPaused, availableContent.length, isLoading, rotationInterval, nextContent])

  // Initial load
  useEffect(() => {
    if (!isInitializedRef.current) {
      isInitializedRef.current = true
      loadContent()
    }
  }, [loadContent])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (rotationIntervalRef.current) {
        clearInterval(rotationIntervalRef.current)
      }
    }
  }, [])

  // Current content
  const currentContent = availableContent.length > 0 ? availableContent[currentIndex] : null

  return {
    currentContent,
    availableContent,
    isLoading,
    error,
    currentIndex,
    totalItems: availableContent.length,
    nextContent,
    previousContent,
    selectContent,
    refreshContent,
    pauseRotation,
    resumeRotation,
    isRotationPaused
  }
}

export default useHeroBanner