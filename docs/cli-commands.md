# Rune CLI Commands

## Repository Commands
| Command                     | Description |
|-----------------------------|-------------|
| `rune init`                 | Create a new repository in the current directory |
| `rune status`               | Show changed, staged, and untracked files |
| `rune add <file>`           | Stage file(s) for commit |
| `rune commit -m "msg"`      | Commit staged changes |
| `rune log`                  | Show commit history |
| `rune branch`               | List or create branches |
| `rune checkout <branch>`    | Switch to another branch |
| `rune stash`                | Stash current changes |

## Large File Support (LFS)
| Command                                  | Description |
|------------------------------------------|-------------|
| `rune lfs track "<pattern>"`             | Track file pattern for LFS storage |
| `rune lfs untrack "<pattern>"`           | Remove pattern from LFS tracking |
| `rune lfs push <file>`                   | Upload large file chunks to remote Shrine |
| `rune lfs pull <file>`                   | Download large file chunks from remote Shrine |
| `rune lfs lock --path <file>`            | Lock a file for exclusive editing |
| `rune lfs unlock --path <file>`          | Unlock a file |

## Server/API
| Command                                            | Description |
|----------------------------------------------------|-------------|
| `rune api --addr <host:port>`                      | Start API server only |
| `rune api --with-shrine --shrine-addr <host:port>` | Start API + Shrine in one process |
| `rune shrine serve --addr <host:port>`             | Start Shrine server separately |

## Utilities
| Command                     | Description |
|-----------------------------|-------------|
| `rune help`                 | Show help for commands |
| `rune completion <shell>`   | Output shell completions |
