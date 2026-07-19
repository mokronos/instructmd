mod model;
mod resolvers;

pub use model::{
    Agent, Candidate, CodexConfig, OpenCodeConfig, PiConfig, Resolution, ResolverConfig, Scope,
    State,
};

use std::path::PathBuf;

pub fn resolve(agent: Agent, dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    resolvers::resolve(agent, dir, cfg)
}

#[cfg(test)]
mod tests;
