# Rune VCS Roadmap

## Current Status: v0.2.6 ✅
- **Tier 1 Legacy**: Core Git features (commit --amend, revert, blame, interactive add)
- **Tier 2 Legacy**: Advanced features (rename detection, LFS improvements, word-level diff, hooks)

---

## Release Plan: v0.2.6 → v0.4.0

### 🎯 **v0.3.0 - Workspace & WIP Management** (Target: Q4 2025)
**Focus: Monorepo support + safer development workflow**

#### Core Features:
- **Partial/Virtual Workspace** - Perforce client view / sparse checkout parity
- **Draft/Checkpoint Commits** - Changelist-style shelved snapshots (safer than stash)
- **Performance Guardrails** - Blob size / file count thresholds
- **Policy-as-Code** - Commit + branch rules; enforce conventional commits

#### Implementation Priority:
1. `v0.3.0-alpha.1` ✅ - Virtual workspace core (COMPLETED)
2. `v0.3.0-alpha.2` ✅ - Draft commits system (COMPLETED)
3. `v0.3.0-alpha.3` - Policy-as-code framework
4. `v0.3.0-rc.1` - Policy-as-code framework

---

### 🚀 **v0.4.0 - Intelligence & Collaboration** (Target: Q1 2026)
**Focus: Smart merges + change impact analysis**

#### Core Features:
- **Semantic Merge & Conflict Assist** - Structured TOML/JSON/Cargo.toml support
- **Structured Changelog Generation** - Leverages enforced commit messages
- **Impact-based Test Selection** - Fast feedback loop for tight iteration
- **Intelligent Change Graph** - Dependency + crate impact visibility
- **Supply Chain Diff Scan** - Manifest change risk analysis

#### Implementation Priority:
1. `v0.4.0-alpha.1` - Semantic merge foundation
2. `v0.4.0-alpha.2` - Change graph analysis
3. `v0.4.0-beta.1` - Test selection intelligence
4. `v0.4.0-rc.1` - Supply chain scanning

---

## Future Releases (Post v0.4.0)

### **v0.5.0 - Storage & Security**
- Pluggable storage backend abstraction
- Encrypted path subsets
- Multi-author structured attribution

### **v0.6.0 - Scaling & Review**
- Merge queue (policy + auto-rebase + test gate)
- Inline persistent review annotations
- Background predictive prefetch

### **v0.7.0 - Advanced Collaboration**
- AI-assisted conflict resolution
- Resumable QUIC object streaming
- Offline collaboration bundles

---

## Tier Classification

### 🔥 **Tier 1 (Must-have: Core parity + workflow safety)**
1. ✅ Partial/virtual workspace (Monorepo virtual subroots)
2. ✅ Draft/checkpoint commits (changelist-style shelved snapshots)
3. ✅ Semantic merge & conflict assist (structured TOML/JSON/Cargo.toml)
4. ✅ Performance guardrails (blob size/file count thresholds)
5. ✅ Policy-as-code (commit + branch rules; enforce conventional commits)
6. ✅ Structured changelog generation
7. ✅ Impact-based test selection
8. ✅ Intelligent change graph
9. ✅ Supply chain diff scan
10. ⏳ Pluggable storage backend abstraction
11. ⏳ Encrypted path subsets

### 🚀 **Tier 2 (Important next: scaling, collaboration, security)**
1. Merge queue (policy + auto-rebase + test gate)
2. Inline persistent review annotations
3. Multi-author structured attribution
4. Background predictive prefetch
5. Sandbox/TTL branches
6. Provenance & build/test attestations
7. AI-assisted conflict resolution
8. Resumable QUIC object streaming
9. Offline collaboration bundles
10. Encrypted selective history redaction

### 🔬 **Tier 3 (Long-tail: advanced optimization)**
1. CRDT live edit layer
2. Automated hotfix propagation
3. Binary regression & similarity clustering
4. Data residency–aware replication
5. Binary dedupe clustering
6. Telemetry privacy budget & redaction
7. Transactional multi-repo atomic commits
8. Auto backport suggestion engine

---

## Implementation Strategy

**Cut line to replace Git + Perforce**: Complete all Tier 1 features.

**Tier 2** becomes necessary as:
- Team size grows
- Release cadence increases  
- Compliance needs rise

**Tier 3** deferred until:
- Scale requirements emerge
- Regulatory drivers appear

---

## Next Steps

Starting with **v0.3.0-alpha.1**: Partial/Virtual Workspace implementation
- Sparse checkout mechanisms
- Virtual root management
- Monorepo path filtering
- Performance optimization for large repos
