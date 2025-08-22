# NSV_CLI_002 — nsv new accepts invalid package names that later fail in nargo

**Severity:** Medium  
**Component:** CLI/Validation  
**Date:** 2025-01-16

## Description
The `nsv new` command accepts package names with hyphens (e.g., "invalid-name-with-hyphens") that are invalid for Noir/nargo, causing the project creation to succeed but subsequent operations to fail with confusing error messages.

## Impact
- **User frustration**: Project appears to be created successfully but fails later
- **Confusing error messages**: Nargo error doesn't clearly indicate the CLI should have prevented this
- **Time waste**: Users must recreate projects with valid names
- **Poor UX**: Success message followed by failure creates bad developer experience

## Likelihood
Medium - Users coming from other ecosystems may naturally try hyphenated names.

## Preconditions / Assumptions
- User runs `nsv new` with hyphenated package name
- Noir/nargo has stricter naming requirements than NSV validation

## Steps to Reproduce
1. Run `nsv new invalid-name-with-hyphens`
2. Observe: ✅ Success message displayed
3. Navigate to project: `cd invalid-name-with-hyphens`
4. Try to use project: `nargo execute`
5. Expected: Project should work OR creation should have failed
6. Observed: "Invalid package name" error from nargo

## Evidence
- NSV allows: "invalid-name-with-hyphens" ✅ (accepted)
- Nargo rejects: "Invalid package name `invalid-name-with-hyphens`" ❌
- NSV correctly rejects: "" (empty) and "test@#$%" (special chars)

## Affected Scope
- `nsv new` command validation logic
- User onboarding experience
- Integration with nargo/Noir ecosystem

## Recommendation
**Align NSV validation with nargo requirements:**

1. **Investigate nargo naming rules**: Document exact regex/rules nargo uses
2. **Update NSV validation**: Match nargo's stricter validation
3. **Improve error message**: Explain naming requirements clearly

**Example improved validation:**
```
Error: Invalid project name 'invalid-name-with-hyphens'

Package names must:
- Contain only letters, numbers, and underscores  
- Start with a letter or underscore
- Match nargo requirements

Try: 'invalid_name_with_underscores' or 'invalidname'
```

**Investigation needed:**
- What are the exact nargo naming requirements?
- Are there other naming conflicts (Rust crate names, filesystem, etc.)?

## References / Notes
- Classic validation bypass issue
- Should validate against downstream tool requirements
- Related to supply chain - names might affect package publishing later