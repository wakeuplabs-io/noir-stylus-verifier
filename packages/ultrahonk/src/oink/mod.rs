//! # Oink Phase  
//!
//! The Oink phase is the first stage of Ultra Honk verification, responsible for
//! setting up the verification environment and handling the initial polynomial
//! commitments. This phase:
//!
//! - Extracts and validates witness commitments from the proof
//! - Generates Fiat-Shamir challenges for the protocol
//! - Computes the public input polynomial and related values
//! - Sets up the verification memory for subsequent phases
//!
//! The name "Oink" comes from the Ultra Honk protocol specification and represents
//! the preparatory phase before the main constraint verification begins.

pub mod types;
pub mod verifier;
