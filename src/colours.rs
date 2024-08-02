macro_rules! owo_colors_ext {
    ($trait_name:ident $stream_type:ident
---
    $(#[$fg_meta:meta] #[$bg_meta:meta] $color:ident $fg_method:ident $bg_method:ident),*
) => {
    impl<T: owo_colors::OwoColorize> $trait_name for T {}

    #[allow(dead_code)]
    pub trait $trait_name: owo_colors::OwoColorize {
    $(
        #[$fg_meta]
        #[must_use]
        #[inline(always)]
        fn $fg_method(&self) -> owo_colors::SupportsColorsDisplay<'_, Self, owo_colors::FgColorDisplay<'_, owo_colors::colors::$color, Self>, fn(&Self) -> owo_colors::FgColorDisplay<'_, owo_colors::colors::$color, Self>>{
            self.if_supports_color(owo_colors::Stream::$stream_type, <Self as owo_colors::OwoColorize>::$fg_method)
        }

        #[$bg_meta]
        #[must_use]
        #[inline(always)]
        fn $bg_method(&self) -> owo_colors::SupportsColorsDisplay<'_, Self, owo_colors::BgColorDisplay<'_, owo_colors::colors::$color, Self>, fn(&Self) -> owo_colors::BgColorDisplay<'_, owo_colors::colors::$color, Self>>{
            self.if_supports_color(owo_colors::Stream::$stream_type, <Self as owo_colors::OwoColorize>::$bg_method)
        }

    )*
    }

};
}

macro_rules! gen_ext {
    ($($trait_name:ident $stream_type:ident)*) => {
        $(owo_colors_ext! {
            $trait_name $stream_type
            ---
        /// Change the foreground color to black
        /// Change the background color to black
        Black    black    on_black,
        /// Change the foreground color to red
        /// Change the background color to red
        Red      red      on_red,
        /// Change the foreground color to green
        /// Change the background color to green
        Green    green    on_green,
        /// Change the foreground color to yellow
        /// Change the background color to yellow
        Yellow   yellow   on_yellow,
        /// Change the foreground color to blue
        /// Change the background color to blue
        Blue     blue     on_blue,
        /// Change the foreground color to magenta
        /// Change the background color to magenta
        Magenta  magenta  on_magenta,
        /// Change the foreground color to purple
        /// Change the background color to purple
        Magenta  purple   on_purple,
        /// Change the foreground color to cyan
        /// Change the background color to cyan
        Cyan     cyan     on_cyan,
        /// Change the foreground color to white
        /// Change the background color to white
        White    white    on_white,

        /// Change the foreground color to the terminal default
        /// Change the background color to the terminal default
        Default default_color on_default_color,

        /// Change the foreground color to bright black
        /// Change the background color to bright black
        BrightBlack    bright_black    on_bright_black,
        /// Change the foreground color to bright red
        /// Change the background color to bright red
        BrightRed      bright_red      on_bright_red,
        /// Change the foreground color to bright green
        /// Change the background color to bright green
        BrightGreen    bright_green    on_bright_green,
        /// Change the foreground color to bright yellow
        /// Change the background color to bright yellow
        BrightYellow   bright_yellow   on_bright_yellow,
        /// Change the foreground color to bright blue
        /// Change the background color to bright blue
        BrightBlue     bright_blue     on_bright_blue,
        /// Change the foreground color to bright magenta
        /// Change the background color to bright magenta
        BrightMagenta  bright_magenta  on_bright_magenta,
        /// Change the foreground color to bright purple
        /// Change the background color to bright purple
        BrightMagenta  bright_purple   on_bright_purple,
        /// Change the foreground color to bright cyan
        /// Change the background color to bright cyan
        BrightCyan     bright_cyan     on_bright_cyan,
        /// Change the foreground color to bright white
        /// Change the background color to bright white
        BrightWhite    bright_white    on_bright_white
    })*
    };
}

gen_ext! {
    OwoColorizeStdoutSupported Stdout
    OwoColorizeStderrSupported Stderr
}
