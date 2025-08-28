import { useState } from 'react';
import { ChevronRight, ChevronDown, File, Folder, FolderOpen, GitBranch, Plus, Minus, Circle } from 'lucide-react';
import { Button } from './ui/button';
import { Badge } from './ui/badge';

interface FileNode {
  id: string;
  name: string;
  type: 'file' | 'folder';
  status?: 'modified' | 'added' | 'deleted' | 'untracked';
  children?: FileNode[];
  isExpanded?: boolean;
}

interface RepositoryTreeProps {
  isPowerMode: boolean;
}

export function RepositoryTree({ isPowerMode }: RepositoryTreeProps) {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set(['1', '3', '7']));
  const [selectedFile, setSelectedFile] = useState<string>('');

  const mockFileTree: FileNode[] = [
    {
      id: '1',
      name: 'src',
      type: 'folder',
      children: [
        { id: '2', name: 'components', type: 'folder', children: [
          { id: '11', name: 'TopBar.tsx', type: 'file', status: 'modified' },
          { id: '12', name: 'Sidebar.tsx', type: 'file', status: 'modified' },
          { id: '13', name: 'HistoryView.tsx', type: 'file' },
        ]},
        { id: '3', name: 'utils', type: 'folder', children: [
          { id: '4', name: 'api.ts', type: 'file', status: 'added' },
          { id: '5', name: 'types.ts', type: 'file' },
        ]},
        { id: '6', name: 'App.tsx', type: 'file', status: 'modified' },
      ]
    },
    {
      id: '7',
      name: 'public',
      type: 'folder',
      children: [
        { id: '8', name: 'index.html', type: 'file' },
        { id: '9', name: 'favicon.ico', type: 'file', status: 'deleted' },
      ]
    },
    { id: '10', name: 'package.json', type: 'file', status: 'modified' },
    { id: '14', name: 'README.md', type: 'file', status: 'untracked' },
  ];

  const toggleFolder = (folderId: string) => {
    setExpandedFolders(prev => {
      const newSet = new Set(prev);
      if (newSet.has(folderId)) {
        newSet.delete(folderId);
      } else {
        newSet.add(folderId);
      }
      return newSet;
    });
  };

  const getStatusIcon = (status?: string) => {
    switch (status) {
      case 'modified': return <Circle className="h-2 w-2 fill-[var(--kodex-aurora-blue)] text-[var(--kodex-aurora-blue)]" />;
      case 'added': return <Plus className="h-3 w-3 text-[var(--kodex-aurora-green)]" />;
      case 'deleted': return <Minus className="h-3 w-3 text-[var(--vcs-red)]" />;
      case 'untracked': return <Circle className="h-2 w-2 fill-[var(--kodex-aurora-purple)] text-[var(--kodex-aurora-purple)]" />;
      default: return null;
    }
  };

  const getStatusCount = (status: string) => {
    return mockFileTree.reduce((count, node) => {
      const countInNode = (n: FileNode): number => {
        let total = n.status === status ? 1 : 0;
        if (n.children) {
          total += n.children.reduce((sum, child) => sum + countInNode(child), 0);
        }
        return total;
      };
      return count + countInNode(node);
    }, 0);
  };

  const renderFileNode = (node: FileNode, depth: number = 0) => {
    const isExpanded = expandedFolders.has(node.id);
    const isSelected = selectedFile === node.id;
    const paddingLeft = `${depth * 16 + 12}px`;

    return (
      <div key={node.id}>
        <div
          className={`flex items-center gap-2 py-1 px-2 hover:bg-[var(--kodex-hover)] cursor-pointer transition-colors ${
            isSelected ? 'bg-[var(--kodex-active)]' : ''
          }`}
          style={{ paddingLeft }}
          onClick={() => {
            if (node.type === 'folder') {
              toggleFolder(node.id);
            } else {
              setSelectedFile(node.id);
            }
          }}
        >
          {node.type === 'folder' && (
            <div className="w-3 h-3 flex items-center justify-center">
              {isExpanded ? <ChevronDown className="h-3 w-3 text-[var(--kodex-text-muted)]" /> : <ChevronRight className="h-3 w-3 text-[var(--kodex-text-muted)]" />}
            </div>
          )}
          
          {node.type === 'folder' ? (
            isExpanded ? <FolderOpen className="h-4 w-4 text-[var(--kodex-text-secondary)]" /> : <Folder className="h-4 w-4 text-[var(--kodex-text-secondary)]" />
          ) : (
            <File className="h-4 w-4 text-[var(--kodex-text-muted)]" />
          )}
          
          <span className="text-sm flex-1 text-[var(--kodex-text-primary)]">
            {node.name}
          </span>
          
          {node.status && getStatusIcon(node.status)}
        </div>
        
        {node.type === 'folder' && isExpanded && node.children && (
          <div>
            {node.children.map(child => renderFileNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="w-72 kodex-sidebar border-r border-[var(--kodex-border)] flex flex-col">
      {/* Header */}
      <div className="p-3 border-b border-[var(--kodex-border)]">
        <div className="flex items-center gap-2 mb-2">
          <GitBranch className="h-4 w-4 text-[var(--kodex-text-secondary)]" />
          <h3 className="text-sm font-medium text-[var(--kodex-text-primary)]">Files</h3>
        </div>
        
        {/* Status Summary */}
        {isPowerMode && (
          <div className="flex gap-1 text-xs">
            <span className="text-[var(--kodex-aurora-green)]">{getStatusCount('added')}</span>
            <span className="text-[var(--kodex-text-muted)]">added</span>
            <span className="text-[var(--kodex-aurora-blue)]">{getStatusCount('modified')}</span>
            <span className="text-[var(--kodex-text-muted)]">modified</span>
          </div>
        )}
      </div>

      {/* File Tree */}
      <div className="flex-1 overflow-y-auto">
        <div className="py-2">
          {mockFileTree.map(node => renderFileNode(node))}
        </div>
      </div>

      {/* Footer Actions */}
      {isPowerMode && (
        <div className="p-3 border-t border-[var(--kodex-border)]">
          <div className="space-y-1">
            <Button variant="outline" size="sm" className="w-full text-xs h-7">
              Stage All
            </Button>
            <Button variant="outline" size="sm" className="w-full text-xs h-7">
              Discard All
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}