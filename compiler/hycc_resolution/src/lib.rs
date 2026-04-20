use crate::diag::ResolverDiag;

pub mod diag;
pub mod ident;

pub type ResolveResult<T = (), E = Option<ResolverDiag>> = Result<T, E>;
