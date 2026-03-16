use crate::syntax::Path;

#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
}
