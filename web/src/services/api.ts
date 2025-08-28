// API service for Rune VCS backend integration

export interface FileChange {
  id: string;
  name: string;
  path: string;
  status: 'added' | 'modified' | 'deleted';
  staged: boolean;
  additions?: number;
  deletions?: number;
  changelist?: string;
}

export interface Changelist {
  id: string;
  name: string;
  description: string;
  files: FileChange[];
}

export interface Commit {
  id: string;
  message: string;
  author: {
    name: string;
    email: string;
  };
  date: number;
  branch: string;
  parents: string[];
}

export interface Repository {
  name: string;
  branch: string;
  url: string;
  lastSync?: string;
}

export interface FileNode {
  id: string;
  name: string;
  type: 'file' | 'folder';
  children?: FileNode[];
  status?: string;
}

export interface Branch {
  name: string;
  current: boolean;
  upstream?: string;
  ahead: number;
  behind: number;
  lastCommit: {
    id: string;
    message: string;
    date: number;
  };
}

class ApiService {
  private baseUrl = '';

  async getRepository(): Promise<Repository> {
    const response = await fetch(`${this.baseUrl}/v1/repository`);
    return response.json();
  }

  async getStatus(): Promise<{ branch: string; staged: string[] }> {
    const response = await fetch(`${this.baseUrl}/v1/status`);
    return response.json();
  }

  async getChanges(): Promise<{ changelists: Changelist[]; unstagedFiles: FileChange[] }> {
    const response = await fetch(`${this.baseUrl}/v1/changes`);
    return response.json();
  }

  async getHistory(): Promise<{ commits: Commit[] }> {
    const response = await fetch(`${this.baseUrl}/v1/history`);
    return response.json();
  }

  async getBranches(): Promise<Branch[]> {
    const response = await fetch(`${this.baseUrl}/v1/branches`);
    const data = await response.json();
    // Transform the simple branch list to our expected format
    return data.map((branch: { name: string }) => ({
      name: branch.name,
      current: false, // We'll need to determine this from status
      upstream: undefined,
      ahead: 0,
      behind: 0,
      lastCommit: {
        id: '',
        message: '',
        date: Date.now()
      }
    }));
  }

  async getFileTree(): Promise<FileNode> {
    const response = await fetch(`${this.baseUrl}/v1/file-tree`);
    return response.json();
  }

  async commit(message: string, name?: string, email?: string): Promise<{ id: string; message: string }> {
    const response = await fetch(`${this.baseUrl}/v1/commit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ message, name, email })
    });
    return response.json();
  }

  async stage(paths: string[]): Promise<{ ok: boolean }> {
    const response = await fetch(`${this.baseUrl}/v1/stage`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ paths })
    });
    return response.json();
  }

  async createBranch(name: string): Promise<{ created: string }> {
    const response = await fetch(`${this.baseUrl}/v1/branch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name })
    });
    return response.json();
  }

  async checkout(name: string): Promise<{ switched?: string; error?: string }> {
    const response = await fetch(`${this.baseUrl}/v1/checkout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name })
    });
    return response.json();
  }
}

export const api = new ApiService();
