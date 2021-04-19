use std::error::Error;
use sliders::Sliders;

fn main() -> Result<(), Box<dyn Error>> {
    Sliders::from_args()?.run()
}
