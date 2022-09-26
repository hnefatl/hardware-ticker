use core::{cell::RefCell, ops::DerefMut};

use cortex_m::interrupt::{free, Mutex};
use stm32f3xx_hal::{gpio, interrupt, pac, syscfg};

pub struct Button {
    pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>,
    pressed: bool,
}
impl Button {
    fn new(pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>) -> Button {
        Button { pin, pressed: false }
    }

    pub fn handle_pressed(&mut self) -> bool {
        let val = self.pressed;
        self.pressed = false;
        return val;
    }
}
pub struct Buttons {
    pub user: Button,
}
impl Buttons {
    pub fn init(
        mut gpioa: gpio::gpioa::Parts,
        exti: &mut pac::EXTI,
        syscfg: &mut syscfg::SysCfg,
    ) -> &'static Mutex<RefCell<Option<Buttons>>> {
        let mut result = Buttons {
            user: Button::new(
                gpioa
                    .pa0
                    .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr)
                    .downgrade(),
            ),
        };

        // Set one of the EXTI interrupt sources to the user pin.
        syscfg.select_exti_interrupt_source(&result.user.pin);
        result.user.pin.enable_interrupt(exti);
        result.user.pin.trigger_on_edge(exti, gpio::Edge::Rising);
        // This is unsafe because critical sections relying on masks might get interrupted. Because this function
        // should be called during initialisation, we assume there's no meaningful interrupts firing yet.
        unsafe { cortex_m::peripheral::NVIC::unmask(result.user.pin.interrupt()) }

        // Update the global store.
        free(|cs| BUTTONS.borrow(cs).replace(Some(result)));

        return &BUTTONS;
    }
}

// TODO: this would be a nice way of avoiding the borrow().borrow_mut().deref_mut() magic, but need to satisfy the
// borrow checker first.
//
//struct ButtonsContainer(Mutex<RefCell<Option<Buttons>>>);
//impl ButtonsContainer {
//    pub fn borrow(&mut self, cs: &CriticalSection) -> &mut Option<Buttons> {
//        self.0.borrow(cs).borrow_mut().deref_mut()
//    }
//
//    fn update(&mut self, cs: &CriticalSection, new_buttons: Buttons) {
//        self.borrow(cs).replace(new_buttons);
//    }
//}
//
//static BUTTONS: ButtonsContainer = ButtonsContainer(Mutex::new(RefCell::new(None)));
static BUTTONS: Mutex<RefCell<Option<Buttons>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn EXTI0() {
    free(|cs| {
        if let Some(buttons) = BUTTONS.borrow(cs).borrow_mut().deref_mut() {
            buttons.user.pin.clear_interrupt();
            buttons.user.pressed = true;
        }
    });
}
