import { useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog';
import { ScrollArea } from './ui/scroll-area';
import { Badge } from './ui/badge';
import { GitBranch, Plus, MoreHorizontal, Check, Calendar, User } from 'lucide-react';

interface Branch {
  id: string;
  name: string;
  isRemote: boolean;
  lastCommit: {
    hash: string;
    message: string;
    author: string;
    date: string;
  };
  ahead?: number;
  behind?: number;
}

interface BranchesViewProps {
  currentBranch: string;
  setCurrentBranch: (branch: string) => void;
}

export function BranchesView({ currentBranch, setCurrentBranch }: BranchesViewProps) {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [newBranchName, setNewBranchName] = useState('');

  const branches: Branch[] = [
    {
      id: 'main',
      name: 'main',
      isRemote: false,
      lastCommit: {
        hash: '7a8b9c2',
        message: 'Add user authentication with enhanced error handling',
        author: 'Sarah Chen',
        date: '2 hours ago'
      }
    },
    {
      id: 'develop',
      name: 'develop',
      isRemote: false,
      lastCommit: {
        hash: '6d5e4f1',
        message: 'Update database schema for user profiles',
        author: 'Alex Rivera',
        date: '5 hours ago'
      },
      ahead: 3,
      behind: 1
    },
    {
      id: 'feature/auth',
      name: 'feature/auth',
      isRemote: false,
      lastCommit: {
        hash: '5c4d3e2',
        message: 'Fix responsive design issues on mobile devices',
        author: 'Maya Patel',
        date: '1 day ago'
      },
      ahead: 2
    },
    {
      id: 'hotfix/security',
      name: 'hotfix/security',
      isRemote: false,
      lastCommit: {
        hash: '4b3a2c1',
        message: 'Implement security patches for authentication',
        author: 'Jordan Kim',
        date: '3 days ago'
      },
      behind: 5
    }
  ];

  const handleCreateBranch = () => {
    if (newBranchName.trim()) {
      // In a real app, this would create the branch
      console.log('Creating branch:', newBranchName);
      setIsCreateDialogOpen(false);
      setNewBranchName('');
    }
  };

  const handleSwitchBranch = (branchName: string) => {
    setCurrentBranch(branchName);
  };

  return (
    <div className="flex h-full">
      {/* Branch List */}
      <div className="flex-1 flex flex-col">
        <div className="p-4 border-b border-[var(--vcs-border)] bg-[var(--vcs-gradient-start)] flex items-center justify-between">
          <h3 className="text-sm font-medium text-[var(--vcs-text-primary)]">Branches</h3>
          <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm" className="bg-[var(--vcs-blue)] hover:bg-blue-600 text-white">
                <Plus className="h-4 w-4 mr-1" />
                New Branch
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <DialogHeader>
                <DialogTitle>Create New Branch</DialogTitle>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <label className="text-sm font-medium text-[var(--vcs-text-primary)] mb-2 block">
                    Branch name
                  </label>
                  <Input
                    placeholder="e.g. feature/new-component"
                    value={newBranchName}
                    onChange={(e) => setNewBranchName(e.target.value)}
                    className="bg-[var(--vcs-window-bg)] border-[var(--vcs-border)]"
                  />
                </div>
                <div className="text-xs text-[var(--vcs-text-muted)]">
                  Branch will be created from: <span className="font-medium">{currentBranch}</span>
                </div>
                <div className="flex gap-2 justify-end">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setIsCreateDialogOpen(false)}
                  >
                    Cancel
                  </Button>
                  <Button
                    size="sm"
                    onClick={handleCreateBranch}
                    disabled={!newBranchName.trim()}
                    className="bg-[var(--vcs-blue)] hover:bg-blue-600 text-white"
                  >
                    Create Branch
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>

        <ScrollArea className="flex-1">
          <div className="p-4 space-y-3">
            {branches.map((branch) => {
              const isCurrent = currentBranch === branch.name;
              
              return (
                <div
                  key={branch.id}
                  className={`p-4 rounded-lg border transition-all duration-200 cursor-pointer hover:shadow-sm ${
                    isCurrent 
                      ? 'bg-[var(--vcs-blue)]/10 border-[var(--vcs-blue)] shadow-sm' 
                      : 'bg-[var(--vcs-window-bg)] border-[var(--vcs-border)] hover:bg-[var(--vcs-hover)]'
                  }`}
                  onClick={() => handleSwitchBranch(branch.name)}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-2">
                      {isCurrent ? (
                        <Check className={`h-4 w-4 text-[var(--vcs-blue)]`} />
                      ) : (
                        <GitBranch className="h-4 w-4 text-[var(--vcs-text-muted)]" />
                      )}
                      <span className={`text-sm font-medium ${
                        isCurrent ? 'text-[var(--vcs-blue)]' : 'text-[var(--vcs-text-primary)]'
                      }`}>
                        {branch.name}
                      </span>
                      {isCurrent && (
                        <Badge variant="secondary" className="text-xs px-1.5 py-0.5">
                          current
                        </Badge>
                      )}
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0 hover:bg-[var(--vcs-hover)]"
                    >
                      <MoreHorizontal className="h-3 w-3" />
                    </Button>
                  </div>

                  {/* Branch Status */}
                  {(branch.ahead || branch.behind) && (
                    <div className="flex items-center gap-3 mb-3 text-xs">
                      {branch.ahead && (
                        <span className="text-[var(--vcs-green)]">
                          ↑ {branch.ahead} ahead
                        </span>
                      )}
                      {branch.behind && (
                        <span className="text-[var(--vcs-orange)]">
                          ↓ {branch.behind} behind
                        </span>
                      )}
                    </div>
                  )}

                  {/* Last Commit */}
                  <div className="space-y-2">
                    <p className="text-xs text-[var(--vcs-text-secondary)] line-clamp-2">
                      {branch.lastCommit.message}
                    </p>
                    <div className="flex items-center gap-4 text-xs text-[var(--vcs-text-muted)]">
                      <span className="flex items-center gap-1">
                        <User className="h-3 w-3" />
                        {branch.lastCommit.author}
                      </span>
                      <span className="flex items-center gap-1">
                        <Calendar className="h-3 w-3" />
                        {branch.lastCommit.date}
                      </span>
                      <span className="font-mono">
                        {branch.lastCommit.hash}
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}