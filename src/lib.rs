use std::mem;
use std::slice::from_raw_parts_mut;

mod bindings;
mod channel_builder;
mod error;
mod strip_type;

use channel_builder::ChannelBuilder;
use error::WS2811Error;
use strip_type::StripType;

use smart_leds_trait::{SmartLedsWrite, RGB8};

pub struct Ws2812Rpi {
    c_struct: bindings::ws2811_t,
}

impl Ws2812Rpi
{
    /// Use ws2812 devices via rpi-ws281x library
    pub fn new(led_count: i32, pin: i32) -> Result<Self, WS2811Error> {
        unsafe {
            let mut ret = Self { c_struct: mem::zeroed() };

            ret.c_struct.freq = 800_000;
            ret.c_struct.dmanum = 10;
            ret.c_struct.channel[0] = ChannelBuilder::new()
                                     .pin(pin)
                                     .count(led_count)
                                     .strip_type(StripType::Ws2811Rgb)
                                     .brightness(255)
                                     .build();
            let res: Result<(), WS2811Error> = bindings::ws2811_init(&mut ret.c_struct).into();

            match res {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            return Ok(ret);
        }

    }

}

impl Drop for Ws2812Rpi {
    fn drop(&mut self) {
        /*
         * Unsafe used here because we need to call an externed
         * function during the drop process.  Unfortunately,
         * I don't have a better way of dealing with this.
         */
        unsafe {
            bindings::ws2811_fini(&mut self.c_struct);
        }
    }
}

impl SmartLedsWrite for Ws2812Rpi
{
    type Error = error::WS2811Error;
    type Color = RGB8;
    // Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        let mut i = 0;

        let mut colors = unsafe {
            from_raw_parts_mut(
                self.c_struct.channel[0].leds as *mut [u8; 4],
                self.c_struct.channel[0].count as usize,
            )
        };

        for c in iterator {
            let c_rgb: Self::Color = c.into();
            colors[i] = [c_rgb.b, c_rgb.r, c_rgb.g, 0];
            i = i + 1;
        }

        unsafe {
            return bindings::ws2811_render(&mut self.c_struct).into();
        }

        Ok(())
    }
}
