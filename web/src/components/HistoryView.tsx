import { useState } from 'react';
import { ScrollArea } from './ui/scroll-area';
import { Avatar, AvatarFallback } from './ui/avatar';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { GitCommit, GitBranch, User, Calendar, FileText, Search, Filter, RotateCcw, Edit3, GitMerge } from 'lucide-react';
import { RepositoryTree } from './RepositoryTree';
import { DetailPanel, mockCommitDetail } from './DetailPanel';

interface Commit {
  id: string;
  hash: string;
  message: string;
  author: string;
  date: string;
  branch: string;
  files: number;
  additions: number;
  deletions: number;
  tags?: string[];
  isMerge?: boolean;
}

interface HistoryViewProps {
  isPowerMode: boolean;
  searchQuery: string;
}

export function HistoryView({ isPowerMode, searchQuery }: HistoryViewProps) {
  const [selectedCommit, setSelectedCommit] = useState<string>('commit-1');
  const [filterBy, setFilterBy] = useState<'all' | 'author' | 'branch'>('all');
  const [filterValue, setFilterValue] = useState('');
  const [isRebaseMode, setIsRebaseMode] = useState(false);

  const commits: Commit[] = [
    {
      id: 'commit-1',
      hash: '7a8b9c2',
      message: 'refactor: Removed redundant files',
      author: 'Johan',
      date: '2 hours ago',
      branch: 'main',
      files: 4,
      additions: 42,
      deletions: 12,
      tags: ['v2.1.0']
    },
    {
      id: 'commit-2',
      hash: '6d5e4f1',
      message: 'feat: integrate basic git source control',
      author: 'Johan',
      date: '5 hours ago',
      branch: 'feature/git-integration',
      files: 8,
      additions: 156,
      deletions: 23
    },
    {
      id: 'commit-3',
      hash: '5c4d3e2',
      message: 'feat: add git service and popup',
      author: 'Johan',
      date: '8 hours ago',
      branch: 'feature/git-integration',
      files: 7,
      additions: 89,
      deletions: 12
    },
    {
      id: 'commit-4',
      hash: '4b3a2c1',
      message: 'feat: add binder item duplication and deletion',
      author: 'Johan',
      date: '1 day ago',
      branch: 'main',
      files: 12,
      additions: 234,
      deletions: 45
    },
    {
      id: 'commit-5',
      hash: '3a2b1c9',
      message: 'test: cover binder item operations',
      author: 'Johan',
      date: '1 day ago',
      branch: 'feature/testing',
      files: 6,
      additions: 178,
      deletions: 8
    },
    {
      id: 'commit-6',
      hash: '2c1b9a8',
      message: 'Merge pull request #47 from Lone-Lodge/codex/remove...',
      author: 'GitHub',
      date: '2 days ago',
      branch: 'main',
      files: 15,
      additions: 298,
      deletions: 156,
      isMerge: true
    },
    {
      id: 'commit-7',
      hash: '1b9a8c7',
      message: 'chore: remove npm lockfiles and document pnpm usage...',
      author: 'Johan',
      date: '2 days ago',
      branch: 'main',
      files: 8,
      additions: 45,
      deletions: 234
    },
    {
      id: 'commit-8',
      hash: '9c8b7a6',
      message: 'Merge pull request #46 from Lone-Lodge/codex/determ...',
      author: 'GitHub',
      date: '3 days ago',
      branch: 'main',
      files: 12,
      additions: 167,
      deletions: 89,
      isMerge: true
    },
    {
      id: 'commit-9',
      hash: '8b7a6c5',
      message: 'chore: remove legacy backend',
      author: 'Johan',
      date: '3 days ago',
      branch: 'main',
      files: 23,
      additions: 12,
      deletions: 456
    },
    {
      id: 'commit-10',
      hash: '7a6c5b4',
      message: 'refactor: migrate localization to TypeScript assets and ...',
      author: 'System',
      date: '4 days ago',
      branch: 'feature/typescript-migration',
      files: 18,
      additions: 289,
      deletions: 145
    }
  ];

  const filteredCommits = commits.filter(commit => {
    const matchesSearch = searchQuery === '' || 
      commit.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      commit.author.toLowerCase().includes(searchQuery.toLowerCase()) ||
      commit.hash.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesFilter = filterBy === 'all' || 
      (filterBy === 'author' && (filterValue === '' || commit.author.toLowerCase().includes(filterValue.toLowerCase()))) ||
      (filterBy === 'branch' && (filterValue === '' || commit.branch.toLowerCase().includes(filterValue.toLowerCase())));
    
    return matchesSearch && matchesFilter;
  });

  const selectedCommitData = filteredCommits.find(c => c.id === selectedCommit);

  const getAuthorInitials = (name: string) => {
    return name.split(' ').map(n => n[0]).join('').toUpperCase();
  };

  const getBranchColor = (branch: string) => {
    if (branch === 'main') return 'bg-[var(--vcs-blue)] text-white';
    if (branch.includes('feature')) return 'bg-[var(--vcs-green)] text-white';
    if (branch.includes('hotfix')) return 'bg-[var(--vcs-red)] text-white';
    return 'bg-[var(--vcs-orange)] text-white';
  };

  return (
    <div className="flex h-full">
      {/* Repository Tree - GitKraken Style */}
      <RepositoryTree isPowerMode={isPowerMode} />
      
      {/* Enhanced Commit Graph - GitKraken Inspired */}
      <div className="flex-1 border-r border-[var(--kodex-border)] flex flex-col">
        <div className="p-3 border-b border-[var(--kodex-border)]">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-medium text-[var(--kodex-text-primary)]">History</h3>
              <span className="text-xs text-[var(--kodex-text-muted)]">{filteredCommits.length} commits</span>
            </div>
            
            {isPowerMode && (
              <div className="flex items-center gap-1">
                <Button
                  variant={isRebaseMode ? "default" : "outline"}
                  size="sm"
                  onClick={() => setIsRebaseMode(!isRebaseMode)}
                  className="text-xs h-7"
                >
                  <Edit3 className="h-3 w-3 mr-1" />
                  Rebase
                </Button>
                <Button variant="outline" size="sm" className="text-xs h-7">
                  <Filter className="h-3 w-3 mr-1" />
                  Filter
                </Button>
              </div>
            )}
          </div>
          
          {/* Simplified Filters */}
          {isPowerMode && (
            <div className="space-y-2">
              <div className="flex gap-2">
                <div className="relative flex-1">
                  <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 h-3 w-3 text-[var(--kodex-text-muted)]" />
                  <Input
                    placeholder="Search commits..."
                    value={searchQuery}
                    className="pl-7 h-7 text-xs bg-[var(--kodex-hover)] border-[var(--kodex-border)]"
                    readOnly
                  />
                </div>
              </div>
              
              {isRebaseMode && (
                <div className="p-2 rounded bg-[var(--kodex-hover)] border border-[var(--kodex-border)]">
                  <p className="text-xs text-[var(--kodex-text-secondary)]">
                    Interactive Rebase Mode
                  </p>
                </div>
              )}
            </div>
          )}
        </div>

        <ScrollArea className="flex-1">
          <div className="p-3 space-y-1">
            {filteredCommits.map((commit, index) => (
              <div
                key={commit.id}
                className={`flex items-start gap-3 p-2 rounded cursor-pointer transition-colors ${
                  selectedCommit === commit.id ? 
                    'bg-[var(--kodex-active)]' : 
                    'hover:bg-[var(--kodex-hover)]'
                }`}
                onClick={() => setSelectedCommit(commit.id)}
              >
                {/* Simple Commit Graph */}
                <div className="flex flex-col items-center mt-1">
                  <div className={`w-3 h-3 rounded-full ${
                    commit.isMerge ? 'bg-[var(--kodex-aurora-green)]' : 
                    selectedCommit === commit.id ? 'bg-[var(--kodex-aurora-blue)]' :
                    'bg-[var(--kodex-text-muted)]'
                  }`} />
                  {index < filteredCommits.length - 1 && (
                    <div className="w-px h-6 bg-[var(--kodex-border)] mt-1" />
                  )}
                </div>
                
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                      <span className="text-sm text-[var(--kodex-text-primary)] truncate">
                        {commit.message}
                      </span>
                      {commit.tags && commit.tags.map(tag => (
                        <span key={tag} className="text-xs px-1.5 py-0.5 bg-[var(--kodex-aurora-blue)] text-white rounded flex-shrink-0">
                          {tag}
                        </span>
                      ))}
                      {commit.isMerge && (
                        <span className="text-xs px-1.5 py-0.5 bg-[var(--kodex-aurora-green)]/20 text-[var(--kodex-aurora-green)] rounded flex-shrink-0">
                          merge
                        </span>
                      )}
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-3 text-xs text-[var(--kodex-text-muted)] mb-1">
                    <span>{commit.author}</span>
                    <code className="font-mono text-xs">{commit.hash}</code>
                    <span>{commit.date}</span>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-[var(--kodex-aurora-green)]">{commit.branch}</span>
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-[var(--kodex-aurora-green)]">+{commit.additions}</span>
                      <span className="text-[var(--vcs-red)]">-{commit.deletions}</span>
                      <span className="text-[var(--kodex-text-muted)]">{commit.files}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* GitKraken-Style Detail Panel */}
      <DetailPanel 
        selectedCommit={selectedCommitData ? {
          ...mockCommitDetail,
          id: selectedCommitData.id,
          message: selectedCommitData.message,
          author: selectedCommitData.author,
          date: selectedCommitData.date,
          hash: selectedCommitData.hash,
          branch: selectedCommitData.branch
        } : null}
        isPowerMode={isPowerMode}
      />
    </div>
  );
}