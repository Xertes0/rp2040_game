use cortex_m::delay::Delay;

use crate::pcd8544::PCD8544;

pub fn inverse_blink(
    pcd:    &mut PCD8544,
    delay:  &mut Delay,
    delay_time:  u32,
    times:  u32)
{
    for _ in 0..2*times {
        pcd.inverse();
        pcd.draw();
        delay.delay_ms(delay_time);
    }
}
