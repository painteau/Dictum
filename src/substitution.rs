use crate::config::Substitution;

const MAX_RULES: usize = 100;

/// Substitutions françaises prédéfinies courantes (abréviations SMS/oral)
pub fn french_defaults() -> Vec<Substitution> {
    vec![
        sub("dc", "donc"),
        sub("pk", "pourquoi"),
        sub("pcq", "parce que"),
        sub("pr", "pour"),
        sub("tt", "tout"),
        sub("tjs", "toujours"),
        sub("jsp", "je ne sais pas"),
        sub("jsuis", "je suis"),
        sub("jvais", "je vais"),
        sub("cad", "c'est-à-dire"),
        sub("svp", "s'il vous plaît"),
        sub("stp", "s'il te plaît"),
        sub("rdv", "rendez-vous"),
        sub("asap", "dès que possible"),
        sub("ok", "d'accord"),
    ]
}

fn sub(from: &str, to: &str) -> Substitution {
    Substitution { from: from.to_string(), to: to.to_string(), case_insensitive: true }
}

pub fn apply(rules: &[Substitution], text: &str) -> String {
    if rules.is_empty() {
        return text.to_string();
    }
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
