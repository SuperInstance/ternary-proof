//! Ternary proof system: verification returns {-1=invalid, 0=inconclusive, +1=valid}.

use std::collections::HashMap;

/// Verification result
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerifyResult { Invalid, Inconclusive, Valid }

impl VerifyResult {
    pub fn to_i8(self) -> i8 {
        match self { VerifyResult::Invalid => -1, VerifyResult::Inconclusive => 0, VerifyResult::Valid => 1 }
    }
    pub fn from_i8(v: i8) -> Self {
        match v { -1 => VerifyResult::Invalid, 0 => VerifyResult::Inconclusive, 1 => VerifyResult::Valid, _ => VerifyResult::Inconclusive }
    }
    pub fn and(self, other: Self) -> Self {
        Self::from_i8(self.to_i8().min(other.to_i8()))
    }
    pub fn or(self, other: Self) -> Self {
        Self::from_i8(self.to_i8().max(other.to_i8()))
    }
}

/// A proof assertion
#[derive(Clone, Debug)]
pub struct Assertion {
    pub id: String,
    pub claim: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

impl Assertion {
    pub fn new(id: &str, claim: &str, confidence: f64) -> Self {
        Self { id: id.to_string(), claim: claim.to_string(), evidence: Vec::new(), confidence }
    }

    pub fn with_evidence(mut self, evidence: &[&str]) -> Self {
        self.evidence = evidence.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn verify(&self) -> VerifyResult {
        if self.confidence >= 0.9 { VerifyResult::Valid }
        else if self.confidence >= 0.5 { VerifyResult::Inconclusive }
        else { VerifyResult::Invalid }
    }
}

/// A proof chain: sequence of assertions where each depends on the previous
pub struct ProofChain {
    pub assertions: Vec<Assertion>,
}

impl ProofChain {
    pub fn new(assertions: Vec<Assertion>) -> Self {
        Self { assertions }
    }

    /// Verify entire chain: all must be valid
    pub fn verify_all(&self) -> VerifyResult {
        self.assertions.iter()
            .map(|a| a.verify())
            .fold(VerifyResult::Valid, |acc, v| acc.and(v))
    }

    /// Verify: at least one valid
    pub fn verify_any(&self) -> VerifyResult {
        self.assertions.iter()
            .map(|a| a.verify())
            .fold(VerifyResult::Invalid, |acc, v| acc.or(v))
    }

    /// Majority vote
    pub fn verify_majority(&self) -> VerifyResult {
        let valid = self.assertions.iter().filter(|a| a.verify() == VerifyResult::Valid).count();
        let invalid = self.assertions.iter().filter(|a| a.verify() == VerifyResult::Invalid).count();
        if valid > invalid && valid > self.assertions.len() / 2 { VerifyResult::Valid }
        else if invalid > valid && invalid > self.assertions.len() / 2 { VerifyResult::Invalid }
        else { VerifyResult::Inconclusive }
    }

    pub fn len(&self) -> usize { self.assertions.len() }
}

/// Challenge-response verification
pub struct ChallengeResponse {
    pub challenge: String,
    pub expected_response: String,
    pub tolerance: f64,
}

impl ChallengeResponse {
    pub fn new(challenge: &str, expected: &str) -> Self {
        Self { challenge: challenge.to_string(), expected_response: expected.to_string(), tolerance: 1.0 }
    }

    pub fn verify(&self, response: &str) -> VerifyResult {
        if response == self.expected_response { VerifyResult::Valid }
        else if self.levenshtein_similarity(response, &self.expected_response) >= self.tolerance {
            VerifyResult::Inconclusive
        } else {
            VerifyResult::Invalid
        }
    }

    fn levenshtein_similarity(&self, a: &str, b: &str) -> f64 {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let n = a_chars.len().max(b_chars.len());
        if n == 0 { return 1.0; }
        let mut dp = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];
        for i in 0..=a_chars.len() { dp[i][0] = i; }
        for j in 0..=b_chars.len() { dp[0][j] = j; }
        for i in 1..=a_chars.len() {
            for j in 1..=b_chars.len() {
                let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
                dp[i][j] = dp[i-1][j].min(dp[i][j-1]).min(dp[i-1][j-1]) + cost;
            }
        }
        1.0 - dp[a_chars.len()][b_chars.len()] as f64 / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_assertion() {
        let a = Assertion::new("1", "claim", 0.95);
        assert_eq!(a.verify(), VerifyResult::Valid);
    }

    #[test]
    fn test_inconclusive_assertion() {
        let a = Assertion::new("1", "claim", 0.7);
        assert_eq!(a.verify(), VerifyResult::Inconclusive);
    }

    #[test]
    fn test_invalid_assertion() {
        let a = Assertion::new("1", "claim", 0.3);
        assert_eq!(a.verify(), VerifyResult::Invalid);
    }

    #[test]
    fn test_chain_all_valid() {
        let chain = ProofChain::new(vec![
            Assertion::new("1", "a", 0.95),
            Assertion::new("2", "b", 0.95),
        ]);
        assert_eq!(chain.verify_all(), VerifyResult::Valid);
    }

    #[test]
    fn test_chain_one_invalid() {
        let chain = ProofChain::new(vec![
            Assertion::new("1", "a", 0.95),
            Assertion::new("2", "b", 0.3),
        ]);
        assert_eq!(chain.verify_all(), VerifyResult::Invalid);
        assert_eq!(chain.verify_any(), VerifyResult::Valid);
    }

    #[test]
    fn test_challenge_exact() {
        let cr = ChallengeResponse::new("2+2", "4");
        assert_eq!(cr.verify("4"), VerifyResult::Valid);
    }

    #[test]
    fn test_challenge_wrong() {
        let cr = ChallengeResponse::new("2+2", "4");
        assert_eq!(cr.verify("5"), VerifyResult::Invalid);
    }

    #[test]
    fn test_verify_and_or() {
        assert_eq!(VerifyResult::Valid.and(VerifyResult::Invalid), VerifyResult::Invalid);
        assert_eq!(VerifyResult::Valid.or(VerifyResult::Invalid), VerifyResult::Valid);
    }
}
