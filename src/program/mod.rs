mod hue_cycle;
mod rain;

// pub use rain::Rain;
pub use hue_cycle::HueCycle;

use crate::driver::ScreenDriver;

pub trait Program {
    fn init(&mut self, driver: &mut ScreenDriver);
    fn render(&mut self, driver: &mut ScreenDriver);
}
