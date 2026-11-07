use std::fmt::Display;

use clap::ValueEnum;
use owo_colors::{OwoColorize, Stream};

/// When to color output: `auto` colors a terminal unless `NO_COLOR` is set
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    /// Apply the choice to every color painted afterwards
    pub(crate) fn apply(self) {
        match self {
            Self::Auto => owo_colors::unset_override(),
            Self::Always => owo_colors::set_override(true),
            Self::Never => owo_colors::set_override(false),
        }
    }
}

/// Colors text for standard output, following the chosen [`ColorChoice`]
pub(crate) trait Paint: Display + Sized {
    fn red(&self) -> impl Display + '_ {
        self.if_supports_color(Stream::Stdout, |text| OwoColorize::red(text))
    }

    fn yellow(&self) -> impl Display + '_ {
        self.if_supports_color(Stream::Stdout, |text| OwoColorize::yellow(text))
    }

    fn blue(&self) -> impl Display + '_ {
        self.if_supports_color(Stream::Stdout, |text| OwoColorize::blue(text))
    }

    fn bold(&self) -> impl Display + '_ {
        self.if_supports_color(Stream::Stdout, |text| OwoColorize::bold(text))
    }
}

impl<T: Display> Paint for T {}

/// Colors text for standard error, following the chosen [`ColorChoice`]
pub(crate) trait PaintErr: Display + Sized {
    fn dimmed(&self) -> impl Display + '_ {
        self.if_supports_color(Stream::Stderr, |text| OwoColorize::dimmed(text))
    }
}

impl<T: Display> PaintErr for T {}
