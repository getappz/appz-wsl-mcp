use regex::Regex;

#[derive(Debug, Clone)]
pub enum Action {
    Allow,
    Deny,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub pattern: Regex,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    pub rules: Vec<PolicyRule>,
    pub default_action: Action,
}

impl PolicyEngine {
    pub fn new(rules: Vec<(String, Action)>, default_action: Action) -> Self {
        Self {
            rules: rules
                .into_iter()
                .filter_map(|(pattern, action)| {
                    Regex::new(&pattern).ok().map(|re| PolicyRule { pattern: re, action })
                })
                .collect(),
            default_action,
        }
    }

    pub fn evaluate(&self, command: &str) -> Action {
        for rule in &self.rules {
            if rule.pattern.is_match(command) {
                match rule.action {
                    Action::Deny | Action::Confirm => return rule.action.clone(),
                    Action::Allow => return Action::Allow,
                }
            }
        }
        self.default_action.clone()
    }
}
