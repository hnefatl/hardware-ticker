use embedded_hal::digital::v2::InputPin;
use stm32f3xx_hal::{gpio, pac};

pub struct Button {
    pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>,
    handled: bool,
}
impl Button {
    fn new(pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>) -> Button {
        Button { pin, handled: false }
    }

    pub fn is_pressed(&self) -> bool {
        let Ok(x) = self.pin.is_high();
        return x;
    }
    pub fn handle_pressed(&mut self) -> bool {
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
pub struct Buttons {
    pub user: Button,
}
impl Buttons {
    pub fn new(mut gpioa: gpio::gpioa::Parts, _exti: &mut pac::EXTI) -> Buttons {
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
