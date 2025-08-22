# NSV_CLI_001 — CLI onboarding only suggests on-chain workflow, omits off-chain testing

**Severity:** Low
**Component:** CLI/User Experience  
**Date:** 2025-01-16

## Description
When creating a new project with `nsv generate`, the CLI only suggests the on-chain deployment workflow in the "What's Next?" guidance, omitting the off-chain testing workflow which is typically safer and faster for development.

Current guidance suggests:
1. `nsv check` - Check Stylus compatibility  
2. `nsv deploy` - Deploy to blockchain

Missing guidance on off-chain workflow:
1. `nsv prove` - Generate proofs for testing
2. `nsv verify` (without args) - Verify proofs locally

## Impact
- **Developer confusion**: New users may not discover local testing capabilities
- **Suboptimal workflow**: Users might attempt costly on-chain deployment before local testing
- **Missing best practices**: No guidance toward "test locally first" pattern
- **Incomplete onboarding**: CLI doesn't showcase full feature set

## Likelihood
High - Every new user sees this guidance and may miss local testing workflow.

## Preconditions / Assumptions
- User runs `nsv generate`
- User follows the suggested "What's Next?" guidance
- User is new to NSV and relies on CLI prompts

## Steps to Reproduce
1. Run `nsv generate`
2. Observe "What's Next?" output
3. Expected: Guidance includes both on-chain and off-chain workflows
4. Observed: Only on-chain workflow mentioned

## Evidence
- CLI output shows only deployment-focused next steps
- `nsv prove` and `nsv verify` are not mentioned in onboarding
- Documentation pattern suggests on-chain first, not local testing first

## Affected Scope
- CLI output in `nsv new` command
- User onboarding experience
- Developer workflow guidance

## Recommendation
**Update CLI output to include both workflows:**

```
What's Next?

Test locally first (recommended):
  - nsv prove: Generate proofs for testing
  - nsv verify: Verify proofs locally

Then deploy to blockchain:
  - nsv generate: Generate a verifier contract from a noir circuit
  - nsv check: Check if the generated contract is compatible with Stylus
  - nsv deploy: Deploy the generated contract to the blockchain
```

**Alternative: Workflow-based guidance:**
```
What's Next?

Choose your workflow:
  - Learn: Follow examples/hello_world README
  - Test: nsv prove → nsv verify (local testing)
  - Deploy: nsv generate → nsv check → nsv deploy
```

## References / Notes
- Related to developer experience and best practices
- Could improve adoption by showing local testing capabilities upfront
- Aligns with development best practice of "test before deploy"