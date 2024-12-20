use minifb::{Key, Window, WindowOptions, MouseMode, MouseButton};


const HEIGHT: usize = 500;
const WIDTH: usize = 500;

#[derive(Clone)]
struct Dimensions {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize
}

impl Dimensions {
    fn new(left: usize, top: usize, right: usize, bottom: usize) -> Self {
        Self {
            left,
            top,
            right,
            bottom
        }
    }

    fn contains(&self, pos: impl Into<Position>) -> bool {
        let pos = pos.into();
        pos.x >= self.left && pos.x <= self.right
        && pos.y <= self.bottom && pos.y >= self.top
    }
}

#[derive(Clone, Copy)]
struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y
        }
    }
}

impl From<[usize; 2]> for Position {
    fn from(value: [usize; 2]) -> Position {
        Position {
            x: value[0],
            y: value[1]
        }
    }
}

impl From<&[usize; 2]> for Position {
    fn from(value: &[usize; 2]) -> Position {
        Position {
            x: value[0],
            y: value[1]
        }
    }
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Position {
        Position {
            x: value.0,
            y: value.1
        }
    }
}

// TODO: write funcs for generating images based on argb u8 values and adjusting
// the values of the image buffer as converted to hex then u32 values
// THEN it will be possible to do more advanced rendering based on positions and texture overlays
// (Also, it may be useful to have a function which converts from a u32 pixel value to an argb set)

#[derive(Clone)]
struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }
}

impl From<&Color> for [u8; 4] {
    fn from(value: &Color) -> [u8; 4] {
        [
            value.a,
            value.r, 
            value.g,
            value.b,
        ]
    }
}

impl From<Color> for [u8; 4] {
    fn from(value: Color) -> [u8; 4] {
        [
            value.a,
            value.r, 
            value.g,
            value.b,
        ]
    }
}

impl From<&Color> for Color {
    fn from(value: &Color) -> Color {
        value.clone()
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Color {
        Color::new(value[0], value[1], value[2], value[3])
    }
}

impl From<&[u8; 4]> for Color {
    fn from(value: &[u8; 4]) -> Color {
        Color::new(value[0], value[1], value[2], value[3])
    }
}

impl From<&Color> for u32 {
    fn from(value: &Color) -> u32 {
        use std::fmt::Write;
        let bytes: [u8; 4] = value.into();
        let mut s = String::with_capacity(8);
        for &byte in bytes.iter() {
            s += format!("{:02x?}", byte).as_str()
        }

        //println!("got values {:?}, string {}", &bytes, s);
        u32::from_str_radix(&s, 16).expect("Invalid bytes in Color struct")
    }
}
impl From<Color> for u32 {
    fn from(value: Color) -> u32 {
        use std::fmt::Write;
        let bytes: [u8; 4] = value.into();
        let mut s = String::with_capacity(8);
        for &byte in bytes.iter() {
            s += format!("{:02x?}", byte).as_str()
        }

        //println!("got values {:?}, string {}", &bytes, s);
        u32::from_str_radix(&s, 16).expect("Invalid bytes in Color struct")
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Color {

        let mut s = format!("{:x}", value);
        //println!("got hex val {}", s);

        let mut a = 0;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        for (i, val) in s.chars()
            .collect::<Vec<char>>()
            .chunks(2).enumerate() {
            let mut_ref = match i {
                0 => &mut a,
                1 => &mut r,
                2 => &mut g,
                _ => &mut b,
            };
            let val = val.iter().collect::<String>();
            //println!("got chunk {}", val);
            *mut_ref = u8::from_str_radix(&val, 16).expect("Invalid hex chunk in hex to Color conversion");
        }
        Color {
            a,
            r,
            g,
            b,
        }
    }
}

trait Blend {
    fn blend(&mut self, other_pixel: u32);
}
impl Blend for u32 {
    fn blend(&mut self, other_pixel: u32) {
        let mut current_color: Color = (*self).into();
        let other_color: Color = other_pixel.into();

        let new_alpha = other_color.a;
        // becomes percentage, basically
        let new_alpha = new_alpha as f32 / 255.0;
        if new_alpha < 1.0 {
            current_color.r = ((other_color.r as f32 * new_alpha) + (current_color.r as f32 * (1.0 - new_alpha))) as u8;
            current_color.g = ((other_color.g as f32 * new_alpha) + (current_color.g as f32 * (1.0 - new_alpha))) as u8;
            current_color.b = ((other_color.b as f32 * new_alpha) + (current_color.b as f32 * (1.0 - new_alpha))) as u8;
        } else {
            current_color.r = other_color.r;
            current_color.g = other_color.g;
            current_color.b = other_color.b;
        }
        current_color.a = 255;
        *self = current_color.into();
    }
}

trait Ui {
    fn draw_button(&mut self, dims: &Dimensions, color: impl Into<Color>, buffer_width: usize, buffer_height: usize);
}

impl Ui for Vec<u32> {
    fn draw_button(&mut self, dims: &Dimensions, color: impl Into<Color>, buffer_width: usize, buffer_height: usize) {

        let color: Color = color.into();

        if dims.left >= dims.right {
            panic!("left coordinate must be less than right coordinate")
        }
        if dims.top >= dims.bottom {
            panic!("top coordinate must be less than bottom coordinate")
        }
        for x in dims.left..=dims.right {
            for y in dims.top..=dims.bottom {
                if x < buffer_width && y < buffer_height {
                    self[(y * buffer_width) + x] = u32::from(&color);
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum VirtualScreen {
    Main,
    Options,
    Default
}

fn main() {
    let start = std::time::SystemTime::now();
    println!("Hello, window template!");
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
        ).unwrap_or_else(|e| {
            panic!("{}", e)
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    // buttons (TODO: Use button data struct to pass to draw_button)
    let options_button_area = Dimensions::new(0, 0, WIDTH/10, HEIGHT/10);
    let exit_button_area = Dimensions::new(WIDTH-(WIDTH/10), 0, WIDTH, HEIGHT/10);

    let mut which_screen = VirtualScreen::Main;


    let mut left_mouse_down = false;
    let mut lmd_previous = false;

    let mut middle_mouse_down = false;
    let mut mmd_previous = false;

    let mut right_mouse_down = false;
    let mut rmd_previous = false;

    let mut previous_state = 0;
    let mut state = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        previous_state = state;
        state = (start.elapsed().unwrap().as_millis() / 90 % 255);
        if state != previous_state {
            println!("state = {}", state);
        }
        for i in buffer.iter_mut() {
            *i = Color::new(255, 0,0,0).into(); // write something more funny here!
        }

        let options_button_color: Color = Color::new(0,255,255,0);
        let exit_button_color: Color = Color::new((state % 255) as u8,255,0,0);
        if which_screen == VirtualScreen::Main {
            buffer.draw_button(&options_button_area, options_button_color.clone(), WIDTH, HEIGHT);
        }

        buffer.draw_button(&exit_button_area, exit_button_color.clone(), WIDTH, HEIGHT);

        //let test_color: Color = 4294901760u32.into();
        //println!("test_color = a: {} r: {} g: {} b: {}", test_color.a, test_color.r, test_color.g, test_color.b);


        if window.get_mouse_down(MouseButton::Left) {
            left_mouse_down = true;
            lmd_previous = true;
        } else {
            left_mouse_down = false;
        }
        if lmd_previous && !left_mouse_down {
            // mouse up detected, reset boolean
            lmd_previous = false;
            // discard any mouse clicks outside of window
            if let Some(pos) = window.get_mouse_pos(MouseMode::Discard) {
                // rounds position to nearest integer
                let (mx, my) = ((pos.0 + 0.5) as usize, (pos.1 + 0.5) as usize);

                use VirtualScreen::*;
                
                // check all (valid) buttons and act on them

                match which_screen {
                    Main => {
                        if exit_button_area.contains([mx, my]) {
                            break;
                        }
                        if options_button_area.contains([mx, my]) {
                            println!("Entering options menu");
                            which_screen = Options;
                        }
                    },
                    Options => {
                        if exit_button_area.contains([mx, my]) {
                            println!("Exiting options menu");
                            which_screen = Main;
                        }
                    }
                    _ => {},
                }
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();

    }

}
