mod clock;
mod hue_cycle;
mod rain;

use alloc::boxed::Box;

pub use clock::Clock;
pub use hue_cycle::HueCycle;
pub use rain::Rain;

use crate::led_driver::ScreenDriver;

pub const PROGRAM_CONSTRUCTORS: [fn(&mut ScreenDriver) -> Box<dyn Program>; 3] =
    [HueCycle::new, Rain::new, Clock::new];

pub trait Program {
    fn render(&mut self, driver: &mut ScreenDriver);
}
