//! AI Integration
//!
//! Provides Gemini API client for spec analysis.

mod gemini;

pub use gemini::{GeminiClient, GeminiConfig, SpecAnalysis, AmbiguousPhrase, MissingScenario, Risk};
