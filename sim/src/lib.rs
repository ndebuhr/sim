//! # Overview
//! "Sim" or "Sim-RS" provides a discrete event simulation engine, to
//! facilitate Rust- and npm-based simulation products and projects.
//!
//! This repository contains:
//!
//! * Random variable framework, for easy specification of stochastic model
//! behaviors.
//! * Pre-built atomic models, for quickly building out simulations of
//! dynamic systems with common modular components.
//! * Output analysis framework, for analyzing simulation outputs
//! statistically.
//! * Simulator engine, for managing and executing discrete event
//! simulations.
//!
//! Sim is compatible with a wide variety of compilation targets, including
//! WASM. Sim does not require nightly Rust.
pub mod input_modeling;
pub mod models;
pub mod output_analysis;
pub mod simulator;
pub mod utils;
