// main.rs

#![no_std]
#![no_main]
#![allow(unused_mut)]
// #![deny(warnings)]

use cortex_m_rt::entry;
use panic_halt as _;

// Supported hardware:
//
// BluePill
// https://www.st.com/en/microcontrollers-microprocessors/stm32f103.html
//
// BlackPill (STM32F401CC / STM32F411CE)
// https://www.st.com/en/microcontrollers-microprocessors/stm32f401cc.html
// https://www.st.com/en/microcontrollers-microprocessors/stm32f411ce.html
//
// Nucleo f411
// https://www.st.com/en/microcontrollers-microprocessors/stm32f411re.html
//
// nRF52840-Dongle
// https://www.nordicsemi.com/Products/Development-hardware/nRF52840-Dongle
//
// nRF52840-DK
// https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK

#[cfg(feature = "stm32f103")]
use stm32f1xx_hal as hal;

#[cfg(any(feature = "stm32f401", feature = "stm32f411"))]
use stm32f4xx_hal as hal;

#[cfg(feature = "nrf52840")]
use nrf52840_hal as hal;

#[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
use crate::hal::prelude::*;
use crate::hal::{gpio::*, pac};
use embedded_hal::delay::DelayNs;
#[cfg(feature = "nrf52840")]
use embedded_hal::digital::OutputPin;

trait IOPin {
    fn high(&mut self);
    fn low(&mut self);
}

#[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
impl IOPin for ErasedPin<Output<PushPull>> {
    fn high(&mut self) {
        self.set_high();
    }
    fn low(&mut self) {
        self.set_low();
    }
}

#[cfg(feature = "nrf52840")]
impl IOPin for Pin<Output<PushPull>> {
    fn high(&mut self) {
        self.set_high().ok();
    }
    fn low(&mut self) {
        self.set_low().ok();
    }
}

#[entry]
fn main() -> ! {
    let (mut led1, mut led2, mut delay) = init();
    let mut led_index = 0;

    loop {
        for _i in 1..=3 {
            set_leds(&mut led1, &mut led2, &mut led_index, true);
            delay.delay_ms(100);

            set_leds(&mut led1, &mut led2, &mut led_index, false);
            delay.delay_ms(400);
        }
        delay.delay_ms(1000);
    }
}

fn init() -> (impl IOPin, Option<[impl IOPin; 3]>, impl DelayNs) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Enable clock output MCO
    #[cfg(feature = "stm32f103")]
    dp.RCC.cfgr().modify(|_r, w| w.mco().sysclk());

    // Enable clock outputs 1+2
    // MCO1 = HSI / 5 (3.2 MHz); MCO2 = SYSCLK / 5 (16.8 MHz on F401, 20 MHz on F411)
    #[cfg(any(feature = "stm32f401", feature = "stm32f411"))]
    dp.RCC.cfgr().modify(|_r, w| {
        w.mco1pre().div5();
        w.mco1().hsi();
        w.mco2pre().div5();
        w.mco2().sysclk();
        w
    });

    #[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
    let rcc = dp.RCC.constrain();

    #[cfg(feature = "stm32f103")]
    let mut flash = dp.FLASH.constrain();

    #[cfg(feature = "nrf52840")]
    let p0 = hal::gpio::p0::Parts::new(dp.P0);
    #[cfg(any(feature = "nrf52840_dongle", feature = "nrf52840_dongle_sd"))]
    let p1 = hal::gpio::p1::Parts::new(dp.P1);

    // Setup system clocks
    #[cfg(feature = "stm32f103")]
    let mut rcc = rcc.freeze(
        hal::rcc::Config::hse(8.MHz()) // Use High Speed External 8 MHz crystal oscillator
            .sysclk(72.MHz()) // Use the PLL to multiply SYSCLK to 72MHz
            .hclk(72.MHz()) // Leave AHB prescaler at /1
            .pclk1(36.MHz()) // Use the APB1 prescaler to divide the clock to 36MHz (max supported)
            .pclk2(72.MHz()) // Leave the APB2 prescaler at /1
            .adcclk(12.MHz()), // ADC prescaler of /6 (max speed of 14MHz, but /4 gives 18MHz)
        &mut flash.acr,
    );

    // Setup system clock to 100 MHz
    #[cfg(feature = "nucleo_f411")]
    // default, internal rc osc
    let mut rcc = rcc.freeze(hal::rcc::Config::hsi().sysclk(100.MHz()));
    // for external xtal, use:
    // let mut rcc = rcc.freeze(hal::rcc::Config::hse(8.MHz()).sysclk(100.MHz()));

    #[cfg(feature = "black_pill")]
    let mut rcc = rcc.freeze(hal::rcc::Config::hse(25.MHz()).sysclk(100.MHz()));

    // Black Pill F401CC: 25 MHz crystal, limited to 84 MHz SYSCLK.
    #[cfg(feature = "black_pill_f401")]
    let mut rcc = rcc.freeze(hal::rcc::Config::hse(25.MHz()).sysclk(84.MHz()));

    #[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
    let mut pa = dp.GPIOA.split(&mut rcc);
    #[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
    let mut pc = dp.GPIOC.split(&mut rcc);

    // Clock outputs is alt function on MCO=PA8
    #[cfg(feature = "stm32f103")]
    pa.pa8
        .into_alternate_push_pull(&mut pa.crh)
        .set_speed(&mut pa.crh, IOPinSpeed::Mhz50);

    // Clock outputs are as alt functions on MCO1=PA8, MCO2=PC9
    #[cfg(any(feature = "stm32f401", feature = "stm32f411"))]
    pa.pa8.into_alternate::<0>().set_speed(Speed::VeryHigh);
    #[cfg(any(feature = "stm32f401", feature = "stm32f411"))]
    pc.pc9.into_alternate::<0>().set_speed(Speed::VeryHigh);

    // On Bluepill stm32f103 user led is on PC13, active low
    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    #[cfg(feature = "blue_pill")]
    let led1 = pc.pc13.into_push_pull_output(&mut pc.crh).erase();

    // On both Black Pill variants the user LED is on PC13, active low
    #[cfg(any(feature = "black_pill", feature = "black_pill_f401"))]
    let led1 = pc.pc13.into_push_pull_output().erase();

    // On Nucleo stm32f411 User led LD2 is on PA5, active high
    #[cfg(feature = "nucleo_f411")]
    let led1 = pa.pa5.into_push_pull_output().erase();

    #[cfg(any(feature = "nrf52840_dongle", feature = "nrf52840_dongle_sd"))]
    let led1 = p0.p0_06.into_push_pull_output(Level::High).degrade();
    #[cfg(feature = "nrf52840_dk")]
    let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();

    #[cfg(any(feature = "nrf52840_dongle", feature = "nrf52840_dongle_sd"))]
    let led2 = Some([
        p0.p0_08.into_push_pull_output(Level::High).degrade(),
        p1.p1_09.into_push_pull_output(Level::High).degrade(),
        p0.p0_12.into_push_pull_output(Level::High).degrade(),
    ]);
    #[cfg(feature = "nrf52840_dk")]
    let led2 = Some([
        p0.p0_14.into_push_pull_output(Level::High).degrade(),
        p0.p0_15.into_push_pull_output(Level::High).degrade(),
        p0.p0_16.into_push_pull_output(Level::High).degrade(),
    ]);

    // Sigh, keeping compiler happy with explicit type
    #[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
    let led2: Option<[ErasedPin<Output<PushPull>>; 3]> = None;

    // Create a delay abstraction based on SysTick
    #[cfg(any(feature = "stm32f103", feature = "stm32f401", feature = "stm32f411"))]
    let delay = hal::timer::Timer::syst(cp.SYST, &rcc.clocks).delay();
    #[cfg(feature = "nrf52840")]
    let delay = hal::delay::Delay::new(cp.SYST);

    (led1, led2, delay)
}

fn set_leds(
    led1: &mut impl IOPin,
    led2: &mut Option<[impl IOPin; 3]>,
    led_index: &mut usize,
    state: bool,
) {
    #[cfg(any(
        feature = "black_pill",
        feature = "black_pill_f401",
        feature = "blue_pill",
        feature = "nrf52840"
    ))]
    let active_low = true;
    #[cfg(feature = "nucleo_f411")]
    let active_low = false;

    if state ^ active_low {
        led1.high();
        if let Some(l2) = led2 {
            l2.iter_mut().for_each(|l| l.high());
        }
    } else {
        led1.low();
        if let Some(l2) = led2 {
            l2[*led_index].low();
            *led_index = (*led_index + 1) % l2.len();
        }
    }
}
// EOF
