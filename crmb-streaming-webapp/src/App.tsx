import Sidebar from './components/common/Sidebar/Sidebar.tsx'
import SearchBar from './components/common/SearchBar/SearchBar.tsx'
import HeroBanner from './components/hero/HeroBanner/HeroBanner.tsx'
import ContentSection from './components/carousel/ContentSection/ContentSection.tsx'
import './styles/global.css'

function App() {
  return (
    <div className="app">
      <Sidebar />
      <div className="main-content">
        <header className="header">
          <div className="container">
            <SearchBar />
          </div>
        </header>
        
        <HeroBanner />
        
        <ContentSection 
          title="Up Next" 
          type="episodes"
          showSeeAll={true}
        />
        
        <ContentSection 
          title="Movies - Popular" 
          type="movies"
          showSeeAll={true}
        />
      </div>
    </div>
  )
}

export default App