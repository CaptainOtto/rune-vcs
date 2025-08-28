import { useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { ScrollArea } from './ui/scroll-area';
import { Separator } from './ui/separator';
import { Badge } from './ui/badge';
import { 
  Archive, 
  Calendar, 
  User, 
  FileText, 
  RotateCcw, 
  Trash2, 
  Plus,
  Eye,
  Download
} from 'lucide-react';

interface Stash {
  id: string;
  index: number;
  message: string;
  branch: string;
  author: string;
  date: string;
  filesChanged: number;
  additions: number;
  deletions: number;
}

export function StashView() {
  const [selectedStash, setSelectedStash] = useState<string>('stash-1');
  const [newStashMessage, setNewStashMessage] = useState('');

  const stashes: Stash[] = [
    {
      id: 'stash-1',
      index: 0,
      message: 'WIP: working on authentication flow',
      branch: 'feature/auth',
      author: 'Johan',
      date: '2 hours ago',
      filesChanged: 5,
      additions: 23,
      deletions: 8
    },
    {
      id: 'stash-2',
      index: 1,
      message: 'stash@{1}: temporary UI changes',
      branch: 'main',
      author: 'Johan',
      date: '1 day ago',
      filesChanged: 3,
      additions: 12,
      deletions: 4
    }
  ];

  const selectedStashData = stashes.find(s => s.id === selectedStash);

  const handleCreateStash = () => {
    if (newStashMessage.trim()) {
      console.log('Creating stash:', newStashMessage);
      setNewStashMessage('');
    }
  };

  const handleApplyStash = (stashId: string) => {
    console.log('Applying stash:', stashId);
  };

  const handleDropStash = (stashId: string) => {
    console.log('Dropping stash:', stashId);
  };

  return (
    <div className="flex h-full">
      {/* Stash List */}
      <div className="w-1/2 border-r border-[var(--vcs-border)] flex flex-col">
        <div className="p-4 border-b border-[var(--vcs-border)] bg-[var(--vcs-gradient-start)]">
          <h3 className="text-sm font-medium text-[var(--vcs-text-primary)] mb-3">Git Stash</h3>
          
          {/* Create New Stash */}
          <div className="space-y-2">
            <Input
              placeholder="Stash message (optional)"
              value={newStashMessage}
              onChange={(e) => setNewStashMessage(e.target.value)}
              className="text-sm bg-[var(--vcs-window-bg)] border-[var(--vcs-border)]"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleCreateStash();
                }
              }}
            />
            <Button 
              size="sm"
              onClick={handleCreateStash}
              className="w-full bg-[var(--vcs-blue)] hover:bg-blue-600 text-white"
            >
              <Archive className="h-4 w-4 mr-2" />
              Stash Changes
            </Button>
          </div>
        </div>

        <ScrollArea className="flex-1">
          <div className="p-4 space-y-3">
            {stashes.length === 0 ? (
              <div className="text-center py-8">
                <Archive className="h-12 w-12 mx-auto mb-3 text-[var(--vcs-text-muted)] opacity-50" />
                <p className="text-sm text-[var(--vcs-text-muted)]">No stashes yet</p>
                <p className="text-xs text-[var(--vcs-text-muted)] mt-1">
                  Stash your changes to save them for later
                </p>
              </div>
            ) : (
              stashes.map((stash) => (
                <div
                  key={stash.id}
                  className={`p-4 rounded-lg border cursor-pointer transition-all duration-200 ${
                    selectedStash === stash.id
                      ? 'bg-[var(--vcs-active)] border-[var(--vcs-blue)] shadow-sm'
                      : 'bg-[var(--vcs-window-bg)] border-[var(--vcs-border)] hover:bg-[var(--vcs-hover)]'
                  }`}
                  onClick={() => setSelectedStash(stash.id)}
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="text-xs font-mono px-2">
                        stash@&#123;{stash.index}&#125;
                      </Badge>
                      <Badge 
                        className="text-xs px-2 bg-[var(--vcs-blue)] text-white border-0"
                      >
                        {stash.branch}
                      </Badge>
                    </div>
                  </div>
                  
                  <p className="text-sm text-[var(--vcs-text-primary)] mb-2 line-clamp-2">
                    {stash.message}
                  </p>
                  
                  <div className="flex items-center justify-between text-xs text-[var(--vcs-text-muted)]">
                    <div className="flex items-center gap-3">
                      <span className="flex items-center gap-1">
                        <User className="h-3 w-3" />
                        {stash.author}
                      </span>
                      <span className="flex items-center gap-1">
                        <Calendar className="h-3 w-3" />
                        {stash.date}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-[var(--vcs-green)]">+{stash.additions}</span>
                      <span className="text-[var(--vcs-red)]">-{stash.deletions}</span>
                    </div>
                  </div>
                  
                  <Separator className="my-3" />
                  
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleApplyStash(stash.id);
                      }}
                      className="flex-1 h-7 text-xs"
                    >
                      <Download className="h-3 w-3 mr-1" />
                      Apply
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        // Show diff view
                      }}
                      className="h-7 px-2"
                    >
                      <Eye className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDropStash(stash.id);
                      }}
                      className="h-7 px-2 text-[var(--vcs-red)] border-[var(--vcs-red)]/30 hover:bg-[var(--vcs-red)]/10"
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              ))
            )}
          </div>
        </ScrollArea>
      </div>

      {/* Stash Details */}
      <div className="w-1/2 flex flex-col">
        {selectedStashData ? (
          <>
            <div className="p-4 border-b border-[var(--vcs-border)] bg-[var(--vcs-gradient-start)]">
              <div className="flex items-start justify-between mb-3">
                <div>
                  <h3 className="text-sm font-medium text-[var(--vcs-text-primary)] mb-1">
                    {selectedStashData.message}
                  </h3>
                  <div className="flex items-center gap-4 text-xs text-[var(--vcs-text-muted)]">
                    <span>stash@&#123;{selectedStashData.index}&#125;</span>
                    <span>{selectedStashData.author}</span>
                    <span>{selectedStashData.date}</span>
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button 
                    size="sm"
                    onClick={() => handleApplyStash(selectedStashData.id)}
                    className="bg-[var(--vcs-green)] hover:bg-green-600 text-white"
                  >
                    <RotateCcw className="h-3 w-3 mr-1" />
                    Apply
                  </Button>
                  <Button 
                    variant="outline"
                    size="sm"
                    onClick={() => handleDropStash(selectedStashData.id)}
                    className="text-[var(--vcs-red)] border-[var(--vcs-red)]/30 hover:bg-[var(--vcs-red)]/10"
                  >
                    <Trash2 className="h-3 w-3 mr-1" />
                    Drop
                  </Button>
                </div>
              </div>
              
              <div className="flex items-center gap-6 text-sm">
                <span className="flex items-center gap-1 text-[var(--vcs-text-secondary)]">
                  <FileText className="h-4 w-4" />
                  {selectedStashData.filesChanged} files changed
                </span>
                <span className="text-[var(--vcs-green)]">
                  +{selectedStashData.additions} additions
                </span>
                <span className="text-[var(--vcs-red)]">
                  -{selectedStashData.deletions} deletions
                </span>
              </div>
            </div>

            {/* Changed Files */}
            <ScrollArea className="flex-1">
              <div className="p-4 space-y-3">
                <h4 className="text-sm font-medium text-[var(--vcs-text-primary)] mb-3">Changed Files</h4>
                <div className="space-y-2">
                  {[
                    { name: 'src/components/Auth.tsx', additions: 15, deletions: 3 },
                    { name: 'src/pages/Login.tsx', additions: 8, deletions: 2 },
                    { name: 'src/lib/utils.ts', additions: 0, deletions: 3 }
                  ].map((file, index) => (
                    <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-[var(--vcs-hover)] hover:bg-[var(--vcs-active)] cursor-pointer transition-colors">
                      <div className="flex items-center gap-2">
                        <FileText className="h-4 w-4 text-[var(--vcs-text-muted)]" />
                        <span className="text-sm text-[var(--vcs-text-primary)]">{file.name}</span>
                      </div>
                      <div className="flex items-center gap-2 text-xs">
                        {file.additions > 0 && (
                          <span className="text-[var(--vcs-green)]">+{file.additions}</span>
                        )}
                        {file.deletions > 0 && (
                          <span className="text-[var(--vcs-red)]">-{file.deletions}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </ScrollArea>
          </>
        ) : (
          <div className="flex items-center justify-center h-full text-[var(--vcs-text-muted)]">
            <div className="text-center">
              <Archive className="h-12 w-12 mx-auto mb-3 opacity-50" />
              <p className="text-sm">Select a stash to view details</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}