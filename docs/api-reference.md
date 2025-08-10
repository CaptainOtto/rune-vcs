# Rune JSON API Reference

Base URL (local embedded mode):

```
http://127.0.0.1:7421
```

---

## Status
**GET** `/v1/status`  
Returns repository status.

Example:
```bash
curl http://127.0.0.1:7421/v1/status | jq
```

---

## Commit
**POST** `/v1/commit`  
Request body:
```json
{
  "message": "Commit via API"
}
```

---

## Log
**GET** `/v1/log`  
Returns commit history.

---

## Branches
**GET** `/v1/branches`  
Lists branches in the repository.

---

## LFS
- **POST** `/v1/lfs/push` – Upload file chunks
- **GET** `/v1/lfs/pull` – Download file chunks

---

## Locks
- **POST** `/v1/locks/lock` – Lock a file
- **POST** `/v1/locks/unlock` – Unlock a file
- **GET** `/v1/locks` – List all current locks
