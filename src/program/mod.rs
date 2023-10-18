mod hue_cycle;
mod rain;

use alloc::boxed::Box;

pub use hue_cycle::HueCycle;
pub use rain::Rain;

use crate::driver::ScreenDriver;

pub const PROGRAM_CONSTRUCTORS: [fn() -> Box<dyn Program>; 2] = [HueCycle::new, Rain::new];

pub trait Program {
    fn init(&mut self, driver: &mut ScreenDriver);
    fn render(&mut self, driver: &mut ScreenDriver);
}
