#![no_std]
#![no_main]

use bootloader::{boot_info::FrameBuffer, entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

const pixel_size: usize = 4;
const character_size: usize = 10;

pub struct Screen {
    x_cursor_position: usize,
    y_cursor_position: usize,
    column_size: usize,
    row_size: usize,
    character_size: usize,
    pixel_size: usize,
}

trait Render {
    fn new() -> Self;
    fn print_char(&mut self, character: char, framebuffer: &mut FrameBuffer) {
        let column_size = self.get_column_size();
        for (index, byte) in framebuffer.buffer_mut().iter_mut().enumerate() {
            // pick column
            let (framebuffer_x, framebuffer_y) = self.get_xy_from_buffer_index(index);
            if framebuffer_x != self.get_x() {
                continue;
            }

            // pick row
            if framebuffer_y != self.get_y() {
                continue;
            }

            *byte = self.get_pixel_color(index, character);
        }
        self.inc_cursor();
    }
    fn get_xy_from_buffer_index(&self, index: usize) -> (usize, usize) {
        let column_size = self.get_column_size();
        let character_bytes = character_size * pixel_size;
        let column_index = index % column_size;
        let x = column_index / character_bytes;
        let y = index / (character_size * column_size);
        (x, y)
    }
    fn print_text(&mut self, text: &str, framebuffer: &mut FrameBuffer) {
        for character in text.chars() {
            self.print_char(character, framebuffer)
        }
    }
    fn get_pixel_color(&self, index: usize, character: char) -> u8 {
        let column_size = self.get_column_size();
        // first get the position x,y relative to the box character which is 10x10
        let x = index % (pixel_size * character_size);
        let y = (index / (column_size)) % character_size;
        let screen_character: ScreenCharacter = RenderScreenCharacter::new(character);
        screen_character.draw_char(x / pixel_size, y, character)
    }
    fn get_x(&self) -> usize;
    fn get_y(&self) -> usize;
    fn inc_cursor(&mut self);
    fn get_column_size(&self) -> usize;
}

trait RenderScreenCharacter {
    fn new(character: char) -> Self;
    fn get_character(&self) -> char;
    fn draw_char(&self, x: usize, y: usize, character: char) -> u8 {
        // have black frame for the character
        if x < 1 || x > 8 {
            return 0x00;
        }
        if y < 1 || y > 8 {
            return 0x00;
        }
        match character {
            'H' => return self.draw_h(x, y),
            'E' => return self.draw_e(x, y),
            'L' => return self.draw_l(x, y),
            'O' => return self.draw_o(x, y),
            'W' => return self.draw_w(x, y),
            'R' => return self.draw_r(x, y),
            'D' => return self.draw_d(x, y),
            ' ' => return self.draw_space(x, y),
            '!' => return self.draw_bang(x, y),

            _ => return 0xff,
        }
    }
    fn draw_h(&self, x: usize, y: usize) -> u8 {
        // draw H side lines
        if x < 3 || x > 6 {
            return 0xff;
        }
        // draw H horizontal line
        if y < 6 && y > 3 {
            return 0xff;
        }
        0x00
    }
    fn draw_e(&self, x: usize, y: usize) -> u8 {
        if y < 3 || y > 6 {
            return 0xff;
        }
        if y == 4 || y == 5 {
            return 0xff;
        }
        if x < 3 {
            return 0xff;
        }
        0x00
    }
    fn draw_l(&self, x: usize, y: usize) -> u8 {
        if x < 3 {
            return 0xff;
        }
        if y > 6 {
            return 0xff;
        }
        0x00
    }
    fn draw_o(&self, x: usize, y: usize) -> u8 {
        if x < 3 || x > 6 {
            return 0xff;
        }
        if y < 3 || y > 6 {
            return 0xff;
        }
        0x00
    }
    fn draw_w(&self, x: usize, y: usize) -> u8 {
        if y > 6 {
            return 0xff;
        }
        if x < 3 || x > 6 {
            return 0xff;
        }
        if x < 6 && x > 3 {
            return 0xff;
        }
        0x00
    }

    fn draw_r(&self, x: usize, y: usize) -> u8 {
        if x < 3 {
            return 0xff;
        }
        if y < 3 {
            return 0xff;
        }
        if (x > 6) && (y < 5) {
            return 0xff;
        }
        if y > 3 && y < 6 {
            return 0xff;
        }

        if x > 4 && (x == y || x == y + 1) {
            return 0xff;
        }
        0x00
    }
    fn draw_d(&self, x: usize, y: usize) -> u8 {
        self.draw_o(x, y)
    }
    fn draw_bang(&self, x: usize, y: usize) -> u8 {
        if (x < 6 && x > 3) && (y > 6 || y < 5) {
            return 0xff;
        }

        0x00
    }
    fn draw_space(&self, x: usize, y: usize) -> u8 {
        0x00
    }
}

struct ScreenCharacter {
    character: char,
}

impl RenderScreenCharacter for ScreenCharacter {
    fn new(c: char) -> ScreenCharacter {
        ScreenCharacter { character: c }
    }
    fn get_character(&self) -> char {
        self.character
    }
}

impl Render for Screen {
    fn new() -> Screen {
        Screen {
            x_cursor_position: 0,
            y_cursor_position: 0,
            character_size: character_size,
            pixel_size: pixel_size,
            row_size: 340 * pixel_size,
            column_size: 640 * pixel_size,
        }
    }
    fn get_x(&self) -> usize {
        self.x_cursor_position
    }
    fn get_y(&self) -> usize {
        self.y_cursor_position
    }
    fn inc_cursor(&mut self) {
        self.x_cursor_position += 1;
    }
    fn get_column_size(&self) -> usize {
        self.column_size
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // turn the screen gray
    //draw_screen(boot_info);
    let mut screen: Screen = Render::new();
    let hello = "HELLO WORLD!";
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        screen.print_text(hello, framebuffer);
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
