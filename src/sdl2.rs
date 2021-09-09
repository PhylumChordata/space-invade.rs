use std::time::{Duration, SystemTime};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use crate::memory::Memory;
use std::thread;
use std::sync::Mutex;
use crate::emulator::Emulator;
use crate::emulator_state::EmulatorState;
use crate::listener::Listener;

const RECTANGLE_SIZE: u32 = 2;
const WHITE: Color = Color::RGB(255, 255, 255);
const BLACK: Color = Color::RGB(0, 0, 0);
const RED: Color = Color::RGB(255, 0, 0);
const GREEN: Color = Color::RGB(0, 255, 0);

pub fn sdl2(listener: &'static Mutex<EmulatorState>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("", Emulator::WIDTH as u32 * RECTANGLE_SIZE, Emulator::HEIGHT as u32 * RECTANGLE_SIZE)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let m = Memory::new(Some(listener));// as &Box<dyn GraphicRenderer + Send>);
    let mut memory = Box::new(m);
    memory.read_file("space-invaders.rom", 0);
    let mut computer = Emulator::new(memory, 0 /* pc */);
    let time_per_frame_ms = 16;

    //
    // Spawn the game logic in a separate thread. This logic will communicate with the
    // main thread (and therefore, the actual graphics on your screen) via the `listener`
    // object that this function receives in parameter.
    //
    thread::spawn(move || {
        loop {
            let start = SystemTime::now();
            // Run one frame
            let cycles = computer.run_one_frame(false);
            let elapsed = start.elapsed().unwrap().as_millis();

            // Wait until we reach 16ms before running the next frame.
            // TODO: I'm not 100% sure the event pump is being invoked on a 16ms cadence,
            // which might explain why my game is going a bit too fast. I should actually
            // rewrite this logic to guarantee that it runs every 16ms
            if elapsed < time_per_frame_ms {
                std::thread::sleep(Duration::from_millis((time_per_frame_ms - elapsed) as u64));
            }
            let after_sleep = start.elapsed().unwrap().as_micros();
            if false {
                println!("Actual time frame: {}ms, after sleep: {} ms, cycles: {}",
                         elapsed,
                         after_sleep,
                         cycles);
            }

            listener.lock().unwrap().set_megahertz(cycles as f64 / after_sleep as f64);
        }
    });


    canvas.clear();
    canvas.present();
    let mut last_title_update = SystemTime::now();

    // Main game loop
    'running: loop {
        for event in event_pump.poll_iter() {

            //
            // Read the keyboard
            //
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    => break 'running,
                // Pause / unpause ('p')
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    let mut l = listener.lock().unwrap();
                    if l.is_paused() {
                        l.unpause();
                    } else {
                        l.pause();
                    }
                },

                // Insert coin
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    listener.lock().unwrap().set_bit_in_1(0, true);
                },
                Event::KeyUp { keycode: Some(Keycode::C), .. } => {
                    listener.lock().unwrap().set_bit_in_1(0, false);
                },
                // Start 2 players
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    listener.lock().unwrap().set_bit_in_1(1, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Num2), .. } => {
                    listener.lock().unwrap().set_bit_in_1(1, false);
                },
                // Start 1 player
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    listener.lock().unwrap().set_bit_in_1(2, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Num1), .. } => {
                    listener.lock().unwrap().set_bit_in_1(2, false);
                },
                // Player 1 shot
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    if listener.lock().unwrap().is_paused() {
                        listener.lock().unwrap().unpause();
                    } else {
                        listener.lock().unwrap().set_bit_in_1(4, true);
                    }
                },
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => {
                    listener.lock().unwrap().set_bit_in_1(4, false);
                },
                // Player 1 move left
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    listener.lock().unwrap().set_bit_in_1(5, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                    listener.lock().unwrap().set_bit_in_1(5, false);
                },
                // Player 1 move right
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    listener.lock().unwrap().set_bit_in_1(6, true);
                },
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                    listener.lock().unwrap().set_bit_in_1(6, false);
                },

                // Player 2 shot ('s')
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    listener.lock().unwrap().set_bit_in_2(4, true);
                },
                Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                    listener.lock().unwrap().set_bit_in_2(4, false);
                },
                // Player 2 move left ('a')
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    listener.lock().unwrap().set_bit_in_2(5, true);
                },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                    listener.lock().unwrap().set_bit_in_2(5, false);
                },
                // Player 2 move right ('d')
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    listener.lock().unwrap().set_bit_in_2(6, true);
                },
                Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                    listener.lock().unwrap().set_bit_in_2(6, false);
                },
                // If the emulator is paused, any key will unpause it
                Event::KeyDown { .. } => {
                    if listener.lock().unwrap().is_paused() {
                        listener.lock().unwrap().unpause();
                    }
                }
                _ => {
                }
            }
        }

        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        canvas.clear();

        //
        // Draw the graphic
        // Simply map the listener's frame buffer (updated by the main logic in a separate thread)
        // to the SDL canvas
        //
        let mut i: usize = 0;
        for ix in 0..Emulator::WIDTH {
            for iy in (0..Emulator::HEIGHT).step_by(8) {
                let mut byte = listener.lock().unwrap().byte_color(i);
                i += 1;
                for b in 0..8 {
                    let x: i32 = ix as i32 * RECTANGLE_SIZE as i32;
                    let y: i32 = (Emulator::HEIGHT as i32 - (iy as i32+ b)) * RECTANGLE_SIZE as i32;
                    let color = if byte & 1 == 0 { BLACK } else {
                        if iy > 200 && iy < 220 { RED }
                        else if iy < 80 { GREEN }
                        else { WHITE }
                    };
                    byte >>= 1;

                    canvas.set_draw_color(color);
                    canvas.fill_rect(Rect::new(x, y, RECTANGLE_SIZE as u32, RECTANGLE_SIZE as u32))
                        .unwrap();
                }
            }
        }

        if last_title_update.elapsed().unwrap().gt(&Duration::from_millis(500)) {
            let paused = if listener.lock().unwrap().is_paused() { " - Paused" } else { "" };
            canvas.window_mut().set_title(
                format!("space-invade.rs - Cédric Beust - {:.2} Mhz{}",
                        listener.lock().unwrap().get_megahertz(),
                        paused)
                    .as_str()).unwrap();
            last_title_update = SystemTime::now();
        }

        canvas.present();
    }

    Ok(())
}
