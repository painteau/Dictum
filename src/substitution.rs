use crate::config::Substitution;

const MAX_RULES: usize = 100;

pub fn apply(rules: &[Substitution], text: &str) -> String {
    // Appliquer les règles longues en premier (évite les substitutions partielles)
    let mut sorted_rules: Vec<&Substitution> = rules.iter()
        .filter(|r| !r.from.is_empty())
        .take(MAX_RULES)
        .collect();
    sorted_rules.sort_by(|a, b| b.from.len().cmp(&a.from.len()));

    let mut result = text.to_string();
    for rule in &sorted_rules {
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
            log::debug!("Substitution appliquée : {:?} → {:?} (case_insensitive: {})",
                rule.from, rule.to, rule.case_insensitive);
        }
    }
    result
}
