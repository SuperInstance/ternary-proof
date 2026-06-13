# ternary-proof

Ternary proof verification system. Assertions return {-1=invalid, 0=inconclusive, +1=valid} with confidence-weighted evidence chains, challenge-response verification, and majority-vote aggregation across proof strategies.

## Why It Matters

Binary logic (true/false) is the standard for formal verification. But real-world proof systems — legal arguments, scientific claims, multi-agent consensus — often live in a richer space:

- **+1 (Valid)**: the claim is confirmed with high confidence
- **0 (Inconclusive)**: evidence is insufficient — we neither accept nor reject
- **-1 (Invalid)**: the claim is contradicted by evidence

This ternary verdict system enables:
- **Graceful uncertainty**: "I don't know" is a first-class answer, not a failure
- **Composable verification**: chain assertions with AND/OR/min/max semantics
- **Confidence thresholds**: claims below 50% confidence are inconclusive, not false
- **Majority voting**: aggregate multiple independent verifications
- **Challenge-response**: interactive proof with edit-distance tolerance for fuzzy matching

## How It Works

### Assertion Model

Each assertion carries a confidence score $c \in [0, 1]$:

$$\text{verify}(a) = \begin{cases} \text{Valid (+1)} & \text{if } c \geq 0.9 \\ \text{Inconclusive (0)} & \text{if } 0.5 \leq c < 0.9 \\ \text{Invalid (-1)} & \text{if } c < 0.5 \end{cases}$$

### Compositional Logic

Two ternary results combine via lattice operations:

**AND (min):** The weakest link determines validity.

$$a \wedge b = \min(a, b)$$

| ∧ | -1 | 0 | +1 |
|---|----|---|-----|
| **-1** | -1 | -1 | -1 |
| **0** | -1 | 0 | 0 |
| **+1** | -1 | 0 | +1 |

**OR (max):** The strongest supporter carries the verdict.

$$a \vee b = \max(a, b)$$

| ∨ | -1 | 0 | +1 |
|---|----|---|-----|
| **-1** | -1 | 0 | +1 |
| **0** | 0 | 0 | +1 |
| **+1** | +1 | +1 | +1 |

**Complexity:** O(1) per logical operation — simple integer comparison.

### Proof Chains

A sequence of assertions $\{a_1, a_2, \ldots, a_n\}$ with three verification strategies:

- **All-valid (AND chain):** $\bigwedge_i \text{verify}(a_i)$ — every link must hold
- **Any-valid (OR chain):** $\bigvee_i \text{verify}(a_i)$ — at least one must hold
- **Majority vote:** Valid if $|\{i : v_i = +1\}| > n/2$ and more than invalid count

### Challenge-Response

Verify a response against an expected answer using Levenshtein edit distance:

$$\text{similarity}(a, b) = 1 - \frac{d_{\text{edit}}(a, b)}{\max(|a|, |b|)}$$

- Exact match: Valid (+1)
- Similarity ≥ tolerance: Inconclusive (0)
- Below tolerance: Invalid (-1)

**Levenshtein DP:** Standard $O(m \times n)$ dynamic programming where $m, n$ are string lengths.

## Quick Start

```rust
use ternary_proof::*;

// Individual assertions
let strong = Assertion::new("1", "the sky is blue", 0.95);
assert_eq!(strong.verify(), VerifyResult::Valid);

let weak = Assertion::new("2", "maybe it'll rain", 0.6);
assert_eq!(weak.verify(), VerifyResult::Inconclusive);

let contradicted = Assertion::new("3", "2+2=5", 0.2);
assert_eq!(contradicted.verify(), VerifyResult::Invalid);

// Proof chains
let chain = ProofChain::new(vec![
    Assertion::new("1", "premise A", 0.95),
    Assertion::new("2", "premise B", 0.95),
]);
assert_eq!(chain.verify_all(), VerifyResult::Valid);

// One weak link breaks the chain
let chain2 = ProofChain::new(vec![
    Assertion::new("1", "strong", 0.95),
    Assertion::new("2", "weak", 0.3),
]);
assert_eq!(chain2.verify_all(), VerifyResult::Invalid);
assert_eq!(chain2.verify_any(), VerifyResult::Valid);

// Majority vote
let chain3 = ProofChain::new(vec![
    Assertion::new("1", "yes", 0.95),
    Assertion::new("2", "yes", 0.95),
    Assertion::new("3", "no", 0.3),
]);
assert_eq!(chain3.verify_majority(), VerifyResult::Valid);

// Challenge-response
let cr = ChallengeResponse::new("2+2", "4");
assert_eq!(cr.verify("4"), VerifyResult::Valid);       // exact
assert_eq!(cr.verify("5"), VerifyResult::Invalid);     // wrong

// Lattice operations
use VerifyResult::*;
assert_eq!(Valid.and(Invalid), Invalid);  // weakest link
assert_eq!(Valid.or(Invalid), Valid);     // strongest supporter
```

## API

| Type / Method | Description |
|---|---|
| `VerifyResult::Invalid / Inconclusive / Valid` | Ternary verdict enum |
| `.to_i8() / from_i8()` | Convert to/from {-1, 0, +1} |
| `.and(other) / .or(other)` | Lattice meet/join |
| `Assertion::new(id, claim, confidence)` | Create an assertion |
| `.with_evidence(&[evidence])` | Attach supporting evidence |
| `.verify() → VerifyResult` | Evaluate assertion |
| `ProofChain::new(assertions)` | Chain of dependent assertions |
| `.verify_all() / verify_any() / verify_majority()` | Aggregation strategies |
| `ChallengeResponse::new(challenge, expected)` | Interactive proof |
| `.verify(response) → VerifyResult` | Check with edit-distance tolerance |

## Architecture Notes

The ternary proof system is a direct instance of the **γ + η = C** conservation identity in the epistemic domain. A Valid verdict (+1) represents constructive evidence mass γ — proof that builds confidence. An Invalid verdict (-1) represents inhibitory evidence mass η — proof that undermines confidence. An Inconclusive verdict (0) is the neutral epistemic state — insufficient evidence to commit either way.

The conserved total $C = n$ (number of assertions) bounds the maximum evidence mass. The AND-combinator implements strict conservation: any η mass (Invalid) anywhere in the chain zeroes the entire result. The OR-combinator is lenient: a single γ mass (Valid) can override any amount of η. The majority vote requires $\gamma > \eta$ AND $\gamma > C/2$ — both the constructive majority and the absolute majority conditions.

The confidence thresholds (0.9 for Valid, 0.5 for Inconclusive) partition the [0,1] interval into three regions, creating a ternary signal from a continuous input — the same quantization operation that BitNet applies to neural network weights.

## References

- Kleene, S. C. (1952). *Introduction to Metamathematics.* North-Holland. (Kleene 3-valued logic)
- Goldwasser, S. et al. (1989). *The Knowledge Complexity of Interactive Proof Systems.* STOC.
- Cook, S. A. (1971). *The Complexity of Theorem-Proving Procedures.* STOC. (NP-completeness)
- Levenshtein, V. I. (1966). *Binary Codes Capable of Correcting Deletions, Insertions and Reversals.* Soviet Physics Doklady.

## License

MIT
