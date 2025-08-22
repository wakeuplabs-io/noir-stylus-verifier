# NSV Environment Setup Guide

**Audit Target**: [Repository commit 0ca9afe](https://github.com/wakeuplabs-io/noir-stylus-verifier/commit/0ca9afefba0259bcf2b175bd868f9d2eddf45231)  
**Fix Status**: [Findings addressed in commit deb0382](https://github.com/wakeuplabs-io/noir-stylus-verifier/commit/deb0382febe5f0a344ae7de3395085a674ded875)

## Overview
This guide provides essential environment configuration for NSV functionality testing and validation tool usage. Originally used during security testing.

## Network Configuration

### **Testnet Environment**
- **Network**: Arbitrum Sepolia
- **RPC URL**: `https://sepolia-rollup.arbitrum.io/rpc`
- **Chain ID**: 421614
- **Purpose**: Safe testing environment for verification

### **Local Development**
- **Network**: Nitro testnode (Docker-based)
- **Purpose**: Isolated environment for testing
- **Setup**: Use provided Docker configuration

## Tool Requirements

### **Core Dependencies**
- **Noir toolchain**: nargo, bb (version 1.0.0-beta.6+)
- **NSV CLI**: Latest version from repository
- **Python**: 3.8+ (for validation scripts)
- **Node.js**: For JavaScript verification scripts (optional)

### **Installation References**
- **Noir**: https://noir-lang.org/getting_started
- **NSV**: Repository README and installation guide
- **Barretenberg**: Included with Noir installation

## Deployed Verifier Addresses

### **Arbitrum Sepolia Testnet**

**UltraKeccakFlavor (Normal Proofs)**
- Address: `<CONTRACT_ADDRESS>`
- Usage: Standard verification workflow

**UltraHonkFlavor (ZK Proofs)**
- Address: `<CONTRACT_ADDRESS>`
- Usage: Zero-knowledge verification workflow

## Directory Structure for Testing

Recommended testing environment structure:
```
testing-environment/
├── circuits/                 # Test circuits
│   ├── hello_world/         # Simple test circuit (use the one from pocs/)
│   └── custom/              # Your test circuits
├── proofs/                  # Generated proofs
└── validation/              # Validation scripts
```

## Usage Patterns

### **Proof Generation**
- Generate normal proofs: `nsv prove --circuit-path <circuit>`
- Generate ZK proofs: `nsv prove --circuit-path <circuit> --zk`

### **Verification Testing**
- Off-chain verification: `nsv verify --proof <proof> --vk <vk>`
- On-chain verification: Include `--rpc-url` and `--verifier-address`


## Security Testing Environment


### **Testing Best Practices**
1. **Use Sepolia only** for on-chain testing
2. **Use mainnet for production** after validation
3. **Isolate environments** for testing
4. **Backup important data** before running validation tools

### **Tool Versions**
- **Noir**: 1.0.0-beta.6+
- **Barretenberg**: 0.86.0+
- **NSV**: Latest version from repository

## Support

### **Documentation**
- **Main Docs**: https://nsv.wakeuplabs.link
- **Repository**: https://github.com/wakeuplabs-io/noir-stylus-verifier

### **For Issues**
- Reference individual findings in `findings/` directory


---

**Important**: This environment configuration was originally used for security testing.