import { Outlet } from 'react-router-dom'
import { Header } from './Header'
import { Navigation } from './Navigation'
import { Footer } from './Footer'
import './Layout.css'

export const Layout = () => {
  return (
    <div className="layout">
      <Header />
      <div className="layout__content">
        <Navigation />
        <main className="layout__main">
          <Outlet />
        </main>
      </div>
      <Footer />
    </div>
  )
}