# Ricardian Registry — Soroban Smart Contract

A Soroban smart contract for anchoring legal documents on-chain with contextual metadata and version chains. Part of the [Blubird](https://blubird.io) RWA tokenization platform.

## What It Does

The Ricardian Registry stores document attestations linked to specific entities (registries, opportunities, or instruments) with document type classification and automatic version chaining. Each document's SHA-256 content hash is anchored on-chain, creating an immutable, verifiable record that a specific legal document existed at a specific point in time.

## Contract Interface

| Function | Description |
|----------|-------------|
| `initialize(admin)` | Set the contract administrator |
| `anchor_document(entity_id, entity_type, doc_type, doc_hash)` | Anchor a document hash with context. Returns version number. |
| `get_document(entity_id, doc_type, version)` | Retrieve a document record (pass 0 for latest) |
| `get_history(entity_id, doc_type)` | Get full version chain for an entity + document type |
| `get_all_documents(entity_id)` | Get all latest documents for an entity |
| `verify_document(doc_hash)` | Look up a hash — returns entity context if it exists on-chain |

## Entity Types

| Value | Type |
|-------|------|
| 0 | Registry |
| 1 | Opportunity |
| 2 | Instrument |

## Deployed Contract

| Network | Contract ID |
|---------|-------------|
| Stellar Testnet | `CCJFVK7ICKQEJRVK7ROIULVA7ORBR3WSSLFGDTAIZ76QA6SENK2XRKBG` |

[View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CCJFVK7ICKQEJRVK7ROIULVA7ORBR3WSSLFGDTAIZ76QA6SENK2XRKBG)

## Build
```bash
rustup target add wasm32v1-none
stellar contract build
```

## Verify
```bash
stellar contract invoke \
  --id CCJFVK7ICKQEJRVK7ROIULVA7ORBR3WSSLFGDTAIZ76QA6SENK2XRKBG \
  --network testnet \
  --source operator \
  -- \
  verify_document \
  --doc_hash <SHA256_HEX>
```

## License

See LICENSE
