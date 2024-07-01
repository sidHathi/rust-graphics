mod graphics;
mod sdf;
mod util;
mod debug;
mod playground;

use graphics::run;

fn main() {
    pollster::block_on(run());
}
