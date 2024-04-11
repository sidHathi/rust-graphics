mod graphics;
mod sdf;
mod util;

use graphics::run;

fn main() {
    pollster::block_on(run());
}
