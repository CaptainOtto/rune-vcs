import { useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Switch } from './ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { ScrollArea } from './ui/scroll-area';
import { Separator } from './ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Badge } from './ui/badge';
import { 
  Settings, 
  User, 
  Palette, 
  Zap, 
  Shield, 
  Database,
  ExternalLink,
  FolderOpen,
  Terminal,
  GitBranch,
  Bell
} from 'lucide-react';

interface SettingsViewProps {
  isDark: boolean;
  toggleTheme: () => void;
}

export function SettingsView({ isDark, toggleTheme }: SettingsViewProps) {
  const [userName, setUserName] = useState('Johan');
  const [userEmail, setUserEmail] = useState('johan@example.com');
  const [defaultEditor, setDefaultEditor] = useState('vscode');
  const [autoFetch, setAutoFetch] = useState(true);
  const [showNotifications, setShowNotifications] = useState(true);
  const [useExternalDiff, setUseExternalDiff] = useState(false);
  const [enableHooks, setEnableHooks] = useState(true);

  const settings = [
    {
      id: 'general',
      label: 'General',
      icon: Settings,
      content: (
        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">Application</h3>
            
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Theme</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Choose your preferred theme
                </p>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-sm text-[var(--vcs-text-secondary)]">
                  {isDark ? 'Dark' : 'Light'}
                </span>
                <Switch checked={isDark} onCheckedChange={toggleTheme} />
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Show notifications</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Get notified about important events
                </p>
              </div>
              <Switch checked={showNotifications} onCheckedChange={setShowNotifications} />
            </div>

            <div className="space-y-2">
              <Label>Default editor</Label>
              <Select value={defaultEditor} onValueChange={setDefaultEditor}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="vscode">Visual Studio Code</SelectItem>
                  <SelectItem value="cursor">Cursor</SelectItem>
                  <SelectItem value="sublime">Sublime Text</SelectItem>
                  <SelectItem value="atom">Atom</SelectItem>
                  <SelectItem value="vim">Vim</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>Window size on startup</Label>
              <Select defaultValue="compact">
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="compact">Compact (1100×720)</SelectItem>
                  <SelectItem value="large">Large (1400×900)</SelectItem>
                  <SelectItem value="fullscreen">Fullscreen</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'rune',
      label: 'Rune',
      icon: GitBranch,
      content: (
        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">User Information</h3>
            
            <div className="space-y-2">
              <Label>Name</Label>
              <Input 
                value={userName} 
                onChange={(e) => setUserName(e.target.value)}
                placeholder="Your full name"
              />
              <p className="text-xs text-[var(--vcs-text-muted)]">
                Used for commit authorship in Rune repositories
              </p>
            </div>

            <div className="space-y-2">
              <Label>Email</Label>
              <Input 
                value={userEmail} 
                onChange={(e) => setUserEmail(e.target.value)}
                placeholder="your.email@example.com"
                type="email"
              />
              <p className="text-xs text-[var(--vcs-text-muted)]">
                Associated with your commits and used for collaboration
              </p>
            </div>
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">Repository Behavior</h3>
            
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Auto-sync</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Automatically fetch changes from remote repositories
                </p>
              </div>
              <Switch checked={autoFetch} onCheckedChange={setAutoFetch} />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Use external diff tool</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Use external application for viewing differences
                </p>
              </div>
              <Switch checked={useExternalDiff} onCheckedChange={setUseExternalDiff} />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Enable Rune hooks</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Run pre-commit and post-commit automation
                </p>
              </div>
              <Switch checked={enableHooks} onCheckedChange={setEnableHooks} />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Auto-create changelists</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Automatically organize changes by feature or task
                </p>
              </div>
              <Switch defaultChecked={true} />
            </div>
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">Merge & Conflict Resolution</h3>
            
            <div className="space-y-2">
              <Label>Default merge strategy</Label>
              <Select defaultValue="auto">
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto">Auto-merge when possible</SelectItem>
                  <SelectItem value="manual">Always require manual review</SelectItem>
                  <SelectItem value="fast-forward">Fast-forward only</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label>Show merge previews</Label>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Preview merge results before applying
                </p>
              </div>
              <Switch defaultChecked={true} />
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'integrations',
      label: 'Integrations',
      icon: ExternalLink,
      content: (
        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">External Tools</h3>
            
            <div className="space-y-3">
              {[
                { name: 'Beyond Compare', status: 'Available', available: true, type: 'Diff Tool' },
                { name: 'Kaleidoscope', status: 'Not installed', available: false, type: 'Diff Tool' },
                { name: 'VS Code', status: 'Available', available: true, type: 'Editor' },
                { name: 'Sublime Merge', status: 'Not installed', available: false, type: 'VCS Tool' }
              ].map((tool, index) => (
                <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-[var(--vcs-hover)]">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-[var(--vcs-blue)]/10 rounded-lg flex items-center justify-center">
                      <ExternalLink className="h-4 w-4 text-[var(--vcs-blue)]" />
                    </div>
                    <div>
                      <div className="text-sm font-medium text-[var(--vcs-text-primary)]">
                        {tool.name}
                      </div>
                      <div className="text-xs text-[var(--vcs-text-muted)]">
                        {tool.type} • {tool.status}
                      </div>
                    </div>
                  </div>
                  <Badge 
                    variant={tool.available ? "default" : "secondary"}
                    className={tool.available ? "bg-[var(--vcs-green)] text-white" : ""}
                  >
                    {tool.available ? 'Available' : 'Install'}
                  </Badge>
                </div>
              ))}
            </div>
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">Repository Hosting</h3>
            
            <div className="space-y-3">
              {[
                { name: 'GitHub', connected: true, type: 'Git hosting' },
                { name: 'GitLab', connected: false, type: 'Git hosting' },
                { name: 'Bitbucket', connected: false, type: 'Git hosting' },
                { name: 'Rune Cloud', connected: false, type: 'Native Rune hosting' }
              ].map((service, index) => (
                <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-[var(--vcs-hover)]">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-[var(--vcs-blue)]/10 rounded-lg flex items-center justify-center">
                      <Database className="h-4 w-4 text-[var(--vcs-blue)]" />
                    </div>
                    <div>
                      <div className="text-sm font-medium text-[var(--vcs-text-primary)]">
                        {service.name}
                      </div>
                      <div className="text-xs text-[var(--vcs-text-muted)]">
                        {service.type}
                      </div>
                    </div>
                  </div>
                  <Button 
                    variant={service.connected ? "outline" : "default"}
                    size="sm"
                    className={service.connected ? "text-[var(--vcs-red)] border-[var(--vcs-red)]/30" : ""}
                  >
                    {service.connected ? 'Disconnect' : 'Connect'}
                  </Button>
                </div>
              ))}
            </div>
          </div>
        </div>
      )
    },
    {
      id: 'advanced',
      label: 'Advanced',
      icon: Terminal,
      content: (
        <div className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <h3 className="font-medium text-[var(--vcs-text-primary)]">Power User Settings</h3>
              <Badge className="bg-[var(--vcs-orange)] text-white text-xs">
                Advanced
              </Badge>
            </div>
            
            <div className="space-y-3">
              <div className="p-4 rounded-lg border border-[var(--vcs-orange)]/20 bg-[var(--vcs-orange)]/5">
                <div className="flex items-start gap-3">
                  <Shield className="h-5 w-5 text-[var(--vcs-orange)] flex-shrink-0 mt-0.5" />
                  <div>
                    <h4 className="text-sm font-medium text-[var(--vcs-text-primary)] mb-1">
                      Experimental Features
                    </h4>
                    <p className="text-xs text-[var(--vcs-text-muted)] mb-3">
                      These features are experimental and may cause instability.
                    </p>
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-sm">Interactive rebase mode</span>
                        <Switch defaultChecked={false} />
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-sm">Media file preview (LFS)</span>
                        <Switch defaultChecked={true} />
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-sm">Inline blame annotations</span>
                        <Switch defaultChecked={false} />
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-sm">AI commit message suggestions</span>
                        <Switch defaultChecked={false} />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">System Configuration</h3>
            
            <div className="space-y-3">
              <div className="space-y-2">
                <Label>Default clone location</Label>
                <div className="flex gap-2">
                  <Input 
                    defaultValue="/Users/johan/Projects"
                    className="flex-1"
                  />
                  <Button variant="outline" size="sm">
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                </div>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Default directory for new repository clones
                </p>
              </div>

              <div className="space-y-2">
                <Label>Rune executable path</Label>
                <div className="flex gap-2">
                  <Input 
                    defaultValue="/usr/local/bin/rune"
                    className="font-mono text-sm flex-1"
                  />
                  <Button variant="outline" size="sm">
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                </div>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Path to the Rune command-line tool
                </p>
              </div>

              <div className="space-y-2">
                <Label>Performance</Label>
                <Select defaultValue="auto">
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">Auto (recommended)</SelectItem>
                    <SelectItem value="high">High performance</SelectItem>
                    <SelectItem value="low">Low resource usage</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-[var(--vcs-text-muted)]">
                  Optimize for performance or resource usage
                </p>
              </div>
            </div>
          </div>

          <Separator />

          <div className="space-y-4">
            <h3 className="font-medium text-[var(--vcs-text-primary)]">Debugging</h3>
            
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <Label>Enable debug logging</Label>
                  <p className="text-xs text-[var(--vcs-text-muted)]">
                    Log detailed information for troubleshooting
                  </p>
                </div>
                <Switch defaultChecked={false} />
              </div>

              <div className="flex gap-2">
                <Button variant="outline" size="sm" className="flex-1">
                  <Terminal className="h-3 w-3 mr-1" />
                  Open Log Directory
                </Button>
                <Button variant="outline" size="sm" className="flex-1">
                  <ExternalLink className="h-3 w-3 mr-1" />
                  Export Debug Info
                </Button>
              </div>
            </div>
          </div>
        </div>
      )
    }
  ];

  return (
    <div className="flex h-full">
      {/* Settings Navigation */}
      <div className="w-64 border-r border-[var(--vcs-border)] bg-[var(--vcs-sidebar-bg)]">
        <div className="p-4 border-b border-[var(--vcs-border)]">
          <h2 className="font-medium text-[var(--vcs-text-primary)]">Settings</h2>
        </div>
        <div className="p-4">
          <div className="space-y-1">
            {settings.map((setting) => {
              const Icon = setting.icon;
              return (
                <button
                  key={setting.id}
                  className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left hover:bg-[var(--vcs-hover)] transition-colors"
                >
                  <Icon className="h-4 w-4 text-[var(--vcs-text-muted)]" />
                  <span className="text-sm text-[var(--vcs-text-secondary)]">
                    {setting.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* Settings Content */}
      <div className="flex-1">
        <Tabs defaultValue="general" className="h-full">
          <TabsList className="w-full justify-start border-b border-[var(--vcs-border)] bg-transparent h-auto p-0 rounded-none">
            {settings.map((setting) => (
              <TabsTrigger
                key={setting.id}
                value={setting.id}
                className="rounded-none border-b-2 border-transparent data-[state=active]:border-[var(--vcs-blue)] data-[state=active]:bg-transparent"
              >
                {setting.label}
              </TabsTrigger>
            ))}
          </TabsList>
          
          {settings.map((setting) => (
            <TabsContent key={setting.id} value={setting.id} className="mt-0">
              <ScrollArea className="h-[calc(100vh-180px)]">
                <div className="p-6">
                  {setting.content}
                </div>
              </ScrollArea>
            </TabsContent>
          ))}
        </Tabs>
      </div>
    </div>
  );
}