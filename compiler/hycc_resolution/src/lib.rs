use crate::diag::ResolverDiag;

pub mod diag;

pub mod resolver_traits;

pub mod ident;
pub mod ty;

pub type ResolveResult<T = (), E = Option<ResolverDiag>> = Result<T, E>;
