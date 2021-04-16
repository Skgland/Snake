use conrod_core::{image::Map, text::GlyphCache, Ui};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use piston_window::*;
use std::path::PathBuf;

mod app;
mod game;
mod gui;

use app::*;
use gui::*;

//
//Initial Setting
//

// Change this to OpenGL::V2_1 if not working.

const OPEN_GL_VERSION: OpenGL = OpenGL::V3_2;
const INIT_WIDTH: u32 = 200;
const INIT_HEIGHT: u32 = 200;

fn main() {
    let mut window = create_window();

    let ui = create_ui();

    println!("Construction app!");
    // Create a new game and run it.
    let mut app = create_app(ui);

    println!("Creating render Context!");
    let mut context = create_render_context();

    println!("Creating event loop iterator");
    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        e.render(|r| app.render(&mut context, r));

        if let Event::Input(i, _) = e {
            app.input(i);
        } else {
            e.update(|_| app.update(&mut window));
        }
    }
}

struct TextCache<'font> {
    text_vertex_data: Vec<u8>,
    glyph_cache: GlyphCache<'font>,
    text_texture_cache: Texture,
}

fn create_text_cache<'font>(_: &()) -> TextCache {
    // Create a texture to use for efficiently caching text on the GPU.
    let text_vertex_data: Vec<u8> = Vec::new();
    let (glyph_cache, text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod_core::text::GlyphCache::builder()
            .dimensions(INIT_WIDTH, INIT_HEIGHT)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build();
        let buffer_len = INIT_WIDTH as usize * INIT_HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture =
            opengl_graphics::Texture::from_memory_alpha(&init, INIT_WIDTH, INIT_HEIGHT, &settings)
                .expect("Failed to load Texture!");
        (cache, texture)
    };
    TextCache {
        text_vertex_data,
        glyph_cache,
        text_texture_cache,
    }
}

fn create_window() -> PistonWindow<GlutinWindow> {
    // Create an Glutin window.
    WindowSettings::new("Rust - Snake", [INIT_WIDTH, INIT_HEIGHT])
        .graphics_api(OPEN_GL_VERSION)
        .vsync(true)
        .fullscreen(false)
        .build()
        .expect("Failed to create Window!")
}

fn get_asset_path() -> PathBuf {
    find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .expect("Failed to find assets folder!")
}

fn create_ui() -> Ui {
    //construct Ui
    let mut ui = conrod_core::UiBuilder::new([INIT_WIDTH as f64, INIT_HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = get_asset_path();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts
        .insert_from_file(&font_path)
        .expect(&format!("Failed to find font file: {:?}", font_path));
    ui
}

fn create_app(mut ui: Ui) -> App {
    // Load the rust logo from file to a piston_window texture.
    //let test_texture = load_texture("test.png");

    // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
    // In our case we have no image, however the macro may be used to list multiple.
    let image_map: Map<opengl_graphics::Texture> = conrod_core::image::Map::new();
    //let test_texture = image_map.insert(test_texture);

    // Instantiate the generated list of widget identifiers.
    let generator = ui.widget_id_generator();
    let ids = Ids::new(generator);

    App::new(GUI {
        ui,
        ids,
        image_ids: vec![],
        image_map,
        active_menu: GUIVisibility::MenuOnly(MenuType::Main),
        fullscreen: false,
    })
}

fn create_render_context<'font>() -> RenderContext<'font, opengl_graphics::GlGraphics> {
    let TextCache {
        text_vertex_data,
        glyph_cache,
        text_texture_cache,
    } = create_text_cache(&());
    let gl = GlGraphics::new(OPEN_GL_VERSION);
    RenderContext {
        text_texture_cache,
        glyph_cache,
        text_vertex_data,
        gl,
    }
}
