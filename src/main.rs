use nannou::{prelude::*, rand, image, wgpu, color};

fn main() {
    println!("Starting");

    nannou::app(Model::new)
        .update(update)
        .view(view)
        .loop_mode(LoopMode::RefreshSync)
        .run();
}

type Px = image::Rgba<u8>;
type ImgBuf = image::ImageBuffer<Px, Vec<u8>>;

struct Ball {
    pos: Vec2,
    rad: f32,
    dst: Vec2,
    dstdst: Vec2,
    spd: f32,
    col: Px,
}

const EPS: f32 = 1.0;
const WIDTH: u32 = 720;
const HEIGHT: u32 = 720;
const WIDTHHALF: i32 = (WIDTH / 2) as i32;
const HEIGHTHALF: i32 = (HEIGHT / 2) as i32;
const NUMBALL: usize = 42;

impl Ball {
    fn new() -> Self {
        Ball {
            pos: Vec2::new(0.0, 0.0),
            rad: 9.0,
            dst: Vec2::new(0.0, 0.0),
            dstdst: Vec2::new(0.0, 0.0),
            spd: 150.0,
            col: image::Rgba::<u8>([
                random_range(210, 255),
                random_range(210, 255),
                random_range(210, 255),
                255,
            ]),
        }
    }
}

struct Model {
    balls: Vec<Ball>,
    trails: ImgBuf,
}

impl Model {
    fn new(app: &App) -> Self {
        // initialize the app
        // create initial window for prototyping
        // we can change to drawing pngs later
        app.new_window()
            .size(WIDTH, HEIGHT)
            .build()
            .unwrap();

        // initialize our model
        // spawn the stuff
        let mut balls = Vec::new();
        for _ in 0..NUMBALL {
            balls.push(Ball::new());
        }

        Model {
            balls,
            trails: ImgBuf::new(WIDTH, HEIGHT),
        }
    }
}

fn px2v3(px: &Px) -> Vec3 {
    Vec3::new(
        px.0[0] as f32 / 255.0,
        px.0[1] as f32 / 255.0,
        px.0[2] as f32 / 255.0,
    )
}

fn v32px(v: Vec3) -> Px {
    image::Rgba::<u8>([
        (v.x * 255.0) as u8,
        (v.y * 255.0) as u8,
        (v.z * 255.0) as u8,
        255
    ])
}

fn rndv3(mag: f32) -> Vec3 {
    // not really uniform random here, but eh
    let v = Vec3::new(
        random_range(0.0, 1.0),
        random_range(0.0, 1.0),
        random_range(0.0, 1.0),
    );

    v.normalize() * mag
}

const MAG_SPREAD: f32 = 0.03;
const MODIFY_MAG: f32 = 0.03;
const DARKEN_MAG: f32 = 0.81;

fn step_trails(buf: &mut ImgBuf) {
    // spread ideas
    //  if bright enough:
    //  randomize a step from starting color and spread if there is a sufficiently different color to spread over
    //  absorb a touch of the base color, and also darken by a randomized amount
    //  darken the left behind square to not be bright enough to spread anymore
    // slowly darken everything else

    let mut dirtymap: [[bool; WIDTH as usize]; HEIGHT as usize] = [[false; WIDTH as usize]; HEIGHT as usize];

    // this is a terribly slow way to do this. Really we should do this on the GPU in a shader, not on the CP
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if dirtymap[y as usize][x as usize] {
                continue;
            }

            let mut v = {
                px2v3(buf.get_pixel(x, y))
            };


            let mag = v.length_squared();

            if mag >= MAG_SPREAD {
                // modify
                v = v - rndv3(MODIFY_MAG);

                // spread
                //for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (1,1), (1,-1), (-1,-1), (-1,1)] {
                    let nx = (x as i32) + dx;
                    let ny = (y as i32) + dy;
                    if nx < 0 || nx >= WIDTH as i32 || ny < 0 || ny >= HEIGHT as i32 {
                        continue;
                    }

                    let nx = nx as u32;
                    let ny = ny as u32;

                    if dirtymap[ny as usize][nx as usize] {
                        continue;
                    }

                    dirtymap[ny as usize][nx as usize] = true;

                    let nei = buf.get_pixel_mut(nx, ny);

                    let nv = px2v3(nei);
                    if nv.length_squared() < mag {
                        *nei = v32px(v);

                        // need to mark this one as changed already, so we skip it later

                    }
                }
            }

            // darken
            v *= DARKEN_MAG;

            // restore
            let np = v32px(v);
            
            let p = buf.get_pixel_mut(x, y);
            *p = np;
        }
    }

}

fn update(_app: &App, state: &mut Model, upd: Update) {
    // update state each step here

    // for rendering to pics, make this constant
    let _dlt = upd.since_last.as_secs_f32();
    let dlt = 0.009;

    // do the spread rules on each pixel
    step_trails(&mut state.trails);

    // move the balls
    for b in &mut state.balls {
        if b.pos.distance_squared(b.dst) <= EPS {
            // get a new random dst dst
            // and a new random dst
            let newdstdst = Vec2::new(rand::random_range((-(WIDTHHALF-1))as f32, (WIDTHHALF-1) as f32) , rand::random_range((-(HEIGHTHALF-1)) as f32, (HEIGHTHALF-1) as f32));
            let newdst = Vec2::new(rand::random_range((-(WIDTHHALF-1))as f32, (WIDTHHALF-1) as f32) , rand::random_range((-(HEIGHTHALF-1)) as f32, (HEIGHTHALF-1) as f32));

            b.dstdst = newdstdst;
            b.dst = newdst;
        }

        // move dst toward dstdst linearly
        //b.dst = b.dst.lerp(b.dstdst, 0.03);
        let toward = (b.dstdst - b.dst).normalize() * b.spd * 1.5 *  dlt;

        b.dst = b.dst + toward;


        // move pos toward dst linearly
        let toward = (b.dst - b.pos).normalize() * b.spd * dlt;

        b.pos = b.pos + toward;

        //TODO add to the trails
        let px = state.trails.get_pixel_mut(((b.pos.x as i32) + WIDTHHALF) as u32, (HEIGHTHALF - (b.pos.y as i32)) as u32);
        *px = b.col;
    }

}

fn view(app: &App, state: &Model, frame: Frame) {
    // draw out our stuff
    let d = app.draw();
    let win = app.window_rect();

    if frame.nth() == 0 { 
        println!("{win:?}");
    }

    d.background().color(BLACK);

    // draw the trails
    let window = app.window(app.window_id()).unwrap();
    d.texture(&wgpu::Texture::load_from_image_buffer(
        window.device(),
        window.queue(),
        wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
        &state.trails,
    ));

    for b in &state.balls {
        d.ellipse()
            .x_y(b.pos.x, b.pos.y)
            .radius(b.rad)
            .color(color::rgb8(b.col.0[0], b.col.0[1], b.col.0[2]));
    }

    d.to_frame(app, &frame).unwrap();

    // capture
    let fp = app.project_path().unwrap().join(format!("{:04}.png", frame.nth()));
    app.main_window().capture_frame(fp);
}