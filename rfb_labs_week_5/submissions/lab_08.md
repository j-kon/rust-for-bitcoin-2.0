# Lab 08 — BIP32 HD key tree derivation

## Commands used

```bash
cargo test --test lab_08 -- --nocapture
```

## Terminal output

```text
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

BIP32 extended key derivation details verified:
- Regtest Master xpriv prefix: `tprv...`
- Derivation path `m/84'/1'/0'`: `tprv` and `tpub` derived matching pair
- Unhardened child derivation from `tpub` (`index 7`): successfully derived `tpub` child without access to private key material
- Hardened path detection: `m/44'/0'/0'/0/0` identified as hardened, `m/0/1/2` identified as unhardened.

## Evidence references

- Source implementation: [`src/labs/lab08_bip32.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab08_bip32.rs)
- Test suite: [`tests/lab_08.rs`](file:///Users/jaykon/Developer/jaykon/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_08.rs)
- `bitcoin::bip32::Xpriv` and `Xpub` hierarchical deterministic key derivation.

## Explanation

BIP32 Hierarchical Deterministic (HD) key tree derivation provides structured key management through three main components:

1. **Role of Chain Code**:
   - Extended keys (`xprv` and `xpub`) consist of a 32-byte EC key (private or public) combined with a 32-byte **chain code**.
   - The chain code acts as extra entropy (salt) in HMAC-SHA512 derivation step (`HMAC-SHA512(Key = ChainCode, Data = Public/Private Key || ChildIndex)`). Without the chain code, an attacker with a public key could not compute child public keys.

2. **Watch-Only Wallets via Extended Public Keys (`xpub`)**:
   - For **non-hardened derivation** ($i < 2^{31}$), child public keys are derived directly from the parent public key $K_{parent}$ and parent chain code $c_{parent}$ via EC point addition ($K_{child} = K_{parent} + I_L \cdot G$).
   - This allows watch-only wallets, payment servers, or auditors to generate infinitely many receiving public keys/addresses without storing or exposing private keys online.

3. **Hardened Child Derivation ($i \ge 2^{31}$ or `'`)**:
   - Hardened derivation hashes the **parent private key** rather than the parent public key ($HMAC(c_{parent}, 0x00 || k_{parent} || i)$).
   - Because $k_{parent}$ is required, **a parent `xpub` CANNOT derive hardened child keys**.
   - Hardened steps create security firewalls in the HD tree: if a non-hardened child private key $k_{child}$ and parent $Xpub_{parent}$ are compromised, an attacker can mathematically derive $k_{parent}$. Hardened derivation breaks this key leakage chain, isolating accounts and branches from each other.
