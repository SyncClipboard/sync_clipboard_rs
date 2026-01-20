import { useState } from 'react';
import {
  ClipboardList,
  Settings,
  Server,
  Shield,
  Database,
  Wifi,
  Menu,
  Info
} from 'lucide-react';
import History from './components/History';
import SettingsComponent from './components/Settings';
import Network from './components/Network';
import About from './components/About';
import { cn } from './lib/utils';
import { Button } from './components/ui/button';

import { useTranslation } from 'react-i18next';

type Tab = 'history' | 'network' | 'settings_general' | 'settings_server' | 'settings_security' | 'settings_local' | 'about';

function App() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<Tab>('history');
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

  // Helper helper to generate sidebar button classes
  const getSidebarItemClass = (isActive: boolean, depth = 0) => cn(
    "w-full flex items-center justify-start gap-3 px-4 py-2 text-sm font-medium transition-colors rounded-md",
    isActive
      ? "bg-accent text-accent-foreground"
      : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground",
    depth > 0 && "pl-9"
  );

  return (
    <div className="flex h-screen bg-background text-foreground font-sans overflow-hidden selection:bg-primary/20">
      {/* Sidebar */}
      <aside
        className={cn(
          "border-r bg-card transition-all duration-300 ease-in-out flex flex-col z-20 shadow-sm",
          isSidebarOpen ? "w-64" : "w-16 items-center"
        )}
      >
        {/* Header */}
        <div className="h-16 flex items-center justify-between px-4 shrink-0 border-b">
          {isSidebarOpen && (
            <div className="flex items-center gap-2 animate-in fade-in duration-300">
              <div className="w-8 h-8 bg-primary rounded-lg flex items-center justify-center text-primary-foreground shadow-sm">
                <ClipboardList size={18} />
              </div>
              <h1 className="font-bold text-lg tracking-tight">SyncClip</h1>
            </div>
          )}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="ml-auto"
          >
            <Menu size={18} />
          </Button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 py-4 px-2 space-y-1 overflow-y-auto">
          <button
            onClick={() => setActiveTab('history')}
            className={getSidebarItemClass(activeTab === 'history')}
          >
            <ClipboardList size={18} />
            {isSidebarOpen && <span>{t('nav.history')}</span>}
          </button>

          <button
            onClick={() => setActiveTab('network')}
            className={getSidebarItemClass(activeTab === 'network')}
          >
            <Wifi size={18} />
            {isSidebarOpen && <span>{t('nav.network', 'Network')}</span>}
          </button>

          <div className="mt-6"></div>
          <button
            onClick={() => setActiveTab('settings_general')}
            className={getSidebarItemClass(activeTab === 'settings_general')}
          >
            <Settings size={18} />
            {isSidebarOpen && <span>{t('nav.general')}</span>}
          </button>
          <button
            onClick={() => setActiveTab('settings_server')}
            className={getSidebarItemClass(activeTab === 'settings_server')}
          >
            <Server size={18} />
            {isSidebarOpen && <span>{t('nav.connection')}</span>}
          </button>
          <button
            onClick={() => setActiveTab('settings_security')}
            className={getSidebarItemClass(activeTab === 'settings_security')}
          >
            <Shield size={18} />
            {isSidebarOpen && <span>{t('nav.security')}</span>}
          </button>
          <button
            onClick={() => setActiveTab('settings_local')}
            className={getSidebarItemClass(activeTab === 'settings_local')}
          >
            <Database size={18} />
            {isSidebarOpen && <span>{t('nav.storage')}</span>}
          </button>

          <div className="mt-4"></div>
          <button
            onClick={() => setActiveTab('about')}
            className={getSidebarItemClass(activeTab === 'about')}
          >
            <Info size={18} />
            {isSidebarOpen && <span>{t('nav.about', '关于')}</span>}
          </button>
        </nav>

        {/* Footer */}
        <div className="p-4 border-t bg-muted/20">
          {isSidebarOpen ? (
            <div className="flex items-center gap-3 animate-in fade-in">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-xs">
                SC
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium truncate">SyncClipboard</p>
                <p className="text-xs text-muted-foreground truncate">v0.1.0-alpha</p>
              </div>
            </div>
          ) : (
            <div className="w-8 h-8 mx-auto rounded-full bg-muted flex items-center justify-center text-xs text-muted-foreground">
              v0.1
            </div>
          )}
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col h-full overflow-hidden bg-background relative">
        {/* Header */}
        <header className="h-16 flex items-center px-8 shrink-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b z-10 sticky top-0">
          <div>
            <h2 className="text-lg font-semibold tracking-tight">
              {activeTab === 'history' && t('nav.history')}
              {activeTab === 'network' && t('nav.network', 'Network')}
              {activeTab === 'settings_general' && t('nav.general')}
              {activeTab === 'settings_server' && t('nav.connection')}
              {activeTab === 'settings_security' && t('nav.security')}
              {activeTab === 'settings_local' && t('nav.storage')}
              {activeTab === 'about' && t('nav.about', '关于')}
            </h2>
            <p className="text-sm text-muted-foreground">
              {activeTab === 'history' && t('desc.history')}
              {activeTab === 'network' && t('desc.network', 'View network information and discover devices')}
              {activeTab === 'settings_general' && t('desc.general')}
              {activeTab === 'settings_server' && t('desc.connection')}
              {activeTab === 'settings_security' && t('desc.security')}
              {activeTab === 'settings_local' && t('desc.storage')}
              {activeTab === 'about' && t('desc.about', '查看应用版本信息和依赖库')}
            </p>
          </div>
        </header>

        {/* Scrollable Content */}
        <div className="flex-1 overflow-y-auto p-8 scrollbar-hide">
          <div className="max-w-5xl mx-auto space-y-6">
            {activeTab === 'history' ? (
              <History />
            ) : activeTab === 'network' ? (
              <Network />
            ) : activeTab === 'about' ? (
              <About />
            ) : (
              <SettingsComponent activeSection={activeTab.replace('settings_', '')} />
            )}
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
