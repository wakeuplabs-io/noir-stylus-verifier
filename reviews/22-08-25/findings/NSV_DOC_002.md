# NSV_DOC_002 — Battleship README contains incorrect documentation path

**Severity:** Low  
**Component:** Documentation/Examples  
**Date:** 2025-01-16

## Summary
The battleship example README references a non-existent documentation path. The actual tutorial exists but in a different location with different filename.

## Description
The battleship example README.md contains a broken link to the tutorial documentation, making it impossible for users to find the detailed guide.

## Current vs Actual Paths

**README Link (broken):**
```markdown
[docs/tutorials/building-a-battleship-game](../../docs/tutorials/building-a-battleship-game.md)
```

**Actual File Location:**
```
docs/contents/docs/guides/building-a-battleship-game/index.mdx
```

**Multiple Errors in Path:**
1. **Directory**: `tutorials` should be `guides`
2. **Missing subdirectory**: Should include `/contents/docs/`  
3. **File extension**: `.md` should be `.mdx`
4. **Filename**: `building-a-battleship-game.md` should be `index.mdx`

## Impact
- **Broken user journey**: Users cannot find the detailed tutorial
- **404 errors**: Link leads to non-existent file
- **Poor documentation experience**: Users must manually search for the guide
- **Inconsistent path structure**: Doesn't match actual docs organization

## File Locations
- **README**: `/examples/battleship/README.md` line 5
- **Actual tutorial**: `/docs/contents/docs/guides/building-a-battleship-game/index.mdx`

## Evidence
```bash
# Broken path from README
/docs/tutorials/building-a-battleship-game.md  ❌ Does not exist

# Actual working path  
/docs/contents/docs/guides/building-a-battleship-game/index.mdx  ✅ Exists (14.5KB file)
```

## Recommended Fix
Update the README.md link to point to the correct location:

**Current (broken):**
```markdown
There's a guide for this example at [docs/tutorials/building-a-battleship-game](../../docs/tutorials/building-a-battleship-game.md). Check it out to know more about the project.
```

**Fixed:**
```markdown
There's a guide for this example at [docs/guides/building-a-battleship-game](../../docs/contents/docs/guides/building-a-battleship-game/index.mdx). Check it out to know more about the project.
```

**Alternative (if docs are published):**
```markdown
There's a guide for this example in our documentation at [Building a Battleship Game](https://docs.nsv.wakeuplabs.io/docs/guides/building-a-battleship-game). Check it out to know more about the project.
```

## Relative Path Calculation
From `/examples/battleship/README.md` to target file:
```bash
../../docs/contents/docs/guides/building-a-battleship-game/index.mdx
```

**Verification:**
- `../..` goes from `/examples/battleship/` to repository root
- `/docs/contents/docs/guides/building-a-battleship-game/index.mdx` is the target

## Severity Justification
**LOW** because:
- Documentation exists, just wrong path reference
- Users can find the tutorial through other means
- No functional impact on code
- Simple documentation fix required

## Testing
Verified by filesystem check:
```bash
# Current broken path
ls /Users/francoperez/repos/wakeup/nsv/noir-stylus-verifier/docs/tutorials/
# ls: docs/tutorials/: No such file or directory

# Actual working path  
ls /Users/francoperez/repos/wakeup/nsv/noir-stylus-verifier/docs/contents/docs/guides/building-a-battleship-game/
# index.mdx
```

## Documentation Architecture Note
This suggests the docs follow a structure like:
```
docs/
├── contents/docs/
│   ├── getting-started/
│   └── guides/
│       ├── building-a-battleship-game/
│       └── building-a-voting-app/
```

Rather than the implied structure:
```
docs/
├── tutorials/  ❌ (doesn't exist)
```

## References / Notes
- Located in battleship example at `/examples/battleship/README.md`
- Tutorial content exists and is substantial (14.5KB)
- Likely affects user onboarding for battleship example
- Similar pattern should be checked in other example READMEs