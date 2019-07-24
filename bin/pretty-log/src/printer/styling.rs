use crate::message::Severity;

pub struct ColorStyling;
pub struct NoColorStyling;

pub trait Styling: Send + Sync + private::Sealed {
    #[inline]
    fn severity(_severity: Severity) -> &'static str {
        ""
    }
    #[inline]
    fn weak() -> &'static str {
        ""
    }
    #[inline]
    fn strong() -> &'static str {
        ""
    }
    #[inline]
    fn reset() -> &'static str {
        ""
    }
    #[inline]
    fn underline() -> &'static str {
        ""
    }
    #[inline]
    fn no_underline() -> &'static str {
        ""
    }
    #[inline]
    fn emphasize() -> &'static str {
        ""
    }
    #[inline]
    fn no_emphasize() -> &'static str {
        ""
    }
}

impl Styling for ColorStyling {
    fn severity(severity: Severity) -> &'static str {
        if severity >= Severity::Error {
            "\x1b[31m"
        } else if severity >= Severity::Warning {
            "\x1b[33m"
        } else if severity >= Severity::Info {
            "\x1b[34m"
        } else if severity >= Severity::Debug {
            "\x1b[32m"
        } else {
            ""
        }
    }

    fn weak() -> &'static str {
        "\x1b[2m"
    }
    fn strong() -> &'static str {
        "\x1b[1m"
    }
    fn reset() -> &'static str {
        "\x1b[0m"
    }

    fn underline() -> &'static str {
        "\x1b[4m"
    }
    fn no_underline() -> &'static str {
        "\x1b[24m"
    }

    fn emphasize() -> &'static str {
        "\x1b[7m"
    }
    fn no_emphasize() -> &'static str {
        "\x1b[27m"
    }
}

impl Styling for NoColorStyling {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::ColorStyling {}
    impl Sealed for super::NoColorStyling {}
}
