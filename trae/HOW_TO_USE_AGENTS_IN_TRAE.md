# How to Use CRMB Streaming WebApp Agents in Trae AI

This guide explains how to properly use the 8 specialized AI agents created for the CRMB Streaming WebApp project in Trae AI.

## Overview

The CRMB Streaming WebApp agent team consists of 8 specialized agents, each designed to handle specific aspects of building a premium streaming media center application:

1. **Project Architect Agent** - Overall system architecture and coordination
2. **Frontend Core Agent** - React/TypeScript development
3. **Design System Agent** - Visual design and CSS architecture
4. **API Integration Agent** - External API integration, TMDB client, Stremio addon protocol
5. **Backend API Agent** - Rust/Axum backend server development
6. **State Management Agent** - React state architecture
7. **Performance & Testing Agent** - Performance optimization, testing, and quality assurance
8. **DevOps & Deployment Agent** - Infrastructure and deployment

## Setting Up Agents in Trae AI

### Step 1: Import Agent Configurations

1. Open Trae AI
2. Navigate to the Agent Management section
3. Import each agent configuration file:
   - `01_project_architect_agent.json`
   - `02_frontend_core_agent.json`
   - `03_design_system_agent.json`
   - `04_api_integration_agent.json`
   - `05_backend_api_agent.json`
   - `06_state_management_agent.json`
   - `07_performance_testing_agent.json`
   - `08_devops_deployment_agent.json`

### Step 2: Configure Agent Context

Ensure each agent has access to the following reference files:
- `complete_trae_ai_config.json` (project specifications)
- Frontend context files (React/TypeScript documentation)
- Backend context files (Rust/Axum documentation)
- Any existing codebase files

## Agent Usage Patterns

### 1. Project Initialization (Start Here)

**Use: Project Architect Agent**

```
Prompt: "Initialize the CRMB Streaming WebApp project structure. Create the basic folder structure for both frontend (React/TypeScript) and backend (Rust/Axum) following the specifications in complete_trae_ai_config.json."
```

**Expected Output:**
- Project folder structure
- Package.json and Cargo.toml files
- Basic configuration files
- Environment setup instructions

### 2. API Integration Development

**Use: API Integration Agent**

```
Prompt: "Implement external API integrations for the CRMB Streaming WebApp. Create TMDB API client, MDBList rating aggregation, Stremio addon protocol implementation, and comprehensive caching strategies."
```

**Expected Output:**
- TMDB API client with rate limiting
- MDBList rating aggregation service
- Stremio addon protocol implementation
- API error handling and retry logic
- Caching strategies for external data

### 3. Backend Server Development

**Use: Backend API Agent**

```
Prompt: "Implement the Rust backend server with Axum framework. Create internal API endpoints, database models, authentication middleware, and server infrastructure as specified in the project requirements."
```

**Expected Output:**
- Complete Rust backend server
- Internal API endpoints
- Database models and migrations
- Middleware for CORS, rate limiting, and auth
- JWT authentication system

### 4. Frontend Core Development

**Use: Frontend Core Agent**

```
Prompt: "Create the React/TypeScript frontend application. Implement the main layout with sidebar navigation, hero banner component, and movie carousels. Integrate with the backend API for TMDB data."
```

**Expected Output:**
- React components with TypeScript
- API service integration
- Responsive layout implementation
- Performance optimizations

### 5. Visual Design Implementation

**Use: Design System Agent**

```
Prompt: "Implement the Apple TV+/Netflix-inspired design system. Create the dark theme with lime green accents, design tokens, and CSS architecture following the visual specifications."
```

**Expected Output:**
- CSS design system with custom properties
- Component styling with BEM methodology
- Dark theme implementation
- Animation and transition systems

### 6. State Management Setup

**Use: State Management Agent**

```
Prompt: "Implement the React state management architecture using Context API and custom hooks. Create providers for app state, content state, and UI state with optimized re-rendering."
```

**Expected Output:**
- React Context providers
- Custom hooks for data fetching
- State management patterns
- Performance optimizations

### 7. Testing, Performance & Quality Assurance

**Use: Performance & Testing Agent**

```
Prompt: "Set up comprehensive testing framework, performance monitoring, and quality assurance processes. Implement unit tests, integration tests, E2E tests, performance optimization strategies, and quality gates."
```

**Expected Output:**
- Testing framework setup
- Performance monitoring configuration
- Accessibility testing and WCAG 2.1 AA compliance
- Bundle optimization
- Quality gates and release validation
- Cross-browser compatibility testing

### 8. Infrastructure and Deployment

**Use: DevOps & Deployment Agent**

```
Prompt: "Create Docker configurations and CI/CD pipelines for the CRMB Streaming WebApp. Set up development and production environments with monitoring and security."
```

**Expected Output:**
- Docker configurations
- GitHub Actions workflows
- Infrastructure as Code
- Monitoring and alerting setup

## Multi-Agent Collaboration

### Sequential Development Workflow

1. **Project Architect** → Initialize project structure
2. **API Integration** → Implement external API integrations (TMDB, MDBList, Stremio)
3. **Backend API** → Implement Rust server and internal APIs
4. **Design System** → Create visual design foundation
5. **Frontend Core** → Build React components and pages
6. **State Management** → Implement state architecture
7. **Performance & Testing** → Add testing, optimization, and quality assurance
8. **DevOps & Deployment** → Set up infrastructure and deployment

### Parallel Development (Advanced)

For experienced teams, some agents can work in parallel:

**Phase 1 (Parallel):**
- Project Architect + Design System
- API Integration (external APIs)
- Backend API (internal server development)

**Phase 2 (Parallel):**
- Frontend Core + State Management
- Performance & Testing (test setup)

**Phase 3 (Sequential):**
- Performance & Testing (optimization and quality assurance)
- DevOps & Deployment (final deployment)

## Best Practices

### 1. Context Management
- Always provide the `complete_trae_ai_config.json` file to agents
- Share relevant code files between agents
- Maintain consistent project context across sessions

### 2. Agent Communication
- Use clear, specific prompts referencing project requirements
- Include relevant file paths and component names
- Reference other agents' work when building upon it

### 3. Iterative Development
- Start with basic implementations
- Iterate and refine with the same agent
- Use different agents for different concerns

### 4. Quality Control
- Use Performance & Testing Agent for optimization and quality assurance
- Validate with Project Architect for architectural decisions
- Ensure API Integration Agent handles external service reliability

## Example Multi-Agent Session

```
1. Project Architect: "Initialize CRMB project structure"
   → Creates folder structure, package files

2. API Integration: "Implement TMDB, MDBList, and Stremio integrations"
   → Creates external API clients and caching strategies

3. Backend API: "Implement Rust backend server with internal APIs"
   → Creates complete backend server infrastructure

4. Design System: "Create Apple TV+ inspired design system"
   → Implements CSS architecture and design tokens

5. Frontend Core: "Build React frontend with hero banner and carousels"
   → Creates React components and pages

6. State Management: "Add React Context and custom hooks"
   → Implements state management layer

7. Performance & Testing: "Add testing, optimization, and quality assurance"
   → Sets up testing framework, optimizations, and quality gates

8. DevOps & Deployment: "Create Docker and CI/CD setup"
   → Implements deployment infrastructure
```

## Troubleshooting

### Common Issues

1. **Agent Context Loss**
   - Re-upload `complete_trae_ai_config.json`
   - Provide relevant code files
   - Reference previous agent outputs

2. **Inconsistent Implementation**
   - Use Project Architect for architectural decisions
   - Cross-reference with Design System for visual consistency
   - Validate with Performance & Testing Agent

3. **Integration Problems**
   - Use specific agents for their expertise areas
   - Use API Integration Agent for external service issues
   - Provide clear integration requirements
   - Test with Performance & Testing Agent

### Getting Help

- Each agent configuration includes detailed specifications
- Reference the `complete_trae_ai_config.json` for project requirements
- Use the Project Architect Agent for high-level guidance
- Consult Performance & Testing Agent for validation and quality assurance

## Conclusion

The CRMB Streaming WebApp agent team provides a comprehensive, professional approach to building a premium streaming media center. By using each agent for their specialized expertise and following the recommended workflows, you can create a high-quality application that rivals Netflix, Apple TV+, and Disney+ in both functionality and user experience.

Remember to:
- Start with Project Architect for structure
- Use agents sequentially for best results
- Maintain consistent context across sessions
- Validate with Quality Assurance before deployment
- Leverage each agent's specialized expertise

With proper usage, these agents will help you build a professional-grade streaming platform efficiently and effectively.