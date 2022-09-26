#![no_std]
#![no_main]

use core::convert::Infallible;

use panic_halt as _;

use cortex_m::asm;
use embedded_hal::digital::v2::{ToggleableOutputPin, StatefulOutputPin};
use cortex_m_rt::entry;
use stm32f3xx_hal::{gpio, pac, prelude::*};

// Type alias to reduce boilerplate.
type LED<const PIN: u8> = gpio::Pin<gpio::Gpioe, gpio::U<PIN>, gpio::Output<gpio::PushPull>>;
// Combination of useful traits to allow returning an LED with an arbitrary type-level index.
trait LEDTrait: ToggleableOutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible> {}
impl<T: ToggleableOutputPin<Error = Infallible> + StatefulOutputPin<Error = Infallible>> LEDTrait for T {}

struct LEDWheel {
    nw: LED<8>,
    n: LED<9>,
    ne: LED<10>,
    e: LED<11>,
    se: LED<12>,
    s: LED<13>,
    sw: LED<14>,
    w: LED<15>,
}
impl LEDWheel {
    fn new(mut gpioe: gpio::gpioe::Parts) -> LEDWheel {
        LEDWheel {
            nw: gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            n: gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ne: gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            e: gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            se: gpioe.pe12.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            s: gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            sw: gpioe.pe14.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            w: gpioe.pe15.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        }
    }

    fn by_index(&mut self, index: usize) -> &mut dyn LEDTrait {
        let leds: [&mut dyn LEDTrait; 8] = [
            &mut self.n,
            &mut self.ne,
            &mut self.e,
            &mut self.se,
            &mut self.s,
            &mut self.sw,
            &mut self.w,
            &mut self.nw,
        ];
        leds[index % 8]
    }
}

fn sleep(seconds: f32) {
    asm::delay((seconds * 8_000_000f32) as u32)
}

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let mut reset_and_clock_control = peripherals.RCC.constrain();
    let gpioe = peripherals.GPIOE.split(&mut reset_and_clock_control.ahb);
    let mut led_wheel = LEDWheel::new(gpioe);

    let mut index = 0;
    loop {
        led_wheel.by_index(index + 1).set_high().unwrap();
        led_wheel.by_index(index).set_low().unwrap();
        index += 1;
        sleep(0.25);
    }
}
