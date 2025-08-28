import { useState } from 'react';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { Textarea } from './ui/textarea';
import { Input } from './ui/input';
import { Separator } from './ui/separator';
import { ScrollArea } from './ui/scroll-area';
import { Badge } from './ui/badge';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from './ui/collapsible';
import { File, Plus, Minus, GitCommit, ChevronRight, ChevronDown, Folder, List } from 'lucide-react';

interface FileChange {
  id: string;
  name: string;
  path: string;
  status: 'added' | 'modified' | 'deleted';
  staged: boolean;
  additions: number;
  deletions: number;
  changelist?: string;
}

interface Changelist {
  id: string;
  name: string;
  description: string;
  isDefault: boolean;
  isExpanded: boolean;
}

interface ChangesViewProps {
  isPowerMode: boolean;
}

export function ChangesView({ isPowerMode }: ChangesViewProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>('src/components/auth.tsx');
  const [commitMessage, setCommitMessage] = useState('');
  const [changelists, setChangelists] = useState<Changelist[]>([
    { id: 'default', name: 'Default', description: 'Default changelist', isDefault: true, isExpanded: true },
    { id: 'auth', name: 'Authentication', description: 'Auth system improvements', isDefault: false, isExpanded: true },
    { id: 'ui', name: 'UI Updates', description: 'Visual improvements', isDefault: false, isExpanded: false }
  ]);
  const [files, setFiles] = useState<FileChange[]>([
    { id: '1', name: 'auth.tsx', path: 'src/components/auth.tsx', status: 'modified', staged: true, additions: 12, deletions: 3, changelist: 'auth' },
    { id: '2', name: 'login.tsx', path: 'src/pages/login.tsx', status: 'modified', staged: true, additions: 8, deletions: 2, changelist: 'auth' },
    { id: '3', name: 'database.ts', path: 'src/lib/database.ts', status: 'added', staged: false, additions: 45, deletions: 0, changelist: 'default' },
    { id: '4', name: 'config.json', path: 'config.json', status: 'modified', staged: false, additions: 3, deletions: 1, changelist: 'default' },
    { id: '5', name: 'old-utils.ts', path: 'src/utils/old-utils.ts', status: 'deleted', staged: false, additions: 0, deletions: 28, changelist: 'default' },
    { id: '6', name: 'styles.css', path: 'src/styles.css', status: 'modified', staged: false, additions: 15, deletions: 5, changelist: 'ui' },
    { id: '7', name: 'Button.tsx', path: 'src/components/Button.tsx', status: 'modified', staged: false, additions: 4, deletions: 0, changelist: 'ui' }
  ]);

  const stagedFiles = files.filter(f => f.staged);
  const unstagedFiles = files.filter(f => !f.staged);

  const toggleFileStaged = (fileId: string) => {
    setFiles(files.map(f => 
      f.id === fileId ? { ...f, staged: !f.staged } : f
    ));
  };

  const toggleChangelist = (changelistId: string) => {
    setChangelists(changelists.map(cl => 
      cl.id === changelistId ? { ...cl, isExpanded: !cl.isExpanded } : cl
    ));
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'added': return 'text-[var(--vcs-green)]';
      case 'modified': return 'text-[var(--vcs-orange)]';
      case 'deleted': return 'text-[var(--vcs-red)]';
      default: return 'text-[var(--vcs-text-secondary)]';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'added': return '+';
      case 'modified': return 'M';
      case 'deleted': return '−';
      default: return '?';
    }
  };

  const FileItem = ({ file, showCheckbox }: { file: FileChange; showCheckbox: boolean }) => (
    <div
      className={`flex items-center gap-3 p-2 rounded-md hover:bg-[var(--vcs-hover)] cursor-pointer transition-colors ${
        selectedFile === file.path ? 'bg-[var(--vcs-active)]' : ''
      }`}
      onClick={() => setSelectedFile(file.path)}
    >
      {showCheckbox && (
        <Checkbox
          checked={file.staged}
          onCheckedChange={() => toggleFileStaged(file.id)}
          onClick={(e) => e.stopPropagation()}
        />
      )}
      <div className={`w-5 h-5 rounded text-xs flex items-center justify-center font-medium ${getStatusColor(file.status)}`}>
        {getStatusIcon(file.status)}
      </div>
      <File className="h-4 w-4 text-[var(--vcs-text-muted)] flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="text-sm text-[var(--vcs-text-primary)] truncate">{file.name}</div>
        {isPowerMode && (
          <div className="text-xs text-[var(--vcs-text-muted)] truncate">{file.path}</div>
        )}
      </div>
      <div className="flex items-center gap-2 text-xs">
        {file.additions > 0 && (
          <span className="text-[var(--vcs-green)] flex items-center gap-1">
            <Plus className="h-3 w-3" />
            {file.additions}
          </span>
        )}
        {file.deletions > 0 && (
          <span className="text-[var(--vcs-red)] flex items-center gap-1">
            <Minus className="h-3 w-3" />
            {file.deletions}
          </span>
        )}
      </div>
    </div>
  );

  const ChangelistGroup = ({ changelist }: { changelist: Changelist }) => {
    const changelistFiles = files.filter(f => f.changelist === changelist.id);
    const stagedCount = changelistFiles.filter(f => f.staged).length;
    const totalCount = changelistFiles.length;

    if (totalCount === 0) return null;

    return (
      <Collapsible open={changelist.isExpanded} onOpenChange={() => toggleChangelist(changelist.id)}>
        <CollapsibleTrigger className="w-full">
          <div className="flex items-center gap-2 p-2 hover:bg-[var(--vcs-hover)] rounded-lg transition-colors">
            {changelist.isExpanded ? (
              <ChevronDown className="h-4 w-4 text-[var(--vcs-text-muted)]" />
            ) : (
              <ChevronRight className="h-4 w-4 text-[var(--vcs-text-muted)]" />
            )}
            <Folder className="h-4 w-4 text-[var(--vcs-blue)]" />
            <div className="flex-1 text-left">
              <div className="text-sm font-medium text-[var(--vcs-text-primary)]">
                {changelist.name}
              </div>
              {isPowerMode && (
                <div className="text-xs text-[var(--vcs-text-muted)]">
                  {changelist.description}
                </div>
              )}
            </div>
            <div className="flex items-center gap-2">
              {stagedCount > 0 && (
                <Badge className="text-xs px-1.5 py-0.5 bg-[var(--vcs-green)] text-white border-0">
                  {stagedCount} staged
                </Badge>
              )}
              <Badge variant="outline" className="text-xs px-1.5 py-0.5">
                {totalCount}
              </Badge>
            </div>
          </div>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div className="ml-6 space-y-1 mt-1">
            {changelistFiles.map((file) => (
              <FileItem key={file.id} file={file} showCheckbox={true} />
            ))}
          </div>
        </CollapsibleContent>
      </Collapsible>
    );
  };

  return (
    <div className="flex h-full">
      {/* File List */}
      <div className="w-1/2 border-r border-[var(--vcs-border)] flex flex-col">
        <div className="p-4 border-b border-[var(--vcs-border)] bg-[var(--vcs-gradient-start)]">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-[var(--vcs-text-primary)]">Changes</h3>
            {isPowerMode && (
              <Button
                variant="ghost"
                size="sm"
                className="h-6 px-2 text-xs"
              >
                <List className="h-3 w-3 mr-1" />
                Changelist
              </Button>
            )}
          </div>
        </div>

        <ScrollArea className="flex-1">
          <div className="p-4 space-y-4">
            {isPowerMode ? (
              // Power Mode: Show changelists
              <div className="space-y-3">
                {changelists.map((changelist) => (
                  <ChangelistGroup key={changelist.id} changelist={changelist} />
                ))}
              </div>
            ) : (
              // Simple Mode: Show staged/unstaged
              <>
                {/* Staged Changes */}
                {stagedFiles.length > 0 && (
                  <div className="space-y-2">
                    <h3 className="text-sm font-medium text-[var(--vcs-text-primary)] flex items-center gap-2">
                      Staged Changes
                      <Badge className="text-xs px-1.5 py-0.5 bg-[var(--vcs-green)] text-white border-0">
                        {stagedFiles.length}
                      </Badge>
                    </h3>
                    <div className="space-y-1">
                      {stagedFiles.map((file) => (
                        <FileItem key={file.id} file={file} showCheckbox={true} />
                      ))}
                    </div>
                  </div>
                )}

                {stagedFiles.length > 0 && unstagedFiles.length > 0 && <Separator />}

                {/* Unstaged Changes */}
                {unstagedFiles.length > 0 && (
                  <div className="space-y-2">
                    <h3 className="text-sm font-medium text-[var(--vcs-text-primary)] flex items-center gap-2">
                      Changes
                      <Badge variant="outline" className="text-xs px-1.5 py-0.5">
                        {unstagedFiles.length}
                      </Badge>
                    </h3>
                    <div className="space-y-1">
                      {unstagedFiles.map((file) => (
                        <FileItem key={file.id} file={file} showCheckbox={true} />
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}
          </div>
        </ScrollArea>

        {/* Commit Section */}
        <div className="border-t border-[var(--vcs-border)] p-4 bg-[var(--vcs-gradient-start)]">
          <div className="space-y-3">
            <Textarea
              placeholder={isPowerMode ? "Enter detailed commit message...\n\nDescribe what changed and why." : "Enter commit message..."}
              value={commitMessage}
              onChange={(e) => setCommitMessage(e.target.value)}
              className={`text-sm resize-none bg-[var(--vcs-window-bg)] border-[var(--vcs-border)] ${
                isPowerMode ? 'min-h-[100px]' : 'min-h-[80px]'
              }`}
            />
            
            {isPowerMode && (
              <div className="flex gap-2 text-xs">
                <Button variant="outline" size="sm" className="h-6 px-2">
                  Template
                </Button>
                <Button variant="outline" size="sm" className="h-6 px-2">
                  AI Suggest
                </Button>
              </div>
            )}

            <Button 
              className="w-full bg-[var(--vcs-blue)] hover:bg-blue-600 text-white"
              disabled={stagedFiles.length === 0 || !commitMessage.trim()}
            >
              <GitCommit className="h-4 w-4 mr-2" />
              Commit {stagedFiles.length} file{stagedFiles.length !== 1 ? 's' : ''}
            </Button>
          </div>
        </div>
      </div>

      {/* Diff Preview */}
      <div className="w-1/2 flex flex-col">
        {selectedFile ? (
          <>
            <div className="p-4 border-b border-[var(--vcs-border)] bg-[var(--vcs-gradient-start)]">
              <h3 className="text-sm font-medium text-[var(--vcs-text-primary)] truncate">
                {selectedFile}
              </h3>
              {isPowerMode && (
                <div className="flex gap-2 mt-2">
                  <Button variant="outline" size="sm" className="h-6 text-xs">
                    Side by Side
                  </Button>
                  <Button variant="outline" size="sm" className="h-6 text-xs">
                    Inline
                  </Button>
                  <Button variant="outline" size="sm" className="h-6 text-xs">
                    Blame
                  </Button>
                </div>
              )}
            </div>
            <ScrollArea className="flex-1">
              <div className="p-4 space-y-2 font-mono text-xs">
                {/* Enhanced diff content for power mode */}
                <div className="space-y-1">
                  <div className="text-[var(--vcs-text-muted)]">@@ -15,7 +15,10 @@ export function AuthForm() &#123;</div>
                  <div className="bg-red-50 dark:bg-red-950/20 text-[var(--vcs-red)] pl-4">- const handleSubmit = async (data) =&gt; &#123;</div>
                  <div className="bg-green-50 dark:bg-green-950/20 text-[var(--vcs-green)] pl-4">+ const handleSubmit = async (data: FormData) =&gt; &#123;</div>
                  <div className="pl-4 text-[var(--vcs-text-secondary)]">    try &#123;</div>
                  <div className="bg-green-50 dark:bg-green-950/20 text-[var(--vcs-green)] pl-4">+     console.log('Submitting form:', data);</div>
                  <div className="pl-4 text-[var(--vcs-text-secondary)]">      await login(data);</div>
                  <div className="bg-green-50 dark:bg-green-950/20 text-[var(--vcs-green)] pl-4">+     toast.success('Login successful');</div>
                  <div className="pl-4 text-[var(--vcs-text-secondary)]">    &#125; catch (error) &#123;</div>
                  <div className="bg-red-50 dark:bg-red-950/20 text-[var(--vcs-red)] pl-4">-     console.error(error);</div>
                  <div className="bg-green-50 dark:bg-green-950/20 text-[var(--vcs-green)] pl-4">+     console.error('Login failed:', error);</div>
                  <div className="bg-green-50 dark:bg-green-950/20 text-[var(--vcs-green)] pl-4">+     toast.error('Login failed');</div>
                  <div className="pl-4 text-[var(--vcs-text-secondary)]">    &#125;</div>
                  <div className="pl-4 text-[var(--vcs-text-secondary)]">  &#125;;</div>
                </div>
              </div>
            </ScrollArea>
          </>
        ) : (
          <div className="flex items-center justify-center h-full text-[var(--vcs-text-muted)]">
            <div className="text-center">
              <File className="h-12 w-12 mx-auto mb-3 opacity-50" />
              <p className="text-sm">Select a file to view changes</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}