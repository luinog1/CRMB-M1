# CRMB Streaming WebApp - Automated Full Stack Team Configuration

## Project Overview
**Project Name**: CRMB Streaming WebApp  
**Description**: Premium media center web application inspired by Stremio with Apple TV+/Netflix aesthetics  
**Frontend**: React 18+ with TypeScript, Vite build tool, custom CSS  
**Backend**: Rust with Axum framework, SQLite/PostgreSQL database  
**Integrations**: TMDB API, MDBList ratings, Stremio addon protocol  
**Target**: 90+ Lighthouse scores, WCAG 2.1 AA compliance, 60fps performance  

## Automated Agent Team (8 Specialized Agents)

### 🏗️ Agent 1: Project Architect (PA)
**Status**: ✅ READY TO START  
**Role**: System design, architecture decisions, and cross-team coordination  
**Current Task**: Initialize project structure and define component architecture  

**Responsibilities**:
- Define project structure and file organization
- Create architectural blueprints and component hierarchies
- Establish coding standards and TypeScript interfaces
- Coordinate between agents and resolve integration conflicts
- Maintain project roadmap and milestone tracking

**Key Deliverables**:
```
src/
├── components/
│   ├── common/
│   │   ├── Sidebar/
│   │   ├── SearchBar/
│   │   └── LoadingSpinner/
│   ├── hero/
│   │   └── HeroBanner/
│   ├── carousel/
│   │   ├── MovieCarousel/
│   │   ├── CarouselItem/
│   │   └── CarouselControls/
│   └── pages/
│       ├── HomePage/
│       ├── SearchPage/
│       └── DetailsPage/
├── services/
│   ├── api.service.ts
│   ├── tmdb.service.ts
│   ├── mdblist.service.ts
│   └── stremio.service.ts
├── hooks/
│   ├── useDebounce.ts
│   ├── useInfiniteScroll.ts
│   └── useLazyLoading.ts
├── types/
│   ├── api.types.ts
│   ├── movie.types.ts
│   ├── stremio.types.ts
│   └── user.types.ts
└── utils/
    ├── imageUtils.ts
    └── errorHandling.ts
```

**Architecture Decisions**:
- React Context + useReducer for state management
- Custom CSS with CSS variables (no Tailwind)
- TypeScript strict mode with comprehensive interfaces
- Modular service architecture with proper error handling
- Mobile-first responsive design patterns

---

### 🎨 Agent 2: Design System (DS)
**Status**: ✅ READY TO START  
**Role**: CSS styling, animations, and visual design implementation  
**Current Task**: Implement dark theme design system and core animations  

**Responsibilities**:
- Implement dark theme design system with CSS variables
- Create responsive layouts and 60fps animations
- Build CSS utilities and component styles
- Ensure cross-browser compatibility
- Optimize for mobile and touch interactions

**Design Specifications**:
```css
:root {
  /* Core Colors */
  --bg-primary: #0a0a0a;      /* Pure black background */
  --bg-secondary: #1a1a1a;    /* Secondary dark background */
  --bg-elevated: #2a2a2a;     /* Card backgrounds */
  --text-primary: #ffffff;     /* Primary text */
  --text-secondary: #d1d5db;   /* Secondary text */
  --text-muted: #9ca3af;       /* Muted text */
  --accent-primary: #6366f1;   /* Primary accent */
  --accent-secondary: #84cc16; /* Secondary accent */
  --border-subtle: #374151;    /* Subtle borders */
  
  /* Typography Scale */
  --text-xs: 0.75rem;    /* 12px - metadata, tags */
  --text-sm: 0.875rem;   /* 14px - secondary text */
  --text-base: 1rem;     /* 16px - body text */
  --text-lg: 1.125rem;   /* 18px - card titles */
  --text-xl: 1.25rem;    /* 20px - section headers */
  --text-2xl: 1.5rem;    /* 24px - page titles */
  --text-3xl: 1.875rem;  /* 30px - hero titles */
  --text-4xl: 2.25rem;   /* 36px - main hero */
  
  /* Spacing System */
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */
  
  /* Animation */
  --transition-fast: 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-base: 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}
```

**Key Components to Style**:
1. Hero banner with gradient overlays (70vh height)
2. Content carousels with horizontal scrolling
3. Sidebar navigation (80px width, collapsible on mobile)
4. Content cards with hover effects (scale 1.05)
5. Search interface with expandable design
6. Loading skeletons and transition states

---

### ⚛️ Agent 3: Frontend Core (FC)
**Status**: ✅ READY TO START  
**Role**: React/TypeScript components, state management, and UI logic  
**Current Task**: Build core React components and routing structure  

**Responsibilities**:
- Build React components and custom hooks
- Implement state management with Context + useReducer
- Create reusable UI components and utilities
- Handle client-side routing and navigation
- Manage frontend error boundaries and loading states

**Core Components to Build**:
```typescript
// Hero Banner Component
interface HeroBannerProps {
  movie: TMDBMovie;
  onPlayClick: () => void;
  onWatchlistClick: () => void;
}

// Movie Carousel Component
interface MovieCarouselProps {
  title: string;
  movies: TMDBMovie[];
  loading?: boolean;
  onMovieClick: (movie: TMDBMovie) => void;
}

// Search Bar Component (EXACT HTML STRUCTURE REQUIRED)
const SearchBar: React.FC = () => {
  return (
    <form className='flex bg-zinc-800 border border-zinc-700 text-white rounded-md shadow text-sm'>
      <div aria-disabled='true' className='w-10 grid place-content-center'>
        <svg xmlns='http://www.w3.org/2000/svg' width='15' height='60' viewBox='0 0 24 24' fill='none' stroke='limegreen' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
          <circle cx='11' cy='11' r='8'></circle>
          <path d='m21 21-4.3-4.3'></path>
        </svg>
      </div>
      <input 
        type='text' 
        spellCheck='false' 
        name='text' 
        className='bg-transparent py-1.5 outline-none placeholder:text-zinc-400 w-20 focus:w-48 transition-all' 
        placeholder='Search...' 
      />
      <button className='w-10 grid place-content-center' aria-label='Clear input button' type='reset'>
        <svg strokeLinejoin='round' strokeLinecap='round' strokeWidth='2' stroke='limegreen' fill='none' viewBox='0 0 24 24' height='16' width='16'>
          <path d='M18 6 6 18'></path>
          <path d='m6 6 12 12'></path>
        </svg>
      </button>
    </form>
  );
};
```

**State Management Architecture**:
```typescript
interface AppState {
  user: UserState;
  content: ContentState;
  ui: UIState;
  stremio: StremioState;
}

interface ContentState {
  popular: TMDBMovie[];
  trending: TMDBMovie[];
  searchResults: TMDBMovie[];
  currentItem: TMDBMovie | null;
  loading: LoadingState;
  error: ErrorState | null;
}
```

---

### 🔌 Agent 4: API Integration (AI)
**Status**: ✅ READY TO START  
**Role**: External API connections, data fetching, and service integration  
**Current Task**: Implement TMDB API client with rate limiting and MDBList integration  

**Responsibilities**:
- Implement TMDB API client with rate limiting (40 req/10sec)
- Integrate MDBList rating aggregation
- Build Stremio addon protocol support
- Handle API error states and fallbacks
- Create caching and performance optimizations

**Key Integrations**:

**1. TMDB API Service**:
```typescript
class TMDBClientService {
  private apiKey: string;
  private baseUrl = 'https://api.themoviedb.org/3';
  private rateLimiter = new Map<string, number>();
  
  // Rate limiting: 40 requests per 10 seconds
  private checkRateLimit(): boolean {
    const now = Date.now();
    const windowStart = now - 10000;
    const currentRequests = Array.from(this.rateLimiter.values())
      .reduce((sum, count) => sum + count, 0);
    return currentRequests < 40;
  }
  
  async getPopularMovies(page = 1): Promise<TMDBMovie[]>
  async searchMulti(query: string): Promise<TMDBSearchResult[]>
  async getTrendingContent(): Promise<TMDBTrendingResult[]>
  buildImageUrl(path: string, size: string): string
}
```

**2. MDBList Integration**:
```typescript
class MDBListService {
  // Method 1: Public JSON Access (No API Key Required)
  async getPublicList(username: string, listSlug: string): Promise<MDBListItem[]>
  
  // Method 2: Official API (Requires API Key)
  async getUserLists(): Promise<MDBList[]>
  
  // Method 3: RapidAPI Integration
  async searchWithRatings(query: string): Promise<MDBSearchResult[]>
}
```

**3. Stremio Addon Protocol**:
```typescript
interface StremioManifest {
  id: string;
  name: string;
  resources: string[];
  types: string[];
  catalogs: Catalog[];
}

class StremioAddonService {
  async getCatalogFromAddon(addonUrl: string, type: string, id: string): Promise<StremioMetaPreview[]>
  async getMetaFromAddon(addonUrl: string, type: string, id: string): Promise<StremioMetaDetail>
  async getStreamsFromAddon(addonUrl: string, type: string, id: string): Promise<StremioStream[]>
}
```

---

### 🦀 Agent 5: Backend API (BA)
**Status**: ✅ READY TO START  
**Role**: Rust/Axum backend services, database operations, and API proxy  
**Current Task**: Build Rust backend with Axum framework and database setup  

**Responsibilities**:
- Build Rust backend with Axum framework
- Implement API proxy endpoints with rate limiting
- Handle database operations (SQLite/PostgreSQL)
- Create authentication and session management
- Implement CORS and security measures

**Backend Architecture**:
```rust
// main.rs
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let tmdb_service = Arc::new(TMDBService::new(
        std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY must be set")
    ).await.unwrap());
    
    let app = Router::new()
        .route("/api/movies/popular", get(get_popular_movies))
        .route("/api/movies/trending", get(get_trending_movies))
        .route("/api/search", get(search_movies))
        .route("/api/stremio/:type/:id", get(get_streams))
        .route("/api/watchlist", post(add_to_watchlist))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(rate_limit_middleware))
        .with_state(tmdb_service);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**Required Dependencies**:
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

### 🔄 Agent 6: State Management (SM)
**Status**: ✅ READY TO START  
**Role**: Application state, data flow, and client-side data management  
**Current Task**: Design React Context and reducer patterns with local storage  

**Responsibilities**:
- Design React Context and reducer patterns
- Implement local storage persistence
- Manage application state synchronization
- Handle optimistic UI updates
- Create state debugging and dev tools

**State Architecture**:
```typescript
// State Types
interface AppState {
  user: UserState;
  content: ContentState;
  ui: UIState;
  stremio: StremioState;
}

interface UserState {
  watchlist: ContentItem[];
  favorites: ContentItem[];
  watchProgress: Record<string, number>;
  preferences: UserPreferences;
  isAuthenticated: boolean;
}

// Reducer Actions
type ContentAction = 
  | { type: 'SET_POPULAR'; payload: ContentItem[] }
  | { type: 'SET_LOADING'; payload: { key: string; loading: boolean } }
  | { type: 'SET_ERROR'; payload: ErrorState }
  | { type: 'CLEAR_ERROR' }
  | { type: 'UPDATE_SEARCH_RESULTS'; payload: ContentItem[] };

// Custom Hooks
const useContent = () => {
  const context = useContext(ContentContext);
  if (!context) throw new Error('useContent must be used within ContentProvider');
  return context;
};

const useLocalStorage = <T>(key: string, initialValue: T) => {
  // Implementation with error handling and type safety
};
```

---

### ⚡ Agent 7: Performance & Testing (PT)
**Status**: ✅ READY TO START  
**Role**: Optimization, testing, and quality assurance  
**Current Task**: Set up performance monitoring and comprehensive test suites  

**Responsibilities**:
- Implement performance optimizations and monitoring
- Create comprehensive test suites
- Monitor bundle size and loading performance
- Implement accessibility testing
- Handle browser compatibility and mobile optimization

**Performance Targets**:
- Lighthouse Performance: >90
- First Contentful Paint: <2.5s
- Cumulative Layout Shift: <0.1
- Bundle size: <250KB gzipped
- 60fps smooth animations
- Core Web Vitals compliance

**Testing Strategy**:
```typescript
// Unit Tests (Jest + React Testing Library)
describe('ContentCard', () => {
  test('displays movie information correctly', () => {
    render(<ContentCard item={mockMovie} />);
    expect(screen.getByText(mockMovie.title)).toBeInTheDocument();
  });
});

// Accessibility Testing
import { axe, toHaveNoViolations } from 'jest-axe';
expect.extend(toHaveNoViolations);

test('should not have accessibility violations', async () => {
  const { container } = render(<App />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});

// Performance Monitoring
import { getCLS, getFID, getFCP, getLCP, getTTFB } from 'web-vitals';

const vitals = {
  CLS: getCLS(console.log),
  FID: getFID(console.log),
  FCP: getFCP(console.log),
  LCP: getLCP(console.log),
  TTFB: getTTFB(console.log),
};
```

---

### 🚀 Agent 8: DevOps & Deployment (DD)
**Status**: ✅ READY TO START  
**Role**: Build optimization, deployment configuration, and production setup  
**Current Task**: Configure Docker containers and CI/CD pipelines  

**Responsibilities**:
- Configure build pipelines and optimization
- Create Docker configurations for deployment
- Set up CI/CD workflows
- Handle environment management
- Implement monitoring and logging solutions

**Docker Configuration**:
```dockerfile
# Frontend Dockerfile
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80

# Backend Dockerfile
FROM rust:1.70-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY src ./src
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/crmb-backend ./
EXPOSE 3001
CMD ["./crmb-backend"]
```

**Environment Configuration**:
```bash
# Frontend (.env)
REACT_APP_TMDB_API_KEY=your_tmdb_api_key_here
REACT_APP_API_URL=http://localhost:3001/api
REACT_APP_WS_URL=ws://localhost:3001/ws

# Backend (.env)
RUST_LOG=info
TMDB_API_KEY=your_tmdb_api_key_here
DATABASE_URL=sqlite:./data/crmb.db
CORS_ORIGIN=http://localhost:3000
PORT=3001
```

---

## Agent Interaction Protocol

### Phase 1: Foundation (Agents PA, DS)
1. **Project Architect (PA)** creates project structure and TypeScript interfaces
2. **Design System (DS)** implements CSS variables and core styling system

### Phase 2: Core Development (Agents FC, BA)
3. **Frontend Core (FC)** builds React components using DS styling
4. **Backend API (BA)** creates Rust server with database setup

### Phase 3: Integration (Agents AI, SM)
5. **API Integration (AI)** implements TMDB, MDBList, and Stremio services
6. **State Management (SM)** creates Context providers and state logic

### Phase 4: Quality & Deployment (Agents PT, DD)
7. **Performance & Testing (PT)** optimizes and tests all components
8. **DevOps & Deployment (DD)** packages for production deployment

### Communication Flow:
- **PA** coordinates all agents and resolves conflicts
- **DS** provides styling requirements to **FC**
- **AI** provides data patterns to **FC** and **BA**
- **SM** provides state patterns to **FC**
- **BA** provides API contracts to **AI**
- **PT** reviews all outputs for performance compliance
- **DD** integrates all components for deployment

### Quality Gates:
1. **Architecture Review**: PA approves all architectural decisions
2. **Code Review**: Each agent validates integration points
3. **Performance Check**: PT validates performance requirements
4. **Security Review**: BA and DD ensure security compliance
5. **Accessibility Audit**: DS and PT ensure WCAG 2.1 AA compliance

---

## 🎯 SUCCESS CRITERIA

### Technical Excellence
- ✅ TypeScript strict mode with comprehensive error handling
- ✅ 60fps animations with proper easing functions
- ✅ Sub-2s load times with efficient API usage
- ✅ Graceful degradation and offline capabilities
- ✅ Input validation and secure authentication
- ✅ Efficient caching and lazy loading

### User Experience
- ✅ Pixel-perfect implementation matching premium platforms
- ✅ Smooth scrolling and intuitive navigation
- ✅ Instant interactions with responsive feedback
- ✅ WCAG 2.1 AA compliance with keyboard navigation
- ✅ Screen reader support and accessibility features

### Functional Requirements
- ✅ Seamless TMDB API integration with error handling
- ✅ Real-time search with debouncing and optimization
- ✅ Dynamic hero banners and smooth carousels
- ✅ Full Stremio addon compatibility
- ✅ User preferences, watchlist, and progress tracking

---

## 🚀 TEAM STATUS: READY TO START

**All 8 agents are configured and ready to begin development on the CRMB Streaming WebApp project.**

**Next Steps**:
1. Initialize project structure (PA)
2. Set up development environment
3. Begin Phase 1 development
4. Coordinate agent handoffs
5. Monitor progress and quality gates

**Estimated Timeline**: 4-6 weeks for full implementation
**Target Delivery**: Production-ready streaming web application

---

*This automated team configuration ensures comprehensive coverage of all aspects needed to build a production-ready streaming web application with proper separation of concerns and quality assurance at every level.*