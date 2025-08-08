/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_TMDB_API_KEY: string
  readonly VITE_TMDB_BASE_URL: string
  readonly VITE_TMDB_IMAGE_BASE_URL: string
  readonly VITE_API_URL: string
  readonly VITE_WS_URL: string
  readonly VITE_APP_NAME: string
  readonly VITE_APP_VERSION: string
  readonly VITE_ENABLE_ANALYTICS: string
  readonly VITE_ENABLE_PWA: string
  readonly VITE_CACHE_DURATION: string
  readonly VITE_IMAGE_QUALITY: string
  readonly VITE_LAZY_LOADING: string
  readonly VITE_INFINITE_SCROLL: string
  readonly VITE_DARK_MODE: string
  readonly VITE_REDUCED_MOTION: string
  readonly VITE_HIGH_CONTRAST: string
  readonly VITE_FEATURE_WATCHLIST: string
  readonly VITE_FEATURE_RECOMMENDATIONS: string
  readonly VITE_FEATURE_SOCIAL: string
  readonly VITE_FEATURE_OFFLINE: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// Global type declarations
declare global {
  interface Window {
    __DEV__: boolean
  }
}

export {}