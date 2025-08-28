import { useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Badge } from './ui/badge';
import { Tabs, TabsList, TabsTrigger } from './ui/tabs';
import { 
  GitBranch, 
  Upload, 
  Download, 
  Search, 
  Sun, 
  Moon, 
  Zap,
  CheckCircle2,
  ArrowUp,
  ArrowDown,
  RotateCcw,
  Plus,
  X
} from 'lucide-react';

interface Repository {
  id: string;
  name: string;
  path: string;
  isActive: boolean;
}

interface TopBarProps {
  currentBranch: string;
  setCurrentBranch: (branch: string) => void;
  isDark: boolean;
  toggleTheme: () => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  syncStatus: 'synced' | 'ahead' | 'behind' | 'syncing';
  repositories: Repository[];
  setRepositories: (repos: Repository[]) => void;
  activeRepository?: Repository;
  isPowerMode: boolean;
}

export function TopBar({ 
  currentBranch, 
  setCurrentBranch, 
  isDark, 
  toggleTheme,
  searchQuery,
  setSearchQuery,
  syncStatus,
  repositories,
  setRepositories,
  activeRepository,
  isPowerMode
}: TopBarProps) {
  const branches = ['main', 'develop', 'feature/auth', 'hotfix/security'];
  const [isSearchFocused, setIsSearchFocused] = useState(false);

  const getSyncIcon = () => {
    switch (syncStatus) {
      case 'synced': return <CheckCircle2 className="h-3 w-3 text-[var(--vcs-green)]" />;
      case 'ahead': return <ArrowUp className="h-3 w-3 text-[var(--vcs-orange)]" />;
      case 'behind': return <ArrowDown className="h-3 w-3 text-[var(--vcs-red)]" />;
      case 'syncing': return <RotateCcw className="h-3 w-3 text-[var(--vcs-blue)] animate-spin" />;
    }
  };

  const switchRepository = (repoId: string) => {
    setRepositories(repositories.map(repo => ({
      ...repo,
      isActive: repo.id === repoId
    })));
  };

  const closeRepository = (repoId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (repositories.length > 1) {
      const newRepos = repositories.filter(repo => repo.id !== repoId);
      const closedRepo = repositories.find(repo => repo.id === repoId);
      if (closedRepo?.isActive && newRepos.length > 0) {
        newRepos[0].isActive = true;
      }
      setRepositories(newRepos);
    }
  };

  return (
    <div className="h-[60px] kodex-topbar flex items-center justify-between px-6 relative z-20">
      {/* Left Section - Logo + Repo Tabs */}
      <div className="flex items-center gap-6">
        {/* App Logo - Kodex Style */}
        <div className="flex items-center gap-3">
          <h1 className="text-[24px] font-bold kodex-aurora-text rune-font tracking-widest">
            ᚱᚢᚾᛖ
          </h1>
          {isPowerMode && (
            <div className="kodex-glow">
              <Zap className="h-5 w-5 text-[var(--kodex-aurora-blue)]" />
            </div>
          )}
        </div>
        
        {/* Repository Tabs - Kodex Style */}
        <Tabs value={activeRepository?.id} onValueChange={switchRepository} className="w-auto">
          <TabsList className="kodex-glass p-1 h-9 border-0">
            {repositories.map((repo) => (
              <TabsTrigger
                key={repo.id}
                value={repo.id}
                className="relative px-4 py-2 text-sm data-[state=active]:bg-[var(--kodex-active)] data-[state=active]:text-[var(--kodex-text-primary)] data-[state=active]:kodex-glow rounded-md transition-all"
              >
                <span className="truncate max-w-32">{repo.name}</span>
                {repositories.length > 1 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-4 w-4 p-0 ml-2 hover:bg-[var(--vcs-red)] hover:text-white opacity-50 hover:opacity-100 rounded transition-all"
                    onClick={(e) => closeRepository(repo.id, e)}
                  >
                    <X className="h-3 w-3" />
                  </Button>
                )}
              </TabsTrigger>
            ))}
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0 ml-2 kodex-glass hover:kodex-glow transition-all"
              onClick={() => {
                const newRepo: Repository = {
                  id: Date.now().toString(),
                  name: 'new-repo',
                  path: '/Projects/new-repo',
                  isActive: false
                };
                setRepositories([...repositories, newRepo]);
              }}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </TabsList>
        </Tabs>
      </div>
      
      {/* Center Section - Search + Branch */}
      <div className="flex items-center gap-4">
        {/* Quick Search - Kodex Style */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-[var(--kodex-text-muted)]" />
          <Input
            placeholder="Search commits, files..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setIsSearchFocused(true)}
            onBlur={() => setIsSearchFocused(false)}
            className={`pl-10 h-9 w-56 text-sm kodex-glass border-0 transition-all focus:kodex-glow ${
              isSearchFocused || searchQuery ? 'w-72' : ''
            }`}
          />
        </div>
        
        {/* Branch Selector - Kodex Style */}
        <div className="flex items-center gap-3">
          <GitBranch className="h-5 w-5 text-[var(--kodex-aurora-green)]" />
          <Select value={currentBranch} onValueChange={setCurrentBranch}>
            <SelectTrigger className="w-40 h-9 kodex-glass border-0 text-sm">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="kodex-glass border-0">
              {branches.map((branch) => (
                <SelectItem key={branch} value={branch}>
                  {branch}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      
      {/* Right Section - Actions */}
      <div className="flex items-center gap-3">
        {/* Sync Status - Kodex Style */}
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg kodex-glass">
          {getSyncIcon()}
          <span className="text-sm text-[var(--kodex-text-secondary)] capitalize font-medium">
            {syncStatus}
          </span>
        </div>
        
        {/* Pull/Push Buttons - Kodex Style */}
        <Button 
          variant="outline" 
          size="sm"
          className="h-9 px-4 text-sm kodex-glass border-0 hover:kodex-glow transition-all"
          disabled={syncStatus === 'syncing'}
        >
          <Download className="h-4 w-4 mr-2" />
          Pull
        </Button>
        <Button 
          variant="outline" 
          size="sm"
          className="h-9 px-4 text-sm bg-gradient-to-r from-[var(--kodex-aurora-blue)] to-[var(--kodex-aurora-purple)] border-0 text-white hover:opacity-90 transition-all shadow-lg"
          disabled={syncStatus === 'syncing' || syncStatus === 'synced'}
        >
          <Upload className="h-4 w-4 mr-2" />
          Push
          {syncStatus === 'ahead' && (
            <Badge className="ml-2 px-2 py-0 text-xs bg-white/20 text-white border-0 backdrop-blur-sm">
              2
            </Badge>
          )}
        </Button>
        
        {/* Dark Mode Toggle - Kodex Style */}
        <Button
          variant="ghost"
          size="sm"
          onClick={toggleTheme}
          className="h-9 w-9 p-0 kodex-glass hover:kodex-glow transition-all"
        >
          {isDark ? (
            <Sun className="h-5 w-5 text-[var(--kodex-aurora-green)]" />
          ) : (
            <Moon className="h-5 w-5 text-[var(--kodex-text-secondary)]" />
          )}
        </Button>
      </div>
    </div>
  );
}