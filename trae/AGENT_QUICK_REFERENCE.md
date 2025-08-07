# CRMB Streaming WebApp - Agent Quick Reference

Quick reference guide for the 8 specialized AI agents in the CRMB Streaming WebApp project.

## 🏗️ 01. Project Architect Agent
**Role:** System architecture and project coordination  
**Use When:** Starting project, making architectural decisions, coordinating between components  
**Specializes In:**
- Project structure and organization
- Technology stack decisions
- Component integration planning
- Development workflow coordination
- Technical specifications validation

**Example Prompt:**
```
"Initialize the CRMB Streaming WebApp project structure with React/TypeScript frontend and Rust/Axum backend."
```

---

## ⚛️ 02. Frontend Core Agent
**Role:** React/TypeScript development  
**Use When:** Building React components, implementing frontend logic, API integration  
**Specializes In:**
- React 18+ with TypeScript
- Component architecture and hooks
- API service integration
- Performance optimization
- Responsive design implementation

**Example Prompt:**
```
"Create the main layout with sidebar navigation, hero banner, and movie carousels using React/TypeScript."
```

---

## 🎨 03. Design System Agent
**Role:** Visual design and CSS architecture  
**Use When:** Implementing UI design, creating design tokens, styling components  
**Specializes In:**
- Apple TV+/Netflix-inspired aesthetics
- Dark theme with lime green accents
- CSS custom properties and design tokens
- Animation and transition systems
- Responsive design patterns

**Example Prompt:**
```
"Implement the dark theme design system with Apple TV+ aesthetics and lime green accent colors."
```

---

## 🔌 04. API Integration Agent
**Role:** External API integration and data aggregation  
**Use When:** Integrating external services, handling API clients, implementing protocols  
**Specializes In:**
- TMDB API client with rate limiting
- MDBList rating aggregation service
- Stremio addon protocol implementation
- API error handling and retry logic
- Caching strategies for external data

**Example Prompt:**
```
"Implement TMDB API client, MDBList integration, and Stremio addon protocol with comprehensive caching."
```

---

## 🦀 05. Backend API Agent
**Role:** Rust/Axum backend server development  
**Use When:** Building internal server logic, API endpoints, database integration  
**Specializes In:**
- Rust with Axum/Actix-web framework
- Internal API endpoints and routing
- JWT authentication and middleware
- Database models and operations
- Server infrastructure and security

**Example Prompt:**
```
"Implement the Rust backend server with internal APIs, authentication, and database integration."
```

---

## 🔄 06. State Management Agent
**Role:** React state architecture  
**Use When:** Managing application state, data flow, client-side caching  
**Specializes In:**
- React Context API with useReducer
- Custom hooks for data fetching
- Optimistic updates and error handling
- Performance optimization for re-renders
- Client-side caching strategies

**Example Prompt:**
```
"Create React Context providers and custom hooks for managing movie data and user preferences."
```

---

## ⚡ 07. Performance & Testing Agent
**Role:** Performance optimization, testing, and quality assurance  
**Use When:** Optimizing performance, setting up tests, ensuring quality, final validation  
**Specializes In:**
- Lighthouse score optimization (90+)
- Core Web Vitals improvement
- Bundle size optimization
- Comprehensive testing (unit, integration, E2E)
- Accessibility compliance (WCAG 2.1 AA)
- Quality gates and release validation
- Cross-browser compatibility testing

**Example Prompt:**
```
"Set up performance monitoring, comprehensive testing framework, and quality assurance processes for the streaming application."
```

---

## 🚀 08. DevOps & Deployment Agent
**Role:** Infrastructure and deployment automation  
**Use When:** Setting up CI/CD, containerization, production deployment  
**Specializes In:**
- Docker containerization
- GitHub Actions CI/CD pipelines
- Infrastructure as Code (Terraform)
- Monitoring and observability
- Security and compliance

**Example Prompt:**
```
"Create Docker configurations and CI/CD pipelines for development and production environments."
```

---

## 🔄 Recommended Usage Flow

### Phase 1: Foundation
1. **Project Architect** → Project structure
2. **API Integration** → External API setup
3. **Backend API** → Server implementation
4. **Design System** → Visual foundation

### Phase 2: Core Development
5. **Frontend Core** → React components
6. **State Management** → Data flow

### Phase 3: Optimization & Deployment
7. **Performance & Testing** → Optimization and quality assurance
8. **DevOps & Deployment** → Infrastructure and deployment

---

## 🎯 Quick Decision Guide

**Need to start the project?** → Project Architect  
**External API integration?** → API Integration  
**Building React components?** → Frontend Core  
**Styling and design?** → Design System  
**Internal server and APIs?** → Backend API  
**Managing app state?** → State Management  
**Performance, testing, or quality issues?** → Performance & Testing  
**Deployment setup?** → DevOps & Deployment  

---

## 📋 Agent Configuration Files

- `01_project_architect_agent.json`
- `02_frontend_core_agent.json`
- `03_design_system_agent.json`
- `04_api_integration_agent.json`
- `05_backend_api_agent.json`
- `06_state_management_agent.json`
- `07_performance_testing_agent.json`
- `08_devops_deployment_agent.json`

## 📖 Additional Resources

- `complete_trae_ai_config.json` - Complete project specifications
- `HOW_TO_USE_AGENTS_IN_TRAE.md` - Detailed usage guide
- Custom instructions in each agent configuration

---

**Remember:** Each agent is specialized for their domain. Use the right agent for the right task to get the best results!