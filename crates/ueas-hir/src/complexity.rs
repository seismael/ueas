//! Complexity formula parser and asymptotic ordering for UEAS v2.0.

use crate::hir::Expr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Asymptotic complexity specification on an Algorithm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexitySpec {
    pub time: Option<ComplexityBound>,
    pub space: Option<ComplexityBound>,
    pub work: Option<ComplexityBound>,
    pub span_bound: Option<ComplexityBound>,
    pub expected: Option<ComplexityBound>,
    pub bindings: Vec<(String, Expr)>,
}

impl Default for ComplexitySpec {
    fn default() -> Self {
        Self {
            time: Some(ComplexityBound {
                raw: "O(1)".to_string(),
                class: AsymptoticClass::Constant,
            }),
            space: None,
            work: None,
            span_bound: None,
            expected: None,
            bindings: Vec::new(),
        }
    }
}

/// Parsed complexity bound string + normalized asymptotic class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplexityBound {
    pub raw: String,
    pub class: AsymptoticClass,
}

/// Standard asymptotic complexity classes ordered from lowest to highest growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AsymptoticClass {
    Constant,     // O(1)
    Logarithmic,  // O(log N)
    Sqrt,         // O(sqrt N)
    Linear,       // O(N) or O(V + E)
    Linearithmic, // O(N log N) or O((V + E) log V)
    NLog2N,       // O(N log^2 N)
    Quadratic,    // O(N^2) or O(N * M)
    Cubic,        // O(N^3)
    Exponential,  // O(2^N)
    Factorial,    // O(N!)
}

impl AsymptoticClass {
    /// Human-readable canonical Big-O notation.
    pub fn label(self) -> &'static str {
        match self {
            Self::Constant => "O(1)",
            Self::Logarithmic => "O(log N)",
            Self::Sqrt => "O(sqrt N)",
            Self::Linear => "O(N)",
            Self::Linearithmic => "O(N log N)",
            Self::NLog2N => "O(N log^2 N)",
            Self::Quadratic => "O(N^2)",
            Self::Cubic => "O(N^3)",
            Self::Exponential => "O(2^N)",
            Self::Factorial => "O(N!)",
        }
    }

    /// Evaluate the theoretical basis function f(n) for curve fitting.
    pub fn eval_basis(self, n: f64) -> f64 {
        let n = n.max(2.0);
        let log_n = n.log2().max(1.0);
        match self {
            Self::Constant => 1.0,
            Self::Logarithmic => log_n,
            Self::Sqrt => n.sqrt(),
            Self::Linear => n,
            Self::Linearithmic => n * log_n,
            Self::NLog2N => n * log_n * log_n,
            Self::Quadratic => n * n,
            Self::Cubic => n * n * n,
            Self::Exponential => 2.0_f64.powf(n.min(60.0)),
            Self::Factorial => {
                let cap = n.min(20.0) as u64;
                (1..=cap).fold(1.0_f64, |acc, x| acc * (x as f64))
            }
        }
    }
}

/// Parse a complexity string such as `"O(N log N)"` or `O((V+E) log V)`.
pub fn parse_complexity_bound(raw: &str) -> Result<ComplexityBound, String> {
    let trimmed = raw.trim().trim_matches('"').trim();
    let after_o = trimmed
        .strip_prefix('O')
        .map(str::trim_start)
        .filter(|s| s.starts_with('(') && s.ends_with(')'));
    let Some(paren_part) = after_o else {
        return Err(format!(
            "invalid complexity notation '{trimmed}': expected O(...)"
        ));
    };
    let inner = paren_part[1..paren_part.len() - 1].trim();
    if inner.is_empty() {
        return Err("empty complexity expression O()".to_string());
    }
    let canonical_raw = format!("O({inner})");
    let norm = inner
        .replace(' ', "")
        .replace("·", "*")
        .to_ascii_lowercase();

    let class = if norm == "1" {
        AsymptoticClass::Constant
    } else if norm.contains('!') {
        AsymptoticClass::Factorial
    } else if norm.contains("2^") {
        AsymptoticClass::Exponential
    } else if norm.contains("^3") {
        AsymptoticClass::Cubic
    } else if norm.contains("^2") && norm.contains("log") {
        AsymptoticClass::NLog2N
    } else if norm.contains("^2") || (norm.contains('*') && !norm.contains("log")) {
        AsymptoticClass::Quadratic
    } else if (norm.contains("log")
        && (norm.contains("nlog") || norm.contains(")log") || norm.contains("*log")))
        || norm.contains("logn*n")
    {
        AsymptoticClass::Linearithmic
    } else if norm.contains("sqrt") {
        AsymptoticClass::Sqrt
    } else if norm.starts_with("log") {
        AsymptoticClass::Logarithmic
    } else if norm
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '*' | '/' | '-' | '(' | ')' | '_'))
    {
        AsymptoticClass::Linear
    } else {
        return Err(format!("unsupported complexity form '{trimmed}'"));
    };

    Ok(ComplexityBound {
        raw: canonical_raw,
        class,
    })
}

/// Extract uppercase symbolic variable names (e.g. `N`, `V`, `E`, `M`) from a complexity string.
pub fn extract_complexity_vars(raw: &str) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    let trimmed = raw.trim().trim_matches('"');
    let inner = trimmed
        .strip_prefix("O(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(trimmed);
    let mut cur = String::new();
    for ch in inner.chars() {
        if ch.is_ascii_alphabetic() {
            cur.push(ch);
        } else {
            if !cur.is_empty() && cur != "log" && cur != "sqrt" && cur != "min" && cur != "max" {
                vars.insert(cur.clone());
            }
            cur.clear();
        }
    }
    if !cur.is_empty() && cur != "log" && cur != "sqrt" && cur != "min" && cur != "max" {
        vars.insert(cur);
    }
    vars
}
