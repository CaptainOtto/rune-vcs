import { useState, useEffect } from 'react';
import { TopBar } from './components/TopBar';
import { Sidebar } from './components/Sidebar';
import { ChangesView } from './components/ChangesView';
import { HistoryView } from './components/HistoryView';
import { BranchesView } from './components/BranchesView';
import { StashView } from './components/StashView';
import { SettingsView } from './components/SettingsView';

type ViewType = 'changes' | 'history' | 'branches' | 'stash' | 'settings';

interface Repository {
  id: string;
  name: string;
  path: string;
  isActive: boolean;
}

export default function App() {
  const [activeView, setActiveView] = useState<ViewType>('history');
  const [isPowerMode, setIsPowerMode] = useState(false); // Start minimal, allow users to enable power mode
  const [repositories, setRepositories] = useState<Repository[]>([
    { id: '1', name: 'rune-desktop', path: '/Projects/rune-desktop', isActive: true },
    { id: '2', name: 'portfolio-site', path: '/Projects/portfolio-site', isActive: false },
  ]);
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('darkMode');
      if (saved !== null) {
        return JSON.parse(saved);
      }
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return false;
  });
  const [currentBranch, setCurrentBranch] = useState('main');
  const [searchQuery, setSearchQuery] = useState('');
  const [syncStatus, setSyncStatus] = useState<'synced' | 'ahead' | 'behind' | 'syncing'>('synced');

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('darkMode', JSON.stringify(isDark));
  }, [isDark]);

  const toggleTheme = () => {
    setIsDark(!isDark);
  };

  const togglePowerMode = () => {
    setIsPowerMode(!isPowerMode);
  };

  const activeRepository = repositories.find(repo => repo.isActive);

  return (
    <div className="min-h-screen bg-kodex-bg flex flex-col overflow-hidden">
      {/* Subtle Aurora Background Effect */}
      <div className="fixed inset-0 kodex-aurora-bg pointer-events-none opacity-50" />
      
      {/* Top Bar */}
      <TopBar 
        currentBranch={currentBranch} 
        setCurrentBranch={setCurrentBranch}
        isDark={isDark}
        toggleTheme={toggleTheme}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        syncStatus={syncStatus}
        repositories={repositories}
        setRepositories={setRepositories}
        activeRepository={activeRepository}
        isPowerMode={isPowerMode}
      />
      
      {/* Main Content Area */}
      <div className="flex flex-1 h-[calc(100vh-60px)] relative z-10">
        {/* Sidebar */}
        <Sidebar 
          activeView={activeView} 
          setActiveView={setActiveView}
          isPowerMode={isPowerMode}
          togglePowerMode={togglePowerMode}
        />
        
        {/* Main Content */}
        <div className="flex-1 bg-kodex-content">
          {activeView === 'changes' && <ChangesView isPowerMode={isPowerMode} />}
          {activeView === 'history' && <HistoryView isPowerMode={isPowerMode} searchQuery={searchQuery} />}
          {activeView === 'branches' && <BranchesView currentBranch={currentBranch} setCurrentBranch={setCurrentBranch} isPowerMode={isPowerMode} />}
          {activeView === 'stash' && <StashView />}
          {activeView === 'settings' && <SettingsView isDark={isDark} toggleTheme={toggleTheme} />}
        </div>
      </div>
    </div>
  );
}