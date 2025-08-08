# CRMB Streaming WebApp

A premium streaming platform built with React, TypeScript, and modern web technologies. Features Apple TV+/Netflix-level aesthetics with TMDB API integration and Stremio addon support.

## 🏗️ Architecture Overview

### Frontend Stack
- **React 18** with TypeScript
- **Vite** for build tooling and development
- **React Router DOM** for client-side routing
- **CSS Variables** for design system
- **ESLint + Prettier** for code quality

### Backend Integration
- **TMDB API** for movie/TV show data
- **Stremio Addon Protocol** for streaming sources
- **MDBList** for rating aggregation
- **Rust/Axum Backend** (separate repository)

### Design System
- Dark theme with Apple TV+ aesthetics
- Responsive design (mobile-first)
- CSS custom properties for theming
- Modern glassmorphism effects

## 📁 Project Structure

```
src/
├── components/          # Reusable UI components
│   └── Layout/         # Layout components (Header, Navigation, Footer)
├── pages/              # Page components
│   ├── Home/
│   ├── Search/
│   ├── Watchlist/
│   └── Settings/
├── styles/             # Global styles and design system
├── types/              # TypeScript type definitions
├── utils/              # Utility functions
├── hooks/              # Custom React hooks
├── services/           # API services
└── store/              # State management
```

## 🚀 Getting Started

### Prerequisites
- Node.js 18+
- npm or yarn
- TMDB API key

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd crmb-streaming-webapp
```

2. Install dependencies:
```bash
npm install
```

3. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your API keys
```

4. Start development server:
```bash
npm run dev
```

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run lint:fix` - Fix ESLint issues
- `npm run format` - Format code with Prettier
- `npm run type-check` - Run TypeScript type checking
- `npm run test` - Run tests

## 🎨 Design System

### Color Palette
- **Primary**: Electric blue (#007AFF)
- **Background**: Deep black (#000000)
- **Surface**: Dark gray (#1C1C1E)
- **Text**: White/gray hierarchy

### Typography
- **Primary Font**: System fonts (SF Pro, Segoe UI, Roboto)
- **Scale**: Modular scale with consistent spacing
- **Weights**: Regular (400), Medium (500), Semibold (600), Bold (700)

### Spacing
- **Base Unit**: 4px
- **Scale**: 4px, 8px, 12px, 16px, 24px, 32px, 48px, 64px

## 🔧 Configuration

### Environment Variables

```env
# TMDB API
VITE_TMDB_API_KEY=your_api_key
VITE_TMDB_BASE_URL=https://api.themoviedb.org/3
VITE_TMDB_IMAGE_BASE_URL=https://image.tmdb.org/t/p/

# Backend API
VITE_API_URL=http://localhost:3001/api

# Feature Flags
VITE_ENABLE_ANALYTICS=false
VITE_ENABLE_PWA=true
VITE_ENABLE_OFFLINE=true
```

### TypeScript Configuration

- Strict mode enabled
- Path aliases configured (`@/` for `src/`)
- React JSX transform
- ES2020 target

### Build Configuration

- Vite with React plugin
- Code splitting and optimization
- Source maps for development
- Proxy for API calls

## 🧪 Testing Strategy

### Testing Stack
- **Vitest** for unit testing
- **React Testing Library** for component testing
- **MSW** for API mocking
- **Playwright** for E2E testing (planned)

### Performance Targets
- **Lighthouse Score**: 90+
- **First Contentful Paint**: <1.5s
- **Largest Contentful Paint**: <2.5s
- **Cumulative Layout Shift**: <0.1

## 🔄 Development Workflow

### 8-Agent Development System

1. **Project Architect (PA-001)** - System design and coordination
2. **Frontend Core (FC-002)** - React components and routing
3. **Design System (DS-003)** - UI components and styling
4. **API Integration (AI-004)** - External API services
5. **Backend API (BA-005)** - Rust/Axum backend
6. **State Management (SM-006)** - Application state
7. **Performance & Testing (PT-007)** - Optimization and testing
8. **DevOps & Deployment (DD-008)** - CI/CD and deployment

### Code Quality

- **ESLint**: Strict TypeScript rules
- **Prettier**: Consistent formatting
- **Husky**: Pre-commit hooks
- **Conventional Commits**: Standardized commit messages

### Git Workflow

- **Main Branch**: Production-ready code
- **Develop Branch**: Integration branch
- **Feature Branches**: Individual features
- **Release Branches**: Release preparation

## 📱 Responsive Design

### Breakpoints
- **Mobile**: 320px - 768px
- **Tablet**: 769px - 1024px
- **Desktop**: 1025px - 1440px
- **Large Desktop**: 1441px+

### Layout Strategy
- Mobile-first approach
- Flexible grid system
- Touch-friendly interactions
- Progressive enhancement

## 🔒 Security

### Best Practices
- Environment variables for sensitive data
- Content Security Policy headers
- HTTPS enforcement
- Input validation and sanitization
- Secure API communication

## 📈 Performance Optimization

### Strategies
- Code splitting and lazy loading
- Image optimization (WebP/AVIF)
- Service worker for caching
- Bundle size monitoring
- Critical CSS inlining

## 🚀 Deployment

### Production Build
```bash
npm run build
npm run preview
```

### Deployment Targets
- **Vercel** (recommended)
- **Netlify**
- **AWS S3 + CloudFront**
- **Docker containers**

## 📚 API Documentation

### TMDB Integration
- Movie and TV show data
- Image assets and metadata
- Search functionality
- Trending content

### Stremio Protocol
- Addon manifest
- Catalog endpoints
- Stream resolution
- Metadata enrichment

## 🤝 Contributing

### Development Setup
1. Follow installation instructions
2. Create feature branch
3. Make changes with tests
4. Submit pull request

### Code Standards
- TypeScript strict mode
- Component composition patterns
- Accessibility compliance
- Performance considerations

## 📄 License

MIT License - see LICENSE file for details.

## 🔗 Related Projects

- **CRMB Backend**: Rust/Axum API server
- **CRMB Mobile**: React Native mobile app
- **CRMB Desktop**: Electron desktop app

---

**Built with ❤️ by the CRMB Development Team**