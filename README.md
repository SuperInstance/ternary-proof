# ternary-proof

Verification that returns a trivalent verdict: invalid, inconclusive, or valid.

## Why This Exists

Binary verification (pass/fail) is a lie. It forces you to pretend that "I couldn't prove it" is the same as "it's wrong." In real systems — access control, evidence chains, zero-knowledge protocols — you need three outcomes: **rejected** (definitely bad), **inconclusive** (not enough evidence either way), and **verified** (definitely good).

This ternary verdict maps to `{−1, 0, +1}` and composes with lattice semantics: `AND` takes the minimum (a chain is only as strong as its weakest link), `OR` takes the maximum (any valid proof suffices). This isn't arbitrary — it's the three-valued logic of Kleene, and it's the right algebra for verification where absence of evidence is not evidence of absence.

The crate implements assertion verification, proof chains with all/any/majority semantics, and challenge-response protocols with fuzzy matching via Levenshtein similarity.

## Architecture

```
Assertion (claim + evidence + confidence)
    │
    └── verify() ──► VerifyResult {Invalid, Inconclusive, Valid}
            │
    ProofChain (Vec<Assertion>)
    ├── verify_all()  ──► fold AND (weakest link)
    ├── verify_any()  ──► fold OR (strongest link)
    └── verify_majority() ──► majority vote

ChallengeResponse (challenge + expected + tolerance)
    └── verify(response) ──► exact match → Valid
                             fuzzy match → Inconclusive
                             no match → Invalid
```

**Key types:**

- **`VerifyResult`** — `Invalid` (`−1`), `Inconclusive` (`0`), `Valid` (`+1`). Supports `.and()` (min) and `.or()` (max) for composition.
- **`Assertion`** — a claim with evidence and a confidence score `[0, 1]`. `verify()` maps confidence to verdict: ≥0.9 → Valid, ≥0.5 → Inconclusive, <0.5 → Invalid.
- **`ProofChain`** — ordered sequence of assertions. Supports three verification strategies: all-must-pass, any-must-pass, and majority vote.
- **`ChallengeResponse`** — challenge-response verification with fuzzy matching. Exact match → Valid. Levenshtein similarity above tolerance → Inconclusive. Otherwise → Invalid.

## Usage

```rust
use ternary_proof::{Assertion, ProofChain, VerifyResult, ChallengeResponse};

// Single assertions
let strong = Assertion::new("1", "user is authenticated", 0.95)
    .with_evidence(&["token valid", "session active"]);
let weak = Assertion::new("2", "request is authorized", 0.6);
let bad = Assertion::new("3", "suspicious origin", 0.2);

assert_eq!(strong.verify(), VerifyResult::Valid);
assert_eq!(weak.verify(), VerifyResult::Inconclusive);
assert_eq!(bad.verify(), VerifyResult::Invalid);

// VerifyResult composition (Kleene three-valued logic)
assert_eq!(VerifyResult::Valid.and(VerifyResult::Invalid), VerifyResult::Invalid); // min
assert_eq!(VerifyResult::Valid.or(VerifyResult::Invalid), VerifyResult::Valid);    // max

// Proof chains: all must be valid
let chain = ProofChain::new(vec![
    Assertion::new("1", "identity proven", 0.95),
    Assertion::new("2", "credential valid", 0.95),
]);
assert_eq!(chain.verify_all(), VerifyResult::Valid);

// One bad assertion poisons the whole chain (AND semantics)
let chain = ProofChain::new(vec![
    Assertion::new("1", "identity proven", 0.95),
    Assertion::new("2", "credential expired", 0.3),
]);
assert_eq!(chain.verify_all(), VerifyResult::Invalid);
assert_eq!(chain.verify_any(), VerifyResult::Valid); // but OR still passes

// Majority vote
let chain = ProofChain::new(vec![
    Assertion::new("1", "a", 0.95),
    Assertion::new("2", "b", 0.95),
    Assertion::new("3", "c", 0.3),
]);
assert_eq!(chain.verify_majority(), VerifyResult::Valid); // 2/3 valid

// Challenge-response with fuzzy matching
let cr = ChallengeResponse::new("2+2", "4");
assert_eq!(cr.verify("4"), VerifyResult::Valid);      // exact
assert_eq!(cr.verify("four"), VerifyResult::Inconclusive); // fuzzy match
assert_eq!(cr.verify("42"), VerifyResult::Invalid);    // wrong

// Custom tolerance for fuzzy matching
let mut cr = ChallengeResponse::new("name", "Alice");
cr.tolerance = 0.5; // require 50% similarity for inconclusive
```

## API Reference

### `VerifyResult`

| Variant | Value | Description |
|---------|-------|-------------|
| `Invalid` | −1 | Definitely not valid |
| `Inconclusive` | 0 | Cannot determine |
| `Valid` | +1 | Definitely valid |

| Method | Description |
|--------|-------------|
| `.to_i8()` | Convert to `{−1, 0, 1}` |
| `.from_i8(v)` | Convert from `i8` (unknown → Inconclusive) |
| `.and(other)` | Min semantics: chain is weakest link |
| `.or(other)` | Max semantics: any valid suffices |

### `Assertion`

| Method | Description |
|--------|-------------|
| `Assertion::new(id, claim, confidence)` | Create assertion with confidence `[0, 1]` |
| `.with_evidence(&[...])` | Attach evidence strings |
| `.verify()` | ≥0.9 → Valid, ≥0.5 → Inconclusive, <0.5 → Invalid |

### `ProofChain`

| Method | Description |
|--------|-------------|
| `ProofChain::new(assertions)` | Create chain from assertions |
| `.verify_all()` | Fold AND: all must be Valid |
| `.verify_any()` | Fold OR: at least one Valid |
| `.verify_majority()` | Majority vote (>50% Valid wins) |
| `.len()` | Number of assertions |

### `ChallengeResponse`

| Method | Description |
|--------|-------------|
| `ChallengeResponse::new(challenge, expected)` | Create with default tolerance 1.0 |
| `.verify(response)` | Exact → Valid, Levenshtein ≥ tolerance → Inconclusive, else Invalid |

Fields: `challenge: String`, `expected_response: String`, `tolerance: f64`

## The Deeper Idea

This crate implements a specific instance of **three-valued logic (Kleene K3)** applied to verification. The key algebraic property is that `AND` and `OR` form a lattice on `{Invalid, Inconclusive, Valid}` with the natural ordering `Invalid < Inconclusive < Valid`.

This matters because verification is fundamentally compositional. You verify parts, then combine results. In classical binary logic, combining "unknown" with "true" gives "true" (OR) or "unknown" (AND depending on interpretation). Kleene logic handles this cleanly: unknown ⊕ true = true (OR), unknown ⊕ false = false (AND). The middle value is **absorptive** — it doesn't override known results.

The confidence-to-verdict mapping (0.9/0.5 thresholds) is a deliberate design choice. The gap between 0.5 and 0.9 is the "inconclusive zone" — wide enough to capture genuine uncertainty, narrow enough to be useful. Adjust these thresholds to match your system's risk tolerance.

Challenge-response with Levenshtein similarity is a practical concession: in real systems, responses aren't always bit-identical. A slightly corrupted token, a differently-encoded string, or a paraphrased answer might still carry evidence of knowledge. The ternary verdict captures exactly this: it's not *valid*, but it's not *wrong* either.

## Related Crates

- **`ternary-negotiate`** — multi-agent negotiation using the same {-1, 0, +1} stance space
- **`ternary-route`** — routing decisions with reject/queue/accept, structurally similar to verification
- **`ternary-scheduler`** — ternary priority scheduling, where verification determines task priority
