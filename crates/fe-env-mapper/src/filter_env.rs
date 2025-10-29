use anyhow::Context;

#[derive(Debug, Clone)]
pub enum FilterRule {
    Exact(String),
    Regex(regex::Regex),
}

impl FilterRule {
    pub fn matches(&self, key: &str) -> bool {
        match self {
            FilterRule::Exact(exact) => key.eq(exact),
            FilterRule::Regex(re) => re.is_match(key),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    pub include_rules: Vec<FilterRule>,
    pub exclude_rules: Vec<FilterRule>,
}

impl FilterConfig {
    pub fn builder() -> FilterConfigBuilder {
        FilterConfigBuilder::new()
    }
    pub fn should_process(&self, key: &str) -> bool {
        if self.exclude_rules.iter().any(|rule| rule.matches(key)) {
            return false;
        }

        if self.include_rules.is_empty() {
            return true;
        }

        self.include_rules.iter().any(|rule| rule.matches(key))
    }

    pub fn is_empty(&self) -> bool {
        self.include_rules.is_empty() && self.exclude_rules.is_empty()
    }
}

pub struct FilterConfigBuilder {
    include_rules: Vec<FilterRule>,
    exclude_rules: Vec<FilterRule>,
}

impl FilterConfigBuilder {
    pub fn new() -> Self {
        Self {
            include_rules: Vec::new(),
            exclude_rules: Vec::new(),
        }
    }

    pub fn include_exact(mut self, keys: Vec<String>) -> Self {
        for key in keys {
            self.include_rules.push(FilterRule::Exact(key));
        }
        self
    }

    pub fn include_regex(mut self, patterns: Vec<String>) -> anyhow::Result<Self> {
        for pattern in patterns {
            let re = regex::Regex::new(&pattern)
                .context(format!("Invalid include regex: {}", pattern))?;
            self.include_rules.push(FilterRule::Regex(re));
        }
        Ok(self)
    }

    pub fn exclude_exact(mut self, keys: Vec<String>) -> Self {
        for key in keys {
            self.exclude_rules.push(FilterRule::Exact(key));
        }
        self
    }

    pub fn exclude_regex(mut self, patterns: Vec<String>) -> anyhow::Result<Self> {
        for pattern in patterns {
            let re = regex::Regex::new(&pattern)
                .context(format!("Invalid exclude regex: {}", pattern))?;
            self.exclude_rules.push(FilterRule::Regex(re));
        }
        Ok(self)
    }

    pub fn build(self) -> FilterConfig {
        FilterConfig {
            include_rules: self.include_rules,
            exclude_rules: self.exclude_rules,
        }
    }
}