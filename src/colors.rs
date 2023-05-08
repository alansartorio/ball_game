use bevy::prelude::Color;
use lazy_static::lazy_static;



lazy_static! {
    pub(crate) static ref GLOBAL_BACKGROUND: Color = Color::hex("d8dee9").unwrap();
    pub(crate) static ref BACKGROUND: Color = Color::hex("2e3440").unwrap();
    pub(crate) static ref BLOCKS: Color = Color::hex("8fbcbb").unwrap();
    pub(crate) static ref BALLS: Color = Color::hex("d08770").unwrap();
    pub(crate) static ref DARK_TEXT: Color = Color::hex("2e3440").unwrap();
    pub(crate) static ref LIGHT_TEXT: Color = Color::hex("d8dee9").unwrap();
    pub(crate) static ref BUTTON_BACKGROUND: Color = Color::hex("5e81ac").unwrap();
}
