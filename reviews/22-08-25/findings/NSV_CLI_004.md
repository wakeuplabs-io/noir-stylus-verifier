# NSV_CLI_004 — CLI panics on malformed verifier address

**Severity:** Low  
**Component:** CLI/Error Handling  
**Status:** Confirmed  
**Discovered During:** Verification Truth Table Edge Case Testing
**Date:** 2025-01-16

## Summary
The `nsv verify` command panics with unwrap() when given malformed verifier addresses, causing application crash instead of graceful error handling.

## Description
When providing malformed verifier addresses (e.g., `0xinvalid`), NSV CLI panics instead of providing user-friendly error messages. This represents poor error handling pattern.

## Technical Details

**Panic Location**: `cli/src/commands/verify.rs:86:60`

**Error Message:**
```
thread 'main' panicked at cli/src/commands/verify.rs:86:60:
called `Result::unwrap()` on an `Err` value: OddLength
```

**Root Cause**: Hex address parsing fails due to odd-length string, but the error is not handled gracefully.

## Discovery Method
Found during comprehensive edge case testing as part of verification truth table validation (Test E5).

## Reproduction Steps
```bash
cd /Users/francoperez/repos/wakeup/nsv/noir-stylus-verifier/test_vectors/if_then
nsv verify --rpc-url https://sepolia-rollup.arbitrum.io/rpc --verifier-address 0xinvalid
```

**Expected**: Graceful error message about invalid address format  
**Actual**: Application panic with stack trace

## Impact Details
3. **Pattern repetition**: Similar to NSV_CLI_003 (same unwrap() anti-pattern)
4. **Input validation gap**: CLI doesn't validate address format before processing

## Error Scenarios
1. **User error**: Typos in addresses cause application crashes
2. **Automation breaking**: Scripts using NSV may fail unexpectedly
3. **CI/CD disruption**: Deployment scripts could be interrupted by panics

## Expected vs Actual Behavior
- **Expected**: "Error: Invalid verifier address format '0xinvalid'. Please provide a valid 20-byte hex address."
- **Actual**: Thread panic with technical stack trace

## Recommended Remediation
1. **Replace unwrap() with proper error handling**:
```rust
// Instead of:
address.parse().unwrap()

// Use:
address.parse().map_err(|e| AppError::InvalidAddress(format!("Invalid address format: {}", e)))?
```

2. **Add input validation**:
```rust
fn validate_verifier_address(address: &str) -> Result<(), AppError> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(AppError::InvalidAddress("Address must be 42 characters starting with 0x"));
    }
    // Additional validation...
    Ok(())
}
```

3. **Consistent error messages**: Provide user-friendly guidance for address format

## Related Findings
- **NSV_CLI_003**: Similar unwrap() pattern causing panics in `nsv generate`
- **NSV_VER_005**: unwrap() pattern in contract verification logic
- **Pattern**: This is part of a broader issue with error handling across NSV codebase

## Severity Justification
**LOW** because:
- Application interruption through invalid user input
- Affects CLI reliability and user experience
- Part of systemic error handling problems
- Easy to trigger accidentally

## Example Fix Implementation
```rust
fn parse_verifier_address(address_str: &str) -> Result<Address, AppError> {
    // Validate format first
    if !address_str.starts_with("0x") {
        return Err(AppError::InvalidAddress("Address must start with 0x".to_string()));
    }
    
    if address_str.len() != 42 {
        return Err(AppError::InvalidAddress("Address must be 42 characters long".to_string()));
    }
    
    // Parse with proper error handling
    address_str.parse().map_err(|e| {
        AppError::InvalidAddress(format!("Invalid hex format: {}", e))
    })
}
```

## Test Case
Should be included in CI/CD:
```bash
# Should fail gracefully, not panic
nsv verify --rpc-url https://sepolia-rollup.arbitrum.io/rpc --verifier-address 0xinvalid
nsv verify --rpc-url https://sepolia-rollup.arbitrum.io/rpc --verifier-address invalid
nsv verify --rpc-url https://sepolia-rollup.arbitrum.io/rpc --verifier-address 0x123
```

## References / Notes
- Discovered during verification truth table edge case testing
- Part of comprehensive CLI robustness evaluation
- Should be fixed alongside other unwrap() issues for consistent improvement