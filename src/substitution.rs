use crate::config::Substitution;

pub fn apply(rules: &[Substitution], text: &str) -> String {
    let mut result = text.to_string();
    for rule in rules {
        result = result.replace(&rule.from, &rule.to);
    }
    result
}
