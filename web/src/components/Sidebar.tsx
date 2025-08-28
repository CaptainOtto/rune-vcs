import { FileText, History, GitBranch, Archive, Settings, Zap, ZapOff } from 'lucide-react';
import { Button } from './ui/button';
import { Separator } from './ui/separator';
import { Badge } from './ui/badge';

type ViewType = 'changes' | 'history' | 'branches' | 'stash' | 'settings';

interface SidebarProps {
  activeView: ViewType;
  setActiveView: (view: ViewType) => void;
  isPowerMode: boolean;
  togglePowerMode: () => void;
}

export function Sidebar({ activeView, setActiveView, isPowerMode, togglePowerMode }: SidebarProps) {
  const tabs = [
    {
      id: 'changes' as ViewType,
      label: 'Changes',
      icon: FileText,
      count: 7,
      description: 'Modified files'
    },
    {
      id: 'history' as ViewType,
      label: 'History',
      icon: History,
      description: 'Commit timeline'
    },
    {
      id: 'branches' as ViewType,
      label: 'Branches',
      icon: GitBranch,
      count: 4,
      description: 'Branch management'
    },
    {
      id: 'stash' as ViewType,
      label: 'Stash',
      icon: Archive,
      count: 2,
      description: 'Temporary changes'
    },
    {
      id: 'settings' as ViewType,
      label: 'Settings',
      icon: Settings,
      description: 'Configuration'
    }
  ];

  return (
    <div className="w-64 kodex-sidebar flex flex-col relative z-10">
      {/* Navigation Tabs - Kodex Style */}
      <div className="p-6 space-y-2">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeView === tab.id;
          
          return (
            <button
              key={tab.id}
              onClick={() => setActiveView(tab.id)}
              className={`w-full flex items-center gap-4 px-4 py-3 rounded-xl transition-all duration-300 text-left group ${
                isActive 
                  ? 'bg-gradient-to-r from-[var(--kodex-aurora-blue)] to-[var(--kodex-aurora-purple)] text-white shadow-lg kodex-glow' 
                  : 'text-[var(--kodex-text-secondary)] hover:kodex-glass hover:text-[var(--kodex-text-primary)]'
              }`}
            >
              <Icon className="h-5 w-5 flex-shrink-0" strokeWidth={2} />
              <div className="flex-1 min-w-0">
                <span className="text-sm font-medium">{tab.label}</span>
                {isPowerMode && tab.description && (
                  <div className="text-xs opacity-75 truncate mt-1">
                    {tab.description}
                  </div>
                )}
              </div>
              {tab.count && (
                <Badge 
                  className={`text-xs px-2 py-1 rounded-full ${
                    isActive 
                      ? 'bg-white/20 text-white border-0' 
                      : 'bg-[var(--kodex-aurora-green)]/20 text-[var(--kodex-aurora-green)] border-0'
                  }`}
                >
                  {tab.count}
                </Badge>
              )}
            </button>
          );
        })}
      </div>

      <div className="mx-6 my-4 h-px bg-gradient-to-r from-transparent via-[var(--kodex-border)] to-transparent" />

      {/* Power Mode Toggle - Kodex Style */}
      <div className="px-6 pb-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={togglePowerMode}
          className={`w-full justify-start gap-3 h-12 rounded-xl transition-all duration-300 ${
            isPowerMode 
              ? 'bg-gradient-to-r from-[var(--kodex-aurora-green)]/20 to-[var(--kodex-aurora-blue)]/20 text-[var(--kodex-aurora-green)] kodex-glow' 
              : 'text-[var(--kodex-text-muted)] hover:kodex-glass hover:text-[var(--kodex-text-primary)]'
          }`}
        >
          {isPowerMode ? (
            <ZapOff className="h-5 w-5" />
          ) : (
            <Zap className="h-5 w-5" />
          )}
          <span className="text-sm font-medium">
            {isPowerMode ? 'Exit Power Mode' : 'Power Mode'}
          </span>
        </Button>
        {isPowerMode && (
          <p className="text-xs text-[var(--kodex-text-muted)] mt-3 px-3">
            Advanced features enabled
          </p>
        )}
      </div>

      {/* Repository Info - Kodex Style */}
      <div className="mt-auto p-6 border-t border-[var(--kodex-border)]">
        <div className="space-y-3 kodex-glass p-4 rounded-xl">
          <div className="flex items-center justify-between text-xs text-[var(--kodex-text-muted)]">
            <span>Current repo</span>
            <Badge className="text-xs bg-[var(--kodex-aurora-blue)]/20 text-[var(--kodex-aurora-blue)] border-0">
              Rune
            </Badge>
          </div>
          <div className="text-sm text-[var(--kodex-text-primary)] font-mono truncate">
            rune-desktop
          </div>
          <div className="text-xs text-[var(--kodex-text-muted)] truncate">
            /Projects/rune-desktop
          </div>
        </div>
      </div>
    </div>
  );
}