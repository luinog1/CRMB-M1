# CRMB Streaming WebApp - Architecture Documentation

## 🏗️ System Architecture

### High-Level Overview

The CRMB Streaming WebApp follows a modern, scalable architecture designed for premium streaming experiences:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Backend API   │    │  External APIs  │
│  React/TypeScript│◄──►│   Rust/Axum    │◄──►│  TMDB, Stremio  │
│     Vite        │    │   PostgreSQL    │    │    MDBList      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Principles

1. **Component-Driven Development**: Modular, reusable components
2. **Type Safety**: Comprehensive TypeScript coverage
3. **Performance First**: 90+ Lighthouse scores
4. **Accessibility**: WCAG 2.1 AA compliance
5. **Mobile-First**: Responsive design approach
6. **Progressive Enhancement**: Works without JavaScript

## 📁 File Structure Standards

### Directory Organization

```
src/
├── components/              # Reusable UI components
│   ├── common/             # Generic components (Button, Input, etc.)
│   ├── media/              # Media-specific components
│   ├── layout/             # Layout components
│   └── forms/              # Form components
├── pages/                  # Page-level components
│   ├── Home/
│   ├── Search/
│   ├── Watchlist/
│   └── Settings/
├── hooks/                  # Custom React hooks
│   ├── api/               # API-related hooks
│   ├── ui/                # UI-related hooks
│   └── utils/             # Utility hooks
├── services/               # External service integrations
│   ├── tmdb/              # TMDB API service
│   ├── stremio/           # Stremio addon service
│   └── mdblist/           # MDBList service
├── store/                  # State management
│   ├── slices/            # Redux slices
│   ├── middleware/        # Custom middleware
│   └── selectors/         # Reselect selectors
├── utils/                  # Utility functions
│   ├── api/               # API utilities
│   ├── format/            # Formatting utilities
│   └── validation/        # Validation utilities
├── types/                  # TypeScript definitions
│   ├── api/               # API response types
│   ├── components/        # Component prop types
│   └── store/             # Store state types
├── styles/                 # Global styles
│   ├── base/              # Base styles
│   ├── components/        # Component styles
│   └── utilities/         # Utility classes
└── assets/                 # Static assets
    ├── images/
    ├── icons/
    └── fonts/
```

### Naming Conventions

#### Files and Directories
- **Components**: PascalCase (`MediaCard.tsx`)
- **Pages**: PascalCase (`HomePage.tsx`)
- **Hooks**: camelCase with `use` prefix (`useMediaQuery.ts`)
- **Services**: camelCase (`tmdbService.ts`)
- **Types**: camelCase with descriptive suffix (`mediaTypes.ts`)
- **Utilities**: camelCase (`formatDate.ts`)
- **Styles**: kebab-case (`media-card.css`)

#### Code Elements
- **Components**: PascalCase (`MediaCard`)
- **Functions**: camelCase (`fetchMovieData`)
- **Variables**: camelCase (`movieList`)
- **Constants**: SCREAMING_SNAKE_CASE (`API_BASE_URL`)
- **Types/Interfaces**: PascalCase (`MediaItem`, `TMDBResponse`)
- **CSS Classes**: kebab-case with BEM (`media-card__title`)

## 🎨 Design System Architecture

### CSS Custom Properties Structure

```css
:root {
  /* Color System */
  --color-primary: #007AFF;
  --color-primary-alpha: rgba(0, 122, 255, 0.1);
  --color-background: #000000;
  --color-surface: #1C1C1E;
  --color-border: #38383A;
  
  /* Typography */
  --font-family-primary: -apple-system, BlinkMacSystemFont, 'Segoe UI';
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-base: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.25rem;
  
  /* Spacing */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  
  /* Layout */
  --max-width: 1440px;
  --header-height: 70px;
  --navigation-width: 240px;
  --border-radius-sm: 4px;
  --border-radius-md: 8px;
  --border-radius-lg: 12px;
  
  /* Animation */
  --transition-base: 0.2s ease;
  --transition-slow: 0.3s ease;
  
  /* Z-Index Scale */
  --z-index-dropdown: 10;
  --z-index-modal: 100;
  --z-index-navigation: 50;
  --z-index-header: 100;
}
```

### Component Architecture

#### Component Structure
```typescript
// MediaCard.tsx
import { FC } from 'react'
import { MediaItem } from '@/types'
import './MediaCard.css'

interface MediaCardProps {
  item: MediaItem
  onSelect?: (item: MediaItem) => void
  className?: string
}

export const MediaCard: FC<MediaCardProps> = ({ 
  item, 
  onSelect, 
  className = '' 
}) => {
  // Component logic
  return (
    <div className={`media-card ${className}`}>
      {/* Component JSX */}
    </div>
  )
}
```

#### CSS Module Structure
```css
/* MediaCard.css */
.media-card {
  /* Base styles */
}

.media-card__poster {
  /* Element styles */
}

.media-card--featured {
  /* Modifier styles */
}

@media (max-width: 768px) {
  .media-card {
    /* Responsive styles */
  }
}
```

## 🔧 Technical Standards

### TypeScript Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "noImplicitReturns": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": true
  }
}
```

### Code Quality Rules

1. **No `any` types** - Use proper type definitions
2. **Explicit return types** for functions
3. **Consistent error handling** patterns
4. **Comprehensive prop validation**
5. **Accessibility attributes** required

### Performance Standards

1. **Bundle Size**: Main bundle < 250KB gzipped
2. **Code Splitting**: Route-based and component-based
3. **Image Optimization**: WebP/AVIF with fallbacks
4. **Lazy Loading**: Below-the-fold content
5. **Caching Strategy**: Service worker implementation

## 🔄 State Management Architecture

### Redux Toolkit Structure

```typescript
// store/index.ts
export const store = configureStore({
  reducer: {
    media: mediaSlice.reducer,
    user: userSlice.reducer,
    ui: uiSlice.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER],
      },
    }).concat(apiSlice.middleware),
})
```

### State Shape

```typescript
interface RootState {
  media: {
    trending: MediaItem[]
    popular: MediaItem[]
    watchlist: WatchlistItem[]
    loading: LoadingState
    error: string | null
  }
  user: {
    preferences: UserPreferences
    session: UserSession | null
  }
  ui: {
    theme: 'dark' | 'light'
    navigation: NavigationState
    modals: ModalState
  }
}
```

## 🌐 API Integration Patterns

### Service Layer Architecture

```typescript
// services/tmdbService.ts
class TMDBService {
  private baseURL = process.env.VITE_TMDB_BASE_URL
  private apiKey = process.env.VITE_TMDB_API_KEY
  
  async getPopularMovies(page = 1): Promise<TMDBResponse<TMDBMovie[]>> {
    // Implementation
  }
  
  async searchMovies(query: string): Promise<TMDBResponse<TMDBMovie[]>> {
    // Implementation
  }
}

export const tmdbService = new TMDBService()
```

### Error Handling Strategy

```typescript
// utils/errorHandler.ts
export class APIError extends Error {
  constructor(
    message: string,
    public status: number,
    public code: string
  ) {
    super(message)
    this.name = 'APIError'
  }
}

export const handleAPIError = (error: unknown): APIError => {
  // Error handling logic
}
```

## 📱 Responsive Design Strategy

### Breakpoint System

```css
/* Mobile First Approach */
.component {
  /* Mobile styles (default) */
}

@media (min-width: 769px) {
  .component {
    /* Tablet styles */
  }
}

@media (min-width: 1025px) {
  .component {
    /* Desktop styles */
  }
}

@media (min-width: 1441px) {
  .component {
    /* Large desktop styles */
  }
}
```

### Grid System

```css
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-lg);
}

@media (max-width: 768px) {
  .grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--spacing-md);
  }
}
```

## 🧪 Testing Architecture

### Testing Strategy

1. **Unit Tests**: Individual functions and components
2. **Integration Tests**: Component interactions
3. **E2E Tests**: User workflows
4. **Visual Regression**: UI consistency
5. **Performance Tests**: Load and stress testing

### Test Structure

```typescript
// __tests__/MediaCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react'
import { MediaCard } from '../MediaCard'
import { mockMediaItem } from '@/test-utils/mocks'

describe('MediaCard', () => {
  it('renders media item correctly', () => {
    render(<MediaCard item={mockMediaItem} />)
    expect(screen.getByText(mockMediaItem.title)).toBeInTheDocument()
  })
  
  it('calls onSelect when clicked', () => {
    const onSelect = jest.fn()
    render(<MediaCard item={mockMediaItem} onSelect={onSelect} />)
    fireEvent.click(screen.getByRole('button'))
    expect(onSelect).toHaveBeenCalledWith(mockMediaItem)
  })
})
```

## 🚀 Build and Deployment

### Build Configuration

```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  build: {
    target: 'es2020',
    minify: 'terser',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          router: ['react-router-dom'],
        },
      },
    },
  },
  optimizeDeps: {
    include: ['react', 'react-dom'],
  },
})
```

### Environment Configuration

```bash
# Development
VITE_NODE_ENV=development
VITE_API_URL=http://localhost:3001/api
VITE_DEV_MODE=true

# Production
VITE_NODE_ENV=production
VITE_API_URL=https://api.crmb-streaming.com
VITE_DEV_MODE=false
```

## 🤝 Agent Coordination Guidelines

### Communication Protocols

1. **Architecture Changes**: Must be approved by PA-001
2. **API Contracts**: Coordinated between AI-004 and BA-005
3. **Design System**: DS-003 owns all UI component standards
4. **Performance**: PT-007 validates all optimization decisions
5. **Deployment**: DD-008 manages all environment configurations

### Handoff Requirements

1. **Complete Documentation**: All changes must be documented
2. **Type Definitions**: Updated TypeScript interfaces
3. **Test Coverage**: Minimum 80% coverage for new code
4. **Performance Impact**: Lighthouse score validation
5. **Accessibility Audit**: WCAG compliance verification

### Integration Points

```typescript
// Shared interfaces for agent coordination
export interface AgentHandoff {
  agent: string
  task: string
  dependencies: string[]
  deliverables: string[]
  validation: ValidationCriteria
}

export interface ValidationCriteria {
  typeCheck: boolean
  linting: boolean
  testing: boolean
  performance: boolean
  accessibility: boolean
}
```

## 📊 Monitoring and Analytics

### Performance Monitoring

```typescript
// utils/performance.ts
export const trackPerformance = (metric: string, value: number) => {
  if (process.env.VITE_ENABLE_ANALYTICS === 'true') {
    // Analytics implementation
  }
}

export const measureComponentRender = (componentName: string) => {
  // Performance measurement
}
```

### Error Tracking

```typescript
// utils/errorTracking.ts
export const trackError = (error: Error, context: ErrorContext) => {
  if (process.env.NODE_ENV === 'production') {
    // Error tracking service
  }
}
```

---

**This architecture document serves as the foundation for all development agents. Any modifications must be coordinated through the Project Architect (PA-001).**