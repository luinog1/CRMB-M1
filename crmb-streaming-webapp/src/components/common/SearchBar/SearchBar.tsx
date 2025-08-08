import { useState } from 'react'
import './SearchBar.css'

const SearchBar = () => {
  const [searchValue, setSearchValue] = useState('')
  const [isFocused, setIsFocused] = useState(false)

  const handleFocus = () => {
    setIsFocused(true)
  }

  const handleBlur = () => {
    setIsFocused(false)
  }

  const handleClear = () => {
    setSearchValue('')
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (searchValue.trim()) {
      console.log('Searching for:', searchValue)
      // TODO: Implement search functionality
    }
  }

  return (
    <div className="search-container">
      <form className="search-form" onSubmit={handleSubmit}>
        <div className="search-icon-container">
          <svg className="search-icon" fill="currentColor" viewBox="0 0 24 24">
            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
          </svg>
        </div>
        <input 
          type="text" 
          className={`search-input ${isFocused ? 'focused' : ''}`}
          placeholder="Search movies, shows..."
          value={searchValue}
          onChange={(e) => setSearchValue(e.target.value)}
          onFocus={handleFocus}
          onBlur={handleBlur}
        />
        {searchValue && (
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
    </div>
  )
}

export default SearchBar