use crate::config::Substitution;

pub fn apply(rules: &[Substitution], text: &str) -> String {
    let mut result = text.to_string();
    for rule in rules {
        if rule.from.is_empty() { continue; }
        let before = result.clone();
        if rule.case_insensitive {
            let lower = result.to_lowercase();
            let from_lower = rule.from.to_lowercase();
            let mut out = String::with_capacity(result.len());
            let mut pos = 0;
            while let Some(idx) = lower[pos..].find(&from_lower) {
                let abs = pos + idx;
                out.push_str(&result[pos..abs]);
                out.push_str(&rule.to);
                pos = abs + rule.from.len();
            }
            out.push_str(&result[pos..]);
            result = out;
        } else {
            result = result.replace(&rule.from, &rule.to);
        }
        if result != before {
            log::debug!("Substitution : {:?} → {:?}", rule.from, rule.to);
        }
    }
    result
}
