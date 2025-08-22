# NSV Verification Compatibility Matrix

**Audit Target**: [Repository commit 0ca9afe](https://github.com/wakeuplabs-io/noir-stylus-verifier/commit/0ca9afefba0259bcf2b175bd868f9d2eddf45231)  
**Fix Status**: [All findings addressed in commit deb0382](https://github.com/wakeuplabs-io/noir-stylus-verifier/commit/deb0382febe5f0a344ae7de3395085a674ded875)

## Overview
This document provides the complete compatibility matrix for NSV proof generation and verification methods. Essential for understanding which proof/verifier combinations work correctly.

## Proof Generation Methods

### **Normal Proofs**
- **Command**: `nsv prove`
- **Flavor**: UltraKeccakFlavor  
- **Use Case**: Standard verification workflow
- **Output**: Standard proof format

### **ZK Proofs**  
- **Command**: `nsv prove --zk`
- **Flavor**: UltraHonkFlavor
- **Use Case**: Zero-knowledge verification
- **Output**: ZK-optimized proof format

## Verification Methods

### **Off-chain Verification**
- **Command**: `nsv verify`
- **Location**: Local verification
- **Speed**: Fast
- **Cost**: Free

### **On-chain Normal Verification**
```bash
nsv verify \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --verifier-address <CONTRACT_ADDRESS>
```

### **On-chain ZK Verification**
```bash
nsv verify \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --verifier-address <CONTRACT_ADDRESS> \
  --zk
```

## Complete Compatibility Truth Table

| Test# | Proof Type | Verifier Type | Verifier Location | Expected Result | Status | Rationale |
|-------|------------|---------------|-------------------|-----------------|---------|-----------| 
| 1 | Normal | Normal | Off-chain | ✅ PASS | Verified | Compatible flavors |
| 2 | Normal | Normal | On-chain | ✅ PASS | Verified | Compatible flavors |
| 3 | Normal | ZK | Off-chain | ❌ FAIL | Verified | Incompatible flavors |
| 4 | Normal | ZK | On-chain | ❌ FAIL | Verified | Incompatible flavors |
| 5 | ZK | Normal | Off-chain | ❌ FAIL | Verified | Incompatible flavors |
| 6 | ZK | Normal | On-chain | ❌ FAIL | Verified | Incompatible flavors |
| 7 | ZK | ZK | Off-chain | ✅ PASS | Verified | Compatible flavors |
| 8 | ZK | ZK | On-chain | ✅ PASS | Verified | Compatible flavors |

## Key Compatibility Rules

### **✅ Compatible Combinations**
1. **Normal → Normal**: Any location (off-chain, on-chain normal)
2. **ZK → ZK**: Any location (off-chain, on-chain ZK)

### **❌ Incompatible Combinations**  
1. **Normal → ZK**: Any location (flavor mismatch)
2. **ZK → Normal**: Any location (flavor mismatch)

### **🔄 Cross-Flavor Testing**
- All cross-flavor combinations (Normal proof + ZK verifier, ZK proof + Normal verifier) correctly fail
- This is expected behavior, not a security issue
- Flavor compatibility is enforced at the cryptographic level

## Network-Specific Information

### **Arbitrum Sepolia Testnet**
- **RPC URL**: `https://sepolia-rollup.arbitrum.io/rpc`
- **Chain ID**: 421614

### **Local Development**
- **Off-chain verification**: Always available
- **No network dependency**: Local verification only
- **Testing**: Recommended for development

## Practical Usage Examples

### **Standard Workflow (Normal)**
```bash
# 1. Generate normal proof
nsv prove --circuit-path circuits/hello_world

# 2. Verify off-chain
nsv verify --proof-path proof.bin --vk-path vk.bin
# Result: ✅ PASS

# 3. Verify on-chain (Sepolia)
nsv verify \
  --proof-path proof.bin \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --verifier-address <CONTRACT_ADDRESS>
# Result: ✅ PASS
```

### **ZK Workflow**
```bash
# 1. Generate ZK proof  
nsv prove --circuit-path circuits/hello_world --zk

# 2. Verify off-chain
nsv verify --proof-path proof.bin --vk-path vk.bin --zk
# Result: ✅ PASS

# 3. Verify on-chain ZK (Sepolia)
nsv verify \
  --proof-path proof.bin \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --verifier-address <CONTRACT_ADDRESS> \
  --zk
# Result: ✅ PASS
```

### **Cross-Flavor Testing (Expected Failures)**
```bash
# Generate normal proof
nsv prove --circuit-path circuits/hello_world

# Try to verify with ZK verifier (should fail)
nsv verify \
  --proof-path proof.bin \
  --rpc-url https://sepolia-rollup.arbitrum.io/rpc \
  --verifier-address <CONTRACT_ADDRESS> \
  --zk
# Result: ❌ FAIL (Expected - flavor mismatch)
```

## Security Implications

### **Compatibility Security**
- **Flavor Enforcement**: Cryptographic incompatibility prevents cross-flavor attacks
- **Network Isolation**: Off-chain and on-chain verification use same cryptographic validation  
- **Verifier Addresses**: Each flavor has dedicated on-chain verifier contract

### **Testing Validation**
- **Positive Tests**: Compatible combinations must pass
- **Negative Tests**: Incompatible combinations must fail  
- **Security**: Any unexpected passes indicate potential vulnerability

## Troubleshooting Common Issues

### **"Verification Failed" with Compatible Flavor**
- Check proof and VK file paths
- Verify public inputs match circuit expectations
- Ensure circuit compilation succeeded

### **"Invalid Verifier Address"** 
- Confirm correct network (Sepolia testnet)
- Check verifier address matches flavor (Normal vs ZK)
- Verify RPC URL accessibility

### **"Network Connection Failed"**
- Test RPC URL accessibility: `curl https://sepolia-rollup.arbitrum.io/rpc`
- Check internet connectivity
- Try off-chain verification first

## Testing Matrix for Security Validation

| Scenario | Proof | Verifier | Location | Expected | Test Command |
|----------|-------|----------|----------|----------|--------------|
| Baseline Normal | Normal | Normal | Off-chain | ✅ | `nsv verify --proof proof.bin --vk vk.bin` |
| Baseline ZK | ZK | ZK | Off-chain | ✅ | `nsv verify --proof proof.bin --vk vk.bin --zk` |
| Cross-flavor 1 | Normal | ZK | Off-chain | ❌ | `nsv verify --proof normal.bin --vk zk_vk.bin --zk` |
| Cross-flavor 2 | ZK | Normal | Off-chain | ❌ | `nsv verify --proof zk.bin --vk normal_vk.bin` |

**Reference**: See `pocs/nsv_security_check.sh` for complete testing implementation.

---

**Next Steps**: Use this compatibility matrix to validate proper NSV behavior and identify any unexpected verification results that might indicate security issues.