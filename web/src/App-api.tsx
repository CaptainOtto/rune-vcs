import { useState, useEffect } from 'react';
import { TopBar } from './components/TopBar';
import { Sidebar } from './components/Sidebar';
import { ChangesView } from './components/ChangesView';
import { HistoryView } from './components/HistoryView';
import { BranchesView } from './components/BranchesView';
import { StashView } from './components/StashView';
import { SettingsView } from './components/SettingsView';
import { DetailPanel } from './components/DetailPanel';
import { RepositoryTree } from './components/RepositoryTree';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from './components/ui/resizable';
import { api, Repository } from './services/api';

type ViewType = 'changes' | 'history' | 'branches' | 'stash' | 'settings';

function App() {
  const [activeView, setActiveView] = useState<ViewType>('changes');
  const [isPowerMode, setIsPowerMode] = useState(false);
  const [repository, setRepository] = useState<Repository | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadRepository = async () => {
      try {
        const repo = await api.getRepository();
        setRepository(repo);
      } catch (error) {
        console.error('Failed to load repository:', error);
        // Set a fallback repository for development
        setRepository({
          name: 'rune-vcs',
          branch: 'main',
          url: 'local',
          lastSync: undefined
        });
      } finally {
        setLoading(false);
      }
    };

    loadRepository();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading repository...</p>
        </div>
      </div>
    );
  }

  const renderActiveView = () => {
    switch (activeView) {
      case 'changes':
        return <ChangesView isPowerMode={isPowerMode} />;
      case 'history':
        return <HistoryView isPowerMode={isPowerMode} />;
      case 'branches':
        return <BranchesView isPowerMode={isPowerMode} />;
      case 'stash':
        return <StashView isPowerMode={isPowerMode} />;
      case 'settings':
        return <SettingsView isPowerMode={isPowerMode} setIsPowerMode={setIsPowerMode} />;
      default:
        return <ChangesView isPowerMode={isPowerMode} />;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <TopBar 
        repository={repository!} 
        isPowerMode={isPowerMode} 
        setIsPowerMode={setIsPowerMode} 
      />
      
      <ResizablePanelGroup direction="horizontal" className="flex-1">
        <ResizablePanel defaultSize={20} minSize={15} maxSize={30}>
          <div className="h-full border-r border-gray-200 bg-white">
            <Sidebar activeView={activeView} setActiveView={setActiveView} />
            <RepositoryTree />
          </div>
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel defaultSize={50} minSize={30}>
          <div className="h-full bg-white">
            {renderActiveView()}
          </div>
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel defaultSize={30} minSize={25} maxSize={40}>
          <div className="h-full border-l border-gray-200 bg-white">
            <DetailPanel />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
