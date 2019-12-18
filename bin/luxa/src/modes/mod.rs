use crate::error::*;
use crate::luxa::*;

macro_rules! modes {
    ($(
        $( $kw:ident ),+ = $color:ident
    ),* $(,)?) => {
        #[derive(Debug, Copy, Clone)]
        pub enum Mode {
            $( $color ),*
        }

        pub const ALL_MODES: &[Mode] = &[
            $( Mode::$color ),*
        ];

        impl Mode {
            pub fn names(&self) -> &'static [&'static str] {
                match self {
                    $( Mode::$color => &[ $( stringify!($kw) ),* ] ),*
                }
            }

            pub fn run<L: Luxafor>(&self, device: &L) -> Result<(), LuxaError> {
                match self {
                    $( Mode::$color => device.fade(Color::$color, 50) ),*
                }
            }
        }

    };
}

modes! {
    red, busy = RED,
    blue = BLUE,
    green, available, ok = GREEN,
    cyan = CYAN,
    magenta = MAGENTA,
    yellow = YELLOW,
    white = WHITE,
    black, none, blank = BLACK,
}

pub fn get_all_modes() -> &'static [Mode] {
    return ALL_MODES;
}
