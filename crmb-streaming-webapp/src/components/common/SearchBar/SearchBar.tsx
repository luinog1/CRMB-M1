import React, { useState, useEffect, useRef } from 'react'
import './SearchBar.css'
import { useSearch } from '../../../hooks/useSearch'
import { MediaItem } from '../../../types'

interface SearchBarProps {
  onSearchResults?: (results: any[]) => void
  onSearchStateChange?: (isSearching: boolean) => void
  placeholder?: string
  showResults?: boolean
}

const SearchBar = ({ 
  onSearchResults, 
  onSearchStateChange, 
  placeholder = "Search movies, shows...",
  showResults = true 
}: SearchBarProps) => {
  const [isFocused, setIsFocused] = useState(false)
  const [showDropdown, setShowDropdown] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)
  
  const {
    query,
    results,
    isLoading,
    error,
    hasMore,
    setQuery,
    clearSearch,
    loadMore,
    retry
  } = useSearch()

  // Handle search results callback
  useEffect(() => {
    if (onSearchResults) {
      onSearchResults(results)
    }
  }, [results, onSearchResults])

  // Handle search state callback
  useEffect(() => {
    if (onSearchStateChange) {
      onSearchStateChange(isLoading)
    }
  }, [isLoading, onSearchStateChange])

  // Handle click outside to close dropdown
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleFocus = () => {
    setIsFocused(true)
    if (query && results.length > 0) {
      setShowDropdown(true)
    }
  }

  const handleBlur = () => {
    setIsFocused(false)
    // Delay hiding dropdown to allow for clicks
    setTimeout(() => setShowDropdown(false), 150)
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setQuery(value)
    
    if (value.trim()) {
      setShowDropdown(true)
    } else {
      setShowDropdown(false)
      clearSearch()
    }
  }

  const handleClear = () => {
    setQuery('')
    clearSearch()
    setShowDropdown(false)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (query.trim()) {
      setShowDropdown(true)
    }
  }

  const handleResultClick = (result: any) => {
    console.log('Selected result:', result)
    setShowDropdown(false)
    // TODO: Navigate to result detail page
  }

  const handleLoadMore = () => {
    if (hasMore && !isLoading) {
      loadMore()
    }
  }

  const handleRetry = () => {
    if (query) {
      retry()
    }
  }

  return (
    <div className="search-container" ref={dropdownRef}>
      <form className="search-form" onSubmit={handleSubmit}>
        <div className="search-icon-container">
          {isLoading ? (
            <div className="search-loading-spinner"></div>
          ) : (
            <svg className="search-icon" fill="currentColor" viewBox="0 0 24 24">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
          )}
        </div>
        <input 
          type="text" 
          className={`search-input ${isFocused ? 'focused' : ''}`}
          placeholder={placeholder}
          value={query}
          onChange={handleInputChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
        />
        {query && (
          <button 
            type="button" 
            className="clear-button"
            onClick={handleClear}
            aria-label="Clear search"
          >
            <svg fill="currentColor" viewBox="0 0 24 24">
              <path d="M18 6 6 18M6 6l12 12"/>
            </svg>
          </button>
        )}
      </form>
      
      {/* Search Results Dropdown */}
      {showResults && showDropdown && (
        <div className="search-dropdown">
          {error ? (
            <div className="search-error">
              <p>Search failed: {error}</p>
              <button className="retry-button" onClick={handleRetry}>
                Try Again
              </button>
            </div>
          ) : results.length > 0 ? (
            <>
              <div className="search-results">
                {results.slice(0, 8).map((result) => (
                  <div 
                    key={result.id} 
                    className="search-result-item"
                    onClick={() => handleResultClick(result)}
                  >
                    <div className="result-poster">
                      {result.poster ? (
                        <img 
                          src={result.poster || '/placeholder-poster.jpg'} 
                          alt={result.title}
                          loading="lazy"
                          className="search-result-poster"
                        />
                      ) : (
                        <div className="result-placeholder">
                          {result.type === 'movie' ? '🎬' : 
                           result.type === 'tv' ? '📺' : '👤'}
                        </div>
                      )}
                    </div>
                    <div className="result-info">
                      <h4 className="result-title">{result.title}</h4>
                      <p className="result-meta">
                        {result.type === 'movie' ? 'Movie' : 
                         result.type === 'tv' ? 'TV Show' : 'Person'}
                        {result.releaseDate && ` • ${new Date(result.releaseDate).getFullYear()}`}
                        {result.rating && ` • ⭐ ${result.rating}`}
                      </p>
                      {result.overview && (
                        <p className="result-description">
                          {result.overview.length > 100 
                            ? `${result.overview.substring(0, 100)}...` 
                            : result.overview}
                        </p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
              
              {hasMore && (
                <div className="search-load-more">
                  <button 
                    className="load-more-button"
                    onClick={handleLoadMore}
                    disabled={isLoading}
                  >
                    {isLoading ? 'Loading...' : 'Load More Results'}
                  </button>
                </div>
              )}
              
              {results.length > 8 && (
                <div className="search-footer">
                  <p>Showing {Math.min(8, results.length)} of {results.length} results</p>
                </div>
              )}
            </>
          ) : query && !isLoading ? (
            <div className="search-empty">
              <p>No results found for "{query}"</p>
              <p className="search-suggestion">Try different keywords or check spelling</p>
            </div>
          ) : null}
        </div>
      )}
    </div>
  )
}

export default SearchBar