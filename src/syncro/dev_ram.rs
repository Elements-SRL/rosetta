const RAM_LEN: usize = 2048;

pub struct Ram([u8; RAM_LEN]);

impl Ram {
    pub fn new(content: [u8; RAM_LEN]) -> Self {
        Self(content)
    }
}
