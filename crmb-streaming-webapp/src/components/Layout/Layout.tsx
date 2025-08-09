import { Outlet } from 'react-router-dom'
import Sidebar from '../common/Sidebar/Sidebar'
import SearchBar from '../common/SearchBar/SearchBar'
import './Layout.css'

export const Layout = () => {
  return (
    <div className="app">
      <Sidebar />
      <div className="main-content">
        <header className="header">
          <div className="container">
            <SearchBar />
          </div>
        </header>
        <main className="main-content__body">
          <Outlet />
        </main>
      </div>
    </div>
  )
}

export default Layout