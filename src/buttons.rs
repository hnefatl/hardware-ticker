use core::cell::RefCell;

use cortex_m::interrupt::{free, CriticalSection, Mutex};
use stm32f3xx_hal::{gpio, interrupt, pac, syscfg};

pub struct Button {
    pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>,
    pressed: bool,
}
impl Button {
    fn new(pin: gpio::Pin<gpio::Gpioa, gpio::Ux, gpio::Input>) -> Button {
        Button { pin, pressed: false }
    }

    /// Return `true` if the button has been pressed _since the last call_.
    pub fn handle_pressed(&mut self) -> bool {
        let val = self.pressed;
        self.pressed = false;
        return val;
    }
}
pub struct Buttons {
    /// The "user" pushbutton on the board.
    pub user: Button,
}
impl Buttons {
    pub fn init(
        mut gpioa: gpio::gpioa::Parts,
        exti: &mut pac::EXTI,
        syscfg: &mut syscfg::SysCfg,
    ) -> &'static ButtonsContainer {
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
        free(|cs| BUTTONS.update(cs, result));

        return &BUTTONS;
    }
}

static BUTTONS: ButtonsContainer = ButtonsContainer(Mutex::new(RefCell::new(None)));

/// Convenience wrapper that provides safety guarantees to users of this module.
/// - A reference to a `ButtonsContainer` object can only be acquired by a function that initialises it.
/// - Only one `ButtonsContainer` object can be initialised (constructing functions panic otherwise).
/// - `with_ref` provides a passthrough to the raw `Buttons` that's always valid (always initialised).
pub struct ButtonsContainer(Mutex<RefCell<Option<Buttons>>>);
impl ButtonsContainer {
    /// Runs a function that can interact with a Buttons instance.
    pub fn with_ref<F>(&self, cs: &CriticalSection, f: F)
    where
        F: FnOnce(&mut Buttons),
    {
        if let Some(buttons) = self.0.borrow(cs).borrow_mut().as_mut() {
            f(buttons)
        } else {
            panic!("buttons used before initialised")
        }
    }
    /// Boilerplate reducer that creates a critical section for the user.
    pub fn with_ref_cs<F>(&self, f: F)
    where
        F: FnOnce(&mut Buttons),
    {
        free(|cs| self.with_ref(cs, f))
    }

    /// Sets the inner `Buttons` value. Can only be called once.
    fn update(&self, cs: &CriticalSection, new_buttons: Buttons) {
        if let Some(_) = self.0.borrow(cs).borrow_mut().replace(new_buttons) {
            panic!("ButtonsContainer::update called more than once, programmer error.")
        }
    }
}

#[interrupt]
fn EXTI0() {
    BUTTONS.with_ref_cs(|buttons| {
        buttons.user.pin.clear_interrupt();
        buttons.user.pressed = true;
    });
}
