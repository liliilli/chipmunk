
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

#[derive(PartialEq)]
pub enum PixelState {
    Drawn,
    Erased,
}

pub struct DrawMessage {
    pub pos: (u8, u8),
    pub state: PixelState,
}

pub struct Screen {
    screen_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            screen_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    fn draw_xor(&mut self, (x, y): (u8, u8)) -> PixelState {
        let px = &mut self.screen_buffer[(y as usize) * SCREEN_WIDTH + (x as usize)];
        if *px == true {
            *px = false;
            PixelState::Erased
        } else {
            *px = true;
            PixelState::Drawn
        }
    }

    pub fn draw(&mut self, (mut x, mut y): (u8, u8), bytes: &[u8]) -> (Vec<DrawMessage>, bool) {
        let mut result = Vec::<DrawMessage>::new();
        let mut is_any_erased = false;

        x %= SCREEN_WIDTH as u8;
        y %= SCREEN_HEIGHT as u8;

        let origx = x;
        for byte in bytes {
            for i in (0..8).rev() {
                if byte & (0b01 << i) != 0x00 { // XORDraw flag
                    let state = self.draw_xor((x, y));
                    is_any_erased |= state == PixelState::Erased;
                    result.push(DrawMessage{pos: (x, y), state});
                } 

                x = (x + 1) % (SCREEN_WIDTH as u8);
            }

            x = origx;
            y = (y + 1) % (SCREEN_HEIGHT as u8);
        }

        (result, is_any_erased)
    }

    pub fn clear(&mut self) {
        for pixel in self.screen_buffer.iter_mut() {
            *pixel = false;
        }
    }
}


