extern crate sdl2;

use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Instant;
use sdl2::ttf::{self, Font};


struct FVec {
    x: f32, y: f32
}

impl FVec {
    pub fn zero() -> FVec { 
        FVec { x: 0.0, y: 0.0 }
    }
}

struct RectObject {
    rect: Rect,
    position: FVec,
    velocity: FVec
}

impl RectObject {
    pub fn new(rect : Rect,
               position : FVec,
               velocity : FVec) -> RectObject {
       RectObject {
           rect: rect,
           position: position,
           velocity: velocity
       }
    }

    pub fn world_rect(&self) -> Rect {
        Rect::new(
            self.position.x.floor() as i32 + self.rect.x,
            self.position.y.floor() as i32 + self.rect.y,
            self.rect.w as u32, self.rect.h as u32)
    }

    pub fn draw(&self, canvas : &mut WindowCanvas) {
        canvas.fill_rect(self.world_rect()).unwrap();
    }
}

struct GameState<'a> {
    paddle_1:   RectObject,
    paddle_2:   RectObject,
    ball:       RectObject,
    screen:     Rect,
    event:      EventPump,
    score_1:    i32,
    score_2:    i32,
    font:       Option<Font<'a, 'a>>
}


fn update(state : &mut GameState, delta_time : f32) {
    // ball movement
    state.ball.position.x += state.ball.velocity.x * delta_time;
    state.ball.position.y += state.ball.velocity.y * delta_time;

    // ball collision with paddles
    if state.ball.world_rect() & state.paddle_1.world_rect() != None{
        state.ball.velocity.x = state.ball.velocity.x.abs();
    }

    if state.ball.world_rect() & state.paddle_2.world_rect() != None{
        state.ball.velocity.x = -state.ball.velocity.x.abs();
    }

    // ball collision with top and bottom of the screen
    if state.ball.world_rect().y <= 0 {
        state.ball.velocity.y = state.ball.velocity.y.abs();
    }
    if state.ball.world_rect().y + state.ball.world_rect().h >= state.screen.h {
        state.ball.velocity.y = -state.ball.velocity.y.abs();
    }

    // ball leaves court for score
    if state.ball.position.x <= 0.0 {
        println!("right scored");
        state.score_2 += 1;
        state.ball.position.x = (state.screen.w / 2) as f32;
        state.ball.position.y = (state.screen.h / 2) as f32;
    }
    if state.ball.position.x >= state.screen.w as f32 {
        println!("left scored");
        state.score_1 += 1;
        state.ball.position.x = (state.screen.w / 2) as f32;
        state.ball.position.y = (state.screen.h / 2) as f32;
    }

    // "AI"
    state.paddle_2.position.y = state.ball.position.y;

    // move on input
    if state.event.keyboard_state().is_scancode_pressed(Scancode::Up) {
        state.paddle_1.position.y -= 1.0 * delta_time;
    }
    if state.event.keyboard_state().is_scancode_pressed(Scancode::Down) {
        state.paddle_1.position.y += 1.0 * delta_time;
    }

    // clamp paddle y positions
    state.paddle_1.position.y = state.paddle_1
        .position.y
        .clamp((state.paddle_1.rect.h/2) as f32, (state.screen.h-state.paddle_1.rect.h/2) as f32);

    state.paddle_2.position.y = state.paddle_2
        .position.y
        .clamp((state.paddle_2.rect.h/2) as f32, (state.screen.h-state.paddle_2.rect.h/2) as f32);
}

fn draw(state : &GameState, canvas : &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(255,255,255));
    state.paddle_1.draw(canvas);
    state.paddle_2.draw(canvas);
    state.ball.draw(canvas);

    let txtsurf = state.font.as_ref()
        .unwrap()
        .render(&format!("{} - {}", state.score_1, state.score_2))
        .solid(Color::RGB(255,255,255))
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let texture = txtsurf.as_texture(&texture_creator).unwrap();
    let dstrect = Rect::new(
        state.screen.w/2 - txtsurf.rect().w/2, 0,
        txtsurf.rect().w.try_into().unwrap(), txtsurf.rect().h.try_into().unwrap());
    canvas.copy(&texture, txtsurf.rect(), dstrect).unwrap();
}

fn main() {
    // context stuff
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let font = ttf::init().unwrap();

    let w = 800i32;
    let h = 600i32;

    // rendering stuff
    let window = video.window("window", w.try_into().unwrap(), h.try_into().unwrap())
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut game_state = GameState {
        // initialize scores
        score_1: 0,
        score_2: 0,
        font: Some(font.load_font("./IBMPlexMono-Regular.otf", 30).unwrap()),

        // create the player paddle
        paddle_1: RectObject::new(
            Rect::new(-25/2, -50, 25, 100),
            FVec { x: 25.0 , y: (h/2) as f32 },
            FVec::zero()
        ),

        // create the opponent paddle
        paddle_2: RectObject::new(
            Rect::new(-25/2, -50, 25, 100),
            FVec { x: (w-25) as f32, y: (h/2) as f32 },
            FVec::zero()
        ),

        // create the ball at the center
        ball: RectObject::new(
            Rect::new(-25/2, -25/2, 25, 25),
            FVec { x: (w/2) as f32, y: (h/2) as f32 },
            FVec { x: -0.25, y: 0.25 }
        ),

        // create an event pump for event handling
        event: context.event_pump().unwrap(),

        // generate the screen rect
        screen: Rect::new(
            0, 0, w as u32, h as u32
        )
    };

    // draw first frame as a black screen
    canvas.set_draw_color(Color::RGB(0,0,0));
    canvas.clear();
    canvas.present();

    // make a timer to track the delta time
    let mut frame_timer = Instant::now();

    'running: loop {
        // clear pure black, draw, present to window
        canvas.set_draw_color(Color::RGB(0,0,0));
        canvas.clear();
        draw(&game_state, &mut canvas);
        canvas.present();

        // update game logic
        update(&mut game_state,(frame_timer.elapsed().as_micros() as f32) / 1000f32);

        // reset timer
        frame_timer = Instant::now();

        // poll and process events
        for event in game_state.event.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }
    }
}
