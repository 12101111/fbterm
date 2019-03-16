use clap::{value_t, App, Arg};
use fbterm::*;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
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
                .help("height of font")
                .possible_values(&["8", "14", "16"]),
        )
        .get_matches();
    let font = if let Some(f) = matches.value_of("font") {
        match f {
            "8" => Fonts::VGA8x8,
            "14" => Fonts::VGA8x14,
            "16" => Fonts::VGA8x16,
            _ => Fonts::VGA8x14,
        }
    } else {
        Fonts::VGA8x14
    };
    let width = value_t!(matches, "width", usize).unwrap_or(640);
    let height = value_t!(matches, "height", usize).unwrap_or(480);
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
    let background = RGBA8888::new(0, 0, 0xA8, 0);
    let foreground = RGBA8888::new(0xA8, 0xA8, 0xA8, 255);
    let fb = unsafe {
        Framebuffer::new(
            (&mut frame_buffer).as_mut_ptr(),
            width,
            height,
            background,
            foreground,
        )
    };
    let mut fbterm = Fbterm::new(fb, font);
    fbterm.clear();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, width as u32, height as u32)
        .unwrap();
    fbterm.print("Show all characters:\n");
    for i in 0..255 {
        fbterm.putc(i);
    }
    fbterm.print(
        r"
Test finish.
Any characters you type will be displayed on the screen.
Enter and backspace not supported currently.
IME also not supported
",
    );
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
                    fbterm.putc(text.as_bytes()[0]);
                    texture.update(None, &frame_buffer, 4 * width).unwrap();
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                }
                _ => {}
            }
        };
    }
}
