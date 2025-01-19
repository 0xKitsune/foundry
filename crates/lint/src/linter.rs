use clap::ValueEnum;
use core::fmt;
use foundry_compilers::Language;
use solar_ast::Span;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    hash::Hash,
    ops::{Deref, DerefMut},
    path::PathBuf,
};
use yansi::Paint;

/// Trait representing a generic linter for analyzing and reporting issues in smart contract source
/// code files. A linter can be implemented for any smart contract language supported by Foundry.
///
/// # Type Parameters
///
/// - `Language`: Represents the target programming language. Must implement the [`Language`] trait.
/// - `Lint`: Represents the types of lints performed by the linter. Must implement the [`Lint`]
///   trait.
/// - `LinterError`: Represents errors that can occur during the linting process.
///
/// # Required Methods
///
/// - `lint`: Scans the provided source files and returns a [`LinterOutput`] containing categorized
///   findings or an error if linting fails.
pub trait Linter: Send + Sync + Clone {
    type Language: Language;
    type Lint: Lint + Ord;
    type LinterError: Error + Send + Sync + 'static;

    fn lint(&self, input: &[PathBuf]) -> Result<(), Self::LinterError>;
}

pub struct ProjectLinter<L>
where
    L: Linter,
{
    pub linter: L,
}

impl<L> ProjectLinter<L>
where
    L: Linter,
{
    pub fn new(linter: L) -> Self {
        Self { linter }
    }

    pub fn lint(self, input: &[PathBuf]) -> eyre::Result<()> {
        Ok(self.linter.lint(input)?)
    }
}

pub trait Lint: Hash {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn url(&self) -> Option<&'static str>;
    fn severity(&self) -> Severity;
}

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Severity {
    High,
    Med,
    Low,
    Info,
    Gas,
}

impl Severity {
    pub fn color(&self, message: &str) -> String {
        match self {
            Self::High => Paint::red(message).bold().to_string(),
            Self::Med => Paint::rgb(message, 255, 135, 61).bold().to_string(),
            Self::Low => Paint::yellow(message).bold().to_string(),
            Self::Info => Paint::cyan(message).bold().to_string(),
            Self::Gas => Paint::green(message).bold().to_string(),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colored = match self {
            Self::High => self.color("High"),
            Self::Med => self.color("Med"),
            Self::Low => self.color("Low"),
            Self::Info => self.color("Info"),
            Self::Gas => self.color("Gas"),
        };
        write!(f, "{colored}")
    }
}
