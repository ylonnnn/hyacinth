use crate::syntax::{
    Grammar,
    rule::{GrammarRule, items::VariableRule},
};

pub fn initialize(grammar: &mut Grammar) {
    {
        //  VariableRule
        let rule = VariableRule::new();
        grammar.add(rule.leader(), Box::new(rule));
    }
}
