#![no_std]
#![no_main]
#![feature(exhaustive_patterns)]
#![feature(stmt_expr_attributes)]

use panic_halt as _;

use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f3xx_hal::{gpio, pac, prelude::*};

// Type alias to reduce boilerplate.
type LED = gpio::Pin<gpio::Gpioe, gpio::Ux, gpio::Output<gpio::PushPull>>;

struct LEDWheel {
    nw: LED,
    n: LED,
    ne: LED,
    e: LED,
    se: LED,
    s: LED,
    sw: LED,
    w: LED,
}
impl LEDWheel {
    const COUNT: usize = 8;

    fn new(mut gpioe: gpio::gpioe::Parts) -> LEDWheel {
        #[rustfmt::skip]
        LEDWheel {
            nw: gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            n: gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            ne: gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            e: gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            se: gpioe.pe12.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            s: gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            sw: gpioe.pe14.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
            w: gpioe.pe15.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade(),
        }
    }

    fn by_index(&mut self, index: usize) -> &mut LED {
        let leds: [&mut LED; Self::COUNT] = [
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

struct Button {
    pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>,
    handled: bool,
}
impl Button {
    fn new(pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>) -> Button {
        Button { pin, handled: false }
    }

    fn is_pressed(&self) -> bool {
        let Ok(x) = self.pin.is_high();
        return x;
    }
    fn handle_pressed(&mut self) -> bool {
        if self.is_pressed() {
            if !self.handled {
                self.handled = true;
                return true;
            }
        } else if !self.is_pressed() {
            if self.handled {
                self.handled = false
            }
        }
        return false;
    }
}
struct Buttons {
    user: Button,
}
impl Buttons {
    fn new(mut gpioa: gpio::gpioa::Parts, _exti: &mut pac::EXTI) -> Buttons {
        Buttons {
            user: Button::new(
                gpioa
                    .pa0
                    .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr)
                    .downgrade(),
            ),
        }
    }
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
