# NSV_ENV_001 — Build fails without git submodule initialization

**Severity:** Low  
**Component:** Environment/Setup  
**Date:** 2025-01-16

## Description
The project fails to build when git submodules are not initialized, causing a confusing error about missing `ark-ff` dependencies. The error message doesn't clearly indicate that submodules need to be initialized.

## Impact
- **Blocks development setup** for new contributors
- **Confusing error message** doesn't point to root cause
- **Documentation gap** - CONTRIBUTE.md mentions `--recurse-submodules` for clone but not for existing repos
- **Developer friction** during onboarding

## Likelihood
High - Any developer who clones without `--recurse-submodules` will encounter this.

## Preconditions / Assumptions
- Repository cloned without `--recurse-submodules` flag
- Attempting to build via `just build-ultrahonk` or similar commands

## Steps to Reproduce
1. Clone repository: `git clone https://github.com/wakeuplabs-io/noir-stylus-verifier.git`
2. Attempt to build: `just build-ultrahonk`
3. Expected: Clear guidance on missing submodules
4. Observed: Confusing error about missing `ark-ff` dependency

## Evidence
- Error log: Build fails with "No such file or directory" for `/vendor/algebra/ff/Cargo.toml`
- Missing directory: `vendor/algebra/` is empty without submodule init
- Documentation: CONTRIBUTE.md shows clone command but not submodule init for existing repos

## Affected Scope
- **Build system**: All `just` commands that require ultrahonk
- **Developer onboarding**: First-time setup experience
- **Documentation**: Missing troubleshooting guidance

## Recommendation
1. **Improve error handling**: Check for submodule presence before build attempts
2. **Update documentation**: Add troubleshooting section for submodule issues
3. **Add build script validation**: Pre-flight checks in justfile
4. **Consider alternatives**: Bundle dependencies or use git hooks

**Immediate fix for existing setup:**
```bash
git submodule update --init --recursive
```

**Documentation addition needed:**
```markdown
## Troubleshooting

### Build fails with "No such file or directory" for vendor/algebra/
This indicates git submodules aren't initialized. Run:
```bash
git submodule update --init --recursive
```

## References / Notes
- Related to developer experience and onboarding
- Could cause abandonment during initial setup
- Error message could be more helpful