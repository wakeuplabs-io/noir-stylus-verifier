# NSV_DOC_001 — Voting example README has unclear .env setup instructions

**Severity:** Low  
**Component:** Documentation/Examples  
**Status:** Confirmed  
**Date:** 2025-01-16

## Summary
The voting example README provides unclear instructions for .env file setup, not specifying the different file names used by CLI vs WWW components.

## Description
The README.md states: "Lastly copy `.env.example` to `.env` and replace with your values for both `www` and `cli`"

However, the actual file structure uses different naming conventions:
- **CLI**: `.env.example` → `.env` ✅
- **WWW**: `.env.local.example` → `.env.local` (not mentioned)

## Current vs Expected Documentation

**Current (ambiguous):**
```markdown
Lastly copy .env.example to .env and replace with your values for both www and cli
```

**Should be (clear):**
```markdown
Lastly copy .env files and replace with your values:
- CLI: cp apps/cli/.env.example apps/cli/.env
- WWW: cp apps/www/.env.local.example apps/www/.env.local
```

## Impact
- **User confusion**: Developers following instructions may not find `.env.example` in www directory
- **Setup friction**: Users must figure out the correct file names independently
- **Inconsistent experience**: Instructions don't match actual file structure

## Likelihood
High - Every developer following the voting example setup will encounter this ambiguity.

## File Locations
- **README**: `/examples/voting/README.md` line 42
- **CLI env**: `/examples/voting/apps/cli/.env.example`
- **WWW env**: `/examples/voting/apps/www/.env.local.example`

## Evidence
```bash
# Actual file structure
apps/cli/.env.example          ✅ Exists
apps/www/.env.example          ❌ Does not exist  
apps/www/.env.local.example    ✅ Exists (but not mentioned in README)
```

## Recommended Fix
Update README.md line 42 to be more specific:

```markdown
Lastly, set up environment files for both applications:

```bash
# Copy and configure CLI environment
cp apps/cli/.env.example apps/cli/.env
# Edit apps/cli/.env with your values

# Copy and configure WWW environment  
cp apps/www/.env.local.example apps/www/.env.local
# Edit apps/www/.env.local with your values
```
```

## Alternative Fix (Simpler)
```markdown
Lastly copy .env template files to active .env files and replace with your values:
- `apps/cli/.env.example` → `apps/cli/.env`
- `apps/www/.env.local.example` → `apps/www/.env.local`
```

## Severity Justification
**LOW** because:
- Does not affect functionality or security
- Workaround is straightforward (users can discover correct filenames)
- Only impacts initial setup experience
- No code changes required, only documentation

## Testing
Verified by checking file structure:
```bash
find /examples/voting -name ".env*" -type f
# Results:
# ./apps/cli/.env.example
# ./apps/www/.env.local.example  
# (Plus any existing .env files)
```

## References / Notes
- Located in voting example at `/examples/voting/README.md`
- Part of user onboarding experience
- Simple documentation improvement with immediate UX benefit