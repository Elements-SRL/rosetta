const RAM_LEN: usize = 2048;

pub struct Ram {
    board_id: u32,
    raw: [u8; RAM_LEN],
}

impl Ram {
    pub fn new(board_id: u32) -> Self {
        Self {
            board_id,
            raw: [0; RAM_LEN],
        }
    }
}
