#![no_std]
#![no_main]
#![feature(exhaustive_patterns)]
#![feature(stmt_expr_attributes)]

use panic_halt as _;

use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f3xx_hal::{pac, prelude::*};

mod led_wheel;
use led_wheel::LEDWheel;

mod buttons;
use buttons::Buttons;

fn sleep(seconds: f32) {
    asm::delay((seconds * 8_000_000f32) as u32)
}

#[entry]
fn main() -> ! {
    let mut peripherals = pac::Peripherals::take().unwrap();

    let mut reset_and_clock_control = peripherals.RCC.constrain();
    let gpioe = peripherals.GPIOE.split(&mut reset_and_clock_control.ahb);
    let mut led_wheel = LEDWheel::new(gpioe);

    let gpioa = peripherals.GPIOA.split(&mut reset_and_clock_control.ahb);
    let mut buttons = Buttons::new(gpioa, &mut peripherals.EXTI);

    // Start here so that we loop round to 0 on the first iteration.
    let mut index: usize = LEDWheel::COUNT - 1;
    let mut delta: i8 = 1;
    loop {
        if buttons.user.handle_pressed() {
            delta *= -1;
        }
        let next_index = ((index as i8 + delta) % LEDWheel::COUNT as i8) as usize;

        let Ok(_) = led_wheel.by_index(next_index).set_high();
        let Ok(_) = led_wheel.by_index(index).set_low();
        index = next_index;
        sleep(0.25);
    }
}
