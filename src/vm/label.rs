use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LabelTable {
    pub ids: Vec<String>,               // id -> label
    pub labels: HashMap<String, usize>, // label -> addr
}

impl LabelTable {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            labels: HashMap::new(),
        }
    }

    pub fn to_addr(&self, id: usize) -> Option<usize> {
        let label = self.ids.get(id)?;
        self.addr_of(label)
    }

    pub fn addr_of(&self, label: &String) -> Option<usize> {
        Some(*self.labels.get(label)?)
    }

    pub fn defer(&mut self, label: String) -> usize {
        (self.ids.len(), self.ids.push(label)).0
    }

    pub fn add(&mut self, label: String, addr: usize) {
        self.labels.insert(label, addr);
    }
}
