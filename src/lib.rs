//! `axum-guard-router` is a `axum` middleware to create a guard router.

#![doc = include_str!("../README.md")]

mod guard;
mod layer;
mod router;
mod service;

#[cfg(test)]
mod test_helper;

pub mod action;
pub use guard::OnGuard;
pub use router::GuardRouter;
