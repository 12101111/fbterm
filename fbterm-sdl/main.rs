use clap::{value_t, App, Arg};
use fbterm::*;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::io::Read;

fn main() {
    let matches = App::new("Fbterm test on SDL2")
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .takes_value(true)
                .help("width of screen"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .takes_value(true)
                .help("height of screen"),
        )
        .arg(
            Arg::with_name("font")
                .short("f")
                .long("font")
                .takes_value(true)
                .help("path to font file"),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .takes_value(true)
                .help("height of font"),
        )
        .get_matches();
    let width = value_t!(matches, "width", usize).unwrap_or(640);
    let height = value_t!(matches, "height", usize).unwrap_or(480);
    let size = value_t!(matches, "size", f32).unwrap_or(14.0);
    let font = value_t!(matches, "font", String);
    match font {
        Ok(path) => {
            let mut file = std::fs::File::open(&path).expect("File can't open");
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).expect("Can't read file");
            println!("Load font file: {}", path);
            let font = TrueTypeFont::new(&buf, size);
            println!("load font done");
            run(width, height, font)
        }
        Err(_) => {
            let size = if size <= 12.0 {
                println!("Use VGA font 8x8");
                VGAFontConfig::VGA8x8
            } else if size <= 15.0 {
                println!("Use VGA font 8x14");
                VGAFontConfig::VGA8x14
            } else {
                println!("Use VGA font 8x16");
                VGAFontConfig::VGA8x16
            };
            run(width, height, VGAFont::new(size));
        }
    };
}

fn run<F: Font>(width: usize, height: usize, font: F) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let window = video_subsystem
        .window("fbterm-sdl", width as u32, height as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let texture_creator = canvas.texture_creator();
    let mut frame_buffer = vec![0u8; 4 * width * height];
    let mut double_buffer = vec![0u8; 4 * width * height];
    let background = RGBA8888::new(0, 0, 0xA8, 0);
    let foreground = RGBA8888::new(0xA8, 0xA8, 0xA8, 255);
    let fb = unsafe {
        Framebuffer::new(
            std::ptr::NonNull::new((&mut frame_buffer).as_mut_ptr()).expect("fb is null"),
            width,
            height,
            width,
            background,
            foreground,
        )
    };
    let mut term = Fbterm::new(fb, font);
    unsafe {
        term.framebuffer.set_double_buffer(
            std::ptr::NonNull::new((&mut double_buffer).as_mut_ptr()).expect("fb is null"),
        )
    };
    term.clear();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, width as u32, height as u32)
        .unwrap();
    term.print("ASCII:");
    for c in ' '..='~' {
        term.putc(c);
        term.flush();
    }
    term.print(
        r"
Any characters you type will be displayed on the screen.
Turn of IME before input on Windows.
本程序支持中文显示",
    );
    println!("{:?}", term.lines());
    texture.update(None, &frame_buffer, 4 * width).unwrap();
    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();
    loop {
        if let Some(e) = event_pump.poll_event() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break,
                Event::TextInput { text, .. } => {
                    term.print(&text);
                    texture.update(None, &frame_buffer, 4 * width).unwrap();
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                    println!("{:?}", term.lines());
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    match key {
                        Keycode::Backspace => term.putc('\u{08}'),
                        Keycode::KpEnter | Keycode::Return => term.putc('\n'),
                        _ => {}
                    }
                    term.flush();
                    texture.update(None, &frame_buffer, 4 * width).unwrap();
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                    println!("{:?}", term.lines());
                }
                _ => {
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                }
            }
        };
    }
}
