//! Defines the various structures that pools map into
//! Typically, one dex will engineer a new variant of
//! a pool representation and other protcols will hit fork
//! and slap a new name on.
//!
//! Maybe theyll even get crafty and change a fee param...
//!
//! These structures define common pool representations from
//! the original protocols which happen to map to various other protocols
//!
//! Adding new protocls that contain a fork of these variants are very easy,
//! otherwise you must implement a new structure alongside the parsing and
//! liquidity populating logic
pub mod balancer_v2_structure;
pub mod maverick_structure;
pub mod tri_crypto_curve_structure;
pub mod two_crypto_curve_structure;
pub mod v2_structure;
pub mod v3_structure;
