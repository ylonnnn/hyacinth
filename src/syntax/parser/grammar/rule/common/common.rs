use crate::syntax::rule::common::PrattRule;

#[derive(Debug)]
pub struct CommonRules {
    pub pratt: Option<PrattRule>,
}

impl CommonRules {
    pub fn new() -> Self {
        Self {
            pratt: Some(PrattRule::new()),
        }
    }
}
