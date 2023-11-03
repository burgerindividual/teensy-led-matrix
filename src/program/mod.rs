mod hue_cycle;
mod rain;

use alloc::boxed::Box;

pub use hue_cycle::HueCycle;
pub use rain::Rain;

use crate::led_driver::ScreenDriver;

pub const PROGRAM_CONSTRUCTORS: [fn(&mut ScreenDriver) -> Box<dyn Program>; 2] =
    [HueCycle::new, Rain::new];

pub trait Program {
    fn render(&mut self, driver: &mut ScreenDriver);
}
