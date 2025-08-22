# NSV_EXM_001 — Voting example requires Pinata IPFS credentials without documentation

**Severity:** Low  
**Component:** Examples/Integration  
**Status:** Confirmed  
**Date:** 2025-01-16  

## Summary
The voting example fails during proposal creation due to missing Pinata IPFS configuration, but the README doesn't document this requirement or provide setup instructions.

## Description
When attempting to create a proposal in the voting example web app, users encounter a CORS error that prevents proposal creation. The actual issue is missing Pinata IPFS API credentials, but this is not documented anywhere.

## Technical Details

**Error Observed:**
```
Access to fetch at 'https://uploads.pinata.cloud/v3/files' from origin 'http://localhost:5173' 
has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.

PinataError: Error processing json: Failed to fetch
    at async PinataIpfs.uploadJSON (ipfs.ts:19:20)
    at async VotingContract.preparePropose (contract.ts:52:25)
```

**Root Cause:** Missing Pinata IPFS credentials in environment variables.

**Required but Undocumented Variables:**
```bash
# In apps/www/.env
VITE_IPFS_PINATA_JWT="your-pinata-jwt-token"     # Currently: "..."
VITE_IPFS_GATEWAY_URL="your-pinata-gateway"      # Currently: "..."
```

## Impact
- **Broken example functionality**: Proposal creation completely fails
- **Poor user experience**: Misleading CORS error instead of configuration error
- **Missing documentation**: No guidance on obtaining required credentials
- **Development friction**: Users cannot complete the voting example workflow
- **Debugging difficulty**: Error message doesn't indicate the real problem

## Reproduction Steps
1. Follow voting example setup in README
2. Start web app: `pnpm --filter=@voting/www dev`
3. Create ZK account successfully ✅
4. Attempt to create a proposal ❌
5. Observe CORS error in browser console

## Expected vs Actual Behavior
- **Expected**: Clear error message about missing Pinata credentials
- **Actual**: Misleading CORS error that suggests network/server issue

## Code Analysis
**File**: `/packages/core/src/infrastructure/ipfs.ts`
```typescript
async uploadJSON(json: Object): Promise<string> {
  const result = await this.client.upload.public.json(json);  // Fails here
  return result.cid;
}
```

**Issue**: No error handling for invalid credentials; relies on Pinata SDK error messages.

## Documentation Gap
**Current README** mentions:
```markdown
Lastly copy .env.example to .env and replace with your values for both www and cli
```

**Missing information:**
- What Pinata IPFS is and why it's needed
- How to create Pinata account and get JWT
- What the gateway URL should be
- Alternative IPFS providers (if any)

## Recommended Remediation

### 1. Update README.md
Add section explaining Pinata setup:

```markdown
### IPFS Configuration (Required)

The voting app uses Pinata IPFS to store proposal metadata. You need to:

1. Create a free account at [pinata.cloud](https://pinata.cloud)
2. Generate an API JWT token in your Pinata dashboard
3. Copy your dedicated gateway URL

Update your .env files:
```bash
# apps/www/.env
VITE_IPFS_PINATA_JWT="your-jwt-token-here"
VITE_IPFS_GATEWAY_URL="https://your-gateway.mypinata.cloud"

# apps/cli/.env  
IPFS_PINATA_JWT="your-jwt-token-here"
IPFS_GATEWAY_URL="https://your-gateway.mypinata.cloud"
```

### 2. Improve Error Handling
In `ipfs.ts`:
```typescript
async uploadJSON(json: Object): Promise<string> {
  try {
    const result = await this.client.upload.public.json(json);
    return result.cid;
  } catch (error) {
    if (error.message.includes('401') || error.message.includes('authentication')) {
      throw new Error('Invalid Pinata JWT token. Please check your IPFS configuration.');
    }
    throw error;
  }
}
```

### 3. Add Configuration Validation
In `env.ts`:
```typescript
// Validate Pinata credentials format
if (IPFS_PINATA_JWT === "..." || !IPFS_PINATA_JWT.startsWith('eyJ')) {
  throw new Error('Please configure valid Pinata JWT token in .env file');
}
```

## Alternative Solutions
1. **Mock IPFS mode**: Add local/mock IPFS for development
2. **Different IPFS provider**: Use public IPFS nodes for examples  
3. **Better error handling**: Detect auth failures and show helpful message

## Severity Justification
**MEDIUM** because:
- Completely breaks example functionality
- Poor user experience with misleading errors
- Documentation gap affects all users
- Not a security issue, but significant usability problem

## Related Issues
- **NSV_DOC_001**: Documentation clarity issues in voting example
- Both relate to voting example setup experience

## References / Notes
- Affects voting example at `/examples/voting/`
- Pinata SDK version: 2.4.9 (from package.json)
- Error occurs in browser during proposal creation flow
- RESOLVED: Fixed in commit 0ca9afefba0259bcf2b175bd868f9d2eddf45231