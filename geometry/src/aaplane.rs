#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

pub const fn get_axis() -> [Axis; 3] {
    [Axis::X, Axis::Y, Axis::Z]
}

impl From<u8> for Axis {
    fn from(mut value: u8) -> Self {
        value = value.min(2);
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AAPlane {
    pub coord: f64,
    pub axis: Axis,
}
