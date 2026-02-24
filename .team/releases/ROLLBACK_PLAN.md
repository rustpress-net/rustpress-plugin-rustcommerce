# Rollback Plan — RustCommerce v0.1.0-design

**Release Version**: v0.1.0-design
**Release Type**: Design Phase (Planning & Architecture)
**Date**: 2026-02-24
**Prepared By**: Release Manager Agent

---

## 1. Overview

This rollback plan covers the procedures for reverting the RustCommerce project from the v0.1.0-design release state back to the pre-design state if required. Since this is a design phase release containing only documentation artifacts (no production code, no database changes, no deployed services), the rollback is low-risk and straightforward.

### 1.1 When to Execute Rollback

Rollback should be considered if:
- A fundamental architectural flaw is discovered that invalidates the design
- The RustPress core platform changes direction in a way that makes the RustCommerce design incompatible
- The project is cancelled or indefinitely postponed
- A legal or compliance issue is identified that requires redesign from scratch
- Stakeholders reject the design phase deliverables

### 1.2 Rollback Risk Assessment

| Factor | Assessment |
|--------|-----------|
| **Risk Level** | LOW |
| **Data Loss Risk** | NONE (no production data) |
| **Service Disruption** | NONE (no deployed services) |
| **User Impact** | NONE (no end users) |
| **Time to Rollback** | < 30 minutes |

---

## 2. Git-Based Rollback Procedures

### 2.1 Identify the Rollback Target

Determine which state to revert to:

```bash
# List recent tags to find the pre-design state
git tag --list 'v0.*' --sort=-version:refname

# View commit history to identify the pre-design commit
git log --oneline --graph --all --decorate | head -30

# The pre-design state is the commit immediately before the first
# .team/ directory addition
git log --oneline -- .team/ | tail -1
```

### 2.2 Option A: Revert via Git Revert (Recommended)

This option preserves the full history and creates a new commit that undoes the design phase changes. This is the recommended approach as it maintains an audit trail.

```bash
# Identify all commits that added design artifacts
git log --oneline --all -- .team/

# Create a revert branch
git checkout -b rollback/v0.1.0-design

# Revert the design phase commits (in reverse chronological order)
# Replace COMMIT_HASH with actual commit SHAs
git revert COMMIT_HASH_NEWEST --no-edit
git revert COMMIT_HASH_NEXT --no-edit
# ... repeat for each design phase commit

# Or if design was a single merge commit:
git revert MERGE_COMMIT_HASH -m 1 --no-edit

# Push the rollback branch
git push origin rollback/v0.1.0-design

# Create a pull request for the rollback
gh pr create --title "Rollback v0.1.0-design" \
  --body "Reverting design phase artifacts. Reason: [SPECIFY REASON]"
```

### 2.3 Option B: Revert via Branch Reset (Aggressive)

This option removes the design phase commits entirely from the branch history. Only use this if the design artifacts must be completely removed from the repository history (e.g., confidential information was accidentally included).

```bash
# WARNING: This rewrites history. Only use on branches that have not
# been shared or merged into protected branches.

# Create a backup branch first
git branch backup/pre-rollback-$(date +%Y%m%d)

# Reset to the pre-design commit
git reset --hard PRE_DESIGN_COMMIT_HASH

# Force push (requires admin permissions)
git push --force-with-lease origin main
```

### 2.4 Option C: Selective Artifact Removal

If only specific artifacts need to be removed (e.g., one design document needs rework), use selective deletion:

```bash
# Create a cleanup branch
git checkout -b cleanup/v0.1.0-design-partial

# Remove specific files or directories
git rm .team/api-contracts/API_DESIGN.md
# or remove entire category
git rm -r .team/api-contracts/

# Commit the removal
git commit -m "Remove [SPECIFIC ARTIFACT] for rework. Reason: [SPECIFY]"

# Push and create PR
git push origin cleanup/v0.1.0-design-partial
```

### 2.5 Tag Management

```bash
# Delete the release tag locally
git tag -d v0.1.0-design

# Delete the release tag from remote
git push origin --delete v0.1.0-design

# If a new design version will be created later, use a new tag
# e.g., v0.1.1-design or v0.2.0-design
```

---

## 3. GitHub Release Cleanup

### 3.1 Delete the GitHub Release

```bash
# Delete the GitHub release (keeps the tag unless separately deleted)
gh release delete v0.1.0-design --yes

# Confirm deletion
gh release list
```

### 3.2 GitHub Issue Cleanup

If the rollback requires resetting the project state:

```bash
# Reopen closed design-phase issues (M1-M3 issues were closed as design-complete)
for issue in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 31; do
  gh issue reopen $issue --comment "Reopened due to v0.1.0-design rollback. Reason: [SPECIFY]"
done

# Add a rollback label for tracking
gh label create "rollback" --color "B60205" --description "Affected by rollback"

# Label all affected issues
for issue in $(seq 1 31); do
  gh issue edit $issue --add-label "rollback"
done
```

### 3.3 GitHub Milestone Cleanup

If milestones need to be reset:

```bash
# Milestones do not need to be deleted unless the project structure changes.
# If project is cancelled:
gh api repos/{owner}/{repo}/milestones/{milestone_number} \
  -X PATCH -f state="closed" -f description="Closed due to project rollback"
```

---

## 4. File System Cleanup

If the design artifacts need to be removed from the working directory:

```bash
# Remove all design artifacts
rm -rf .team/api-contracts/
rm -rf .team/frontend/
rm -rf .team/devops/
rm -rf .team/infrastructure/
rm -rf .team/marketing/
rm -rf .team/legal/
rm -rf .team/qa/
rm -rf .team/releases/
rm -rf .team/reports/

# Optionally remove planning artifacts
rm -f .team/PROJECT_CHARTER.md
rm -f .team/MILESTONES.md
rm -f .team/KANBAN.md
rm -f .team/TIMELINE.md
rm -f .team/RISK_REGISTER.md
rm -f .team/GITHUB_ISSUES.md
rm -f .team/TEAM_STATUS.md

# Commit the cleanup
git add -A
git commit -m "Remove v0.1.0-design artifacts per rollback plan"
```

---

## 5. Communication Plan

### 5.1 Immediate Notification (Within 1 Hour of Rollback Decision)

| Audience | Channel | Message |
|----------|---------|---------|
| All team members (PM, BE, FE, DevOps, Infra, QA, Marketing, Legal) | Team channel / email | "The v0.1.0-design release has been rolled back. Reason: [SPECIFY]. All design phase work is being reverted. Stand by for further instructions." |
| RustPress Core Team (Project Sponsor) | Direct message | "RustCommerce design phase rollback in progress. Reason: [SPECIFY]. Impact assessment to follow within 24 hours." |

### 5.2 Follow-Up Communication (Within 24 Hours)

| Item | Content |
|------|---------|
| **Root Cause** | Detailed explanation of why the rollback was necessary |
| **Impact Assessment** | What work is affected and what can be salvaged |
| **Path Forward** | Whether the project will restart design, pivot, or be cancelled |
| **Timeline** | Estimated timeline for next steps |
| **Action Items** | Specific tasks assigned to each team member |

### 5.3 Communication Template

```
Subject: RustCommerce v0.1.0-design Rollback Notification

Team,

The v0.1.0-design release of the RustCommerce project has been rolled back
effective [DATE/TIME].

REASON: [Detailed explanation]

IMPACT:
- All design phase artifacts in .team/ have been [reverted/removed]
- GitHub issues #1-#31 have been [reopened/relabeled]
- The v0.1.0-design tag and GitHub release have been [deleted]

WHAT IS PRESERVED:
- Wave 0 skeleton code (src/, Cargo.toml, plugin.json)
- Repository structure and configuration
- Git history (all changes are traceable)

NEXT STEPS:
1. [Specific action items]
2. [Timeline for restart if applicable]
3. [Meeting/sync scheduled for DATE]

Questions? Reach out to [Release Manager / Project Manager].

— Release Manager
```

---

## 6. Verification After Rollback

After executing the rollback, verify the following:

| # | Check | Command / Action | Expected Result |
|---|-------|-----------------|-----------------|
| 1 | Tag removed | `git tag --list 'v0.1.0-design'` | No output |
| 2 | GitHub release removed | `gh release view v0.1.0-design` | "release not found" error |
| 3 | Design artifacts removed | `ls .team/api-contracts/ 2>/dev/null` | "No such file or directory" |
| 4 | Issues reopened (if applicable) | `gh issue list --state open --milestone "M1: Backend Foundation"` | Issues are open |
| 5 | Repository builds cleanly | `cargo check` | Success |
| 6 | No orphaned references | `grep -r "v0.1.0-design" .` | No matches (or only in rollback docs) |

---

## 7. Lessons Learned Template

After rollback, document lessons learned:

```markdown
## Rollback Lessons Learned — v0.1.0-design

**Date of Rollback**: [DATE]
**Reason**: [REASON]
**Decision Made By**: [NAME/ROLE]

### What Went Wrong
- [Description]

### What Could Have Prevented It
- [Description]

### What We Will Do Differently
- [Description]

### Salvageable Work
- [List any artifacts that can be reused in a future design iteration]
```

---

*This rollback plan is a contingency document. Under normal circumstances, no rollback is expected for the design phase release. The plan exists to ensure preparedness for unexpected scenarios.*
