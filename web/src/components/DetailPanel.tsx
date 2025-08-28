import { useState } from 'react';
import { 
  User, 
  Calendar, 
  GitCommit, 
  FileText, 
  Plus, 
  Minus, 
  Copy, 
  ExternalLink,
  Code,
  Eye,
  ChevronDown,
  ChevronRight
} from 'lucide-react';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';
import { ScrollArea } from './ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';

interface CommitDetail {
  id: string;
  message: string;
  author: string;
  authorEmail: string;
  date: string;
  hash: string;
  branch: string;
  changes: FileChange[];
}

interface FileChange {
  path: string;
  status: 'modified' | 'added' | 'deleted' | 'renamed';
  additions: number;
  deletions: number;
  isBinary?: boolean;
}

interface DetailPanelProps {
  selectedCommit: CommitDetail | null;
  isPowerMode: boolean;
}

export function DetailPanel({ selectedCommit, isPowerMode }: DetailPanelProps) {
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState('details');

  const toggleFileExpansion = (filePath: string) => {
    setExpandedFiles(prev => {
      const newSet = new Set(prev);
      if (newSet.has(filePath)) {
        newSet.delete(filePath);
      } else {
        newSet.add(filePath);
      }
      return newSet;
    });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'added': return 'text-[var(--kodex-aurora-green)]';
      case 'deleted': return 'text-[var(--vcs-red)]';
      case 'modified': return 'text-[var(--kodex-aurora-blue)]';
      case 'renamed': return 'text-[var(--kodex-aurora-purple)]';
      default: return 'text-[var(--kodex-text-secondary)]';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'added': return <Plus className="h-3 w-3" />;
      case 'deleted': return <Minus className="h-3 w-3" />;
      case 'modified': return <FileText className="h-3 w-3" />;
      case 'renamed': return <ExternalLink className="h-3 w-3" />;
      default: return <FileText className="h-3 w-3" />;
    }
  };

  if (!selectedCommit) {
    return (
      <div className="w-80 kodex-content border-l border-[var(--kodex-border)] flex items-center justify-center">
        <div className="text-center p-6">
          <GitCommit className="h-8 w-8 text-[var(--kodex-text-muted)] mx-auto mb-3" />
          <p className="text-sm text-[var(--kodex-text-muted)]">
            Select a commit to view details
          </p>
        </div>
      </div>
    );
  }

  const totalAdditions = selectedCommit.changes.reduce((sum, change) => sum + change.additions, 0);
  const totalDeletions = selectedCommit.changes.reduce((sum, change) => sum + change.deletions, 0);

  return (
    <div className="w-80 kodex-content border-l border-[var(--kodex-border)] flex flex-col">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col">
        <TabsList className="m-3 mb-0 p-0.5 h-8 bg-[var(--kodex-hover)]">
          <TabsTrigger value="details" className="flex-1 text-xs">Details</TabsTrigger>
          <TabsTrigger value="files" className="flex-1 text-xs">Files</TabsTrigger>
          {isPowerMode && <TabsTrigger value="diff" className="flex-1 text-xs">Diff</TabsTrigger>}
        </TabsList>

        <TabsContent value="details" className="flex-1 mt-0">
          <ScrollArea className="h-full">
            <div className="p-3 space-y-4">
              {/* Commit Header */}
              <div className="space-y-3">
                <div>
                  <h3 className="text-sm font-medium text-[var(--kodex-text-primary)] mb-2 leading-tight">
                    {selectedCommit.message}
                  </h3>
                  <div className="flex items-center gap-2">
                    <code className="text-xs bg-[var(--kodex-hover)] px-2 py-1 rounded text-[var(--kodex-text-secondary)]">
                      {selectedCommit.hash.substring(0, 8)}
                    </code>
                    <Button variant="ghost" size="sm" className="h-6 w-6 p-0">
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>

                {/* Author & Date */}
                <div className="space-y-2 text-xs">
                  <div className="flex items-center gap-2 text-[var(--kodex-text-secondary)]">
                    <User className="h-3 w-3" />
                    <span>{selectedCommit.author}</span>
                  </div>
                  <div className="flex items-center gap-2 text-[var(--kodex-text-secondary)]">
                    <Calendar className="h-3 w-3" />
                    <span>{selectedCommit.date}</span>
                  </div>
                  <div className="flex items-center gap-2 text-[var(--kodex-text-secondary)]">
                    <GitCommit className="h-3 w-3" />
                    <span className="text-[var(--kodex-aurora-green)]">{selectedCommit.branch}</span>
                  </div>
                </div>

                {/* Stats */}
                <div className="space-y-2">
                  <div className="text-xs text-[var(--kodex-text-muted)]">Changes</div>
                  <div className="flex gap-3 text-xs">
                    <span className="text-[var(--kodex-aurora-green)]">+{totalAdditions}</span>
                    <span className="text-[var(--vcs-red)]">-{totalDeletions}</span>
                    <span className="text-[var(--kodex-text-secondary)]">{selectedCommit.changes.length} files</span>
                  </div>
                </div>
              </div>

              {/* Quick Actions */}
              {isPowerMode && (
                <div className="space-y-2">
                  <div className="text-xs text-[var(--kodex-text-muted)]">Actions</div>
                  <div className="grid grid-cols-2 gap-1">
                    <Button variant="outline" size="sm" className="text-xs h-7">
                      View
                    </Button>
                    <Button variant="outline" size="sm" className="text-xs h-7">
                      Copy
                    </Button>
                  </div>
                </div>
              )}
            </div>
          </ScrollArea>
        </TabsContent>

        <TabsContent value="files" className="flex-1 mt-0">
          <ScrollArea className="h-full">
            <div className="p-3 space-y-1">
              {selectedCommit.changes.map((change, index) => (
                <div key={index} className="border border-[var(--kodex-border)] rounded p-2 space-y-1">
                  <div 
                    className="flex items-center gap-2 cursor-pointer"
                    onClick={() => toggleFileExpansion(change.path)}
                  >
                    <div className="w-3 h-3 flex items-center justify-center">
                      {expandedFiles.has(change.path) ? 
                        <ChevronDown className="h-3 w-3 text-[var(--kodex-text-muted)]" /> : 
                        <ChevronRight className="h-3 w-3 text-[var(--kodex-text-muted)]" />
                      }
                    </div>
                    <div className={`${getStatusColor(change.status)}`}>
                      {getStatusIcon(change.status)}
                    </div>
                    <span className="text-xs font-mono flex-1 truncate text-[var(--kodex-text-primary)]">{change.path}</span>
                    <div className="flex gap-2 text-xs">
                      {change.additions > 0 && (
                        <span className="text-[var(--kodex-aurora-green)]">+{change.additions}</span>
                      )}
                      {change.deletions > 0 && (
                        <span className="text-[var(--vcs-red)]">-{change.deletions}</span>
                      )}
                    </div>
                  </div>
                  
                  {expandedFiles.has(change.path) && (
                    <div className="pl-8 space-y-2">
                      <div className="text-xs text-[var(--kodex-text-muted)]">
                        {change.isBinary ? 'Binary file' : `${change.additions + change.deletions} lines changed`}
                      </div>
                      {isPowerMode && !change.isBinary && (
                        <div className="flex gap-2">
                          <Button variant="outline" size="sm" className="kodex-glass border-0 hover:kodex-glow text-xs h-7">
                            <Code className="h-3 w-3 mr-1" />
                            View Diff
                          </Button>
                          <Button variant="outline" size="sm" className="kodex-glass border-0 hover:kodex-glow text-xs h-7">
                            <Eye className="h-3 w-3 mr-1" />
                            View File
                          </Button>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </ScrollArea>
        </TabsContent>

        {isPowerMode && (
          <TabsContent value="diff" className="flex-1 mt-0">
            <ScrollArea className="h-full">
              <div className="p-4">
                <div className="kodex-glass rounded-lg p-4">
                  <div className="text-sm text-[var(--kodex-text-muted)] text-center">
                    <Code className="h-8 w-8 mx-auto mb-2" />
                    Diff view would show here
                    <br />
                    <span className="text-xs">Advanced diff visualization in Power Mode</span>
                  </div>
                </div>
              </div>
            </ScrollArea>
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}

// Mock data for demonstration
export const mockCommitDetail: CommitDetail = {
  id: '1',
  message: 'feat(ui): implement advanced commit visualization with GitKraken-inspired layout',
  author: 'Nordic Dev',
  authorEmail: 'dev@nordicstudio.com',
  date: 'Dec 15, 2024 at 2:30 PM',
  hash: 'a1b2c3d4e5f6',
  branch: 'main',
  changes: [
    { path: 'src/components/TopBar.tsx', status: 'modified', additions: 25, deletions: 8 },
    { path: 'src/components/DetailPanel.tsx', status: 'added', additions: 180, deletions: 0 },
    { path: 'src/components/RepositoryTree.tsx', status: 'added', additions: 145, deletions: 0 },
    { path: 'src/styles/globals.css', status: 'modified', additions: 12, deletions: 3 },
    { path: 'public/logo.png', status: 'deleted', additions: 0, deletions: 0, isBinary: true },
  ]
};