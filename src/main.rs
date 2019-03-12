use std::collections::HashMap;

use specs::prelude::*;

use quicksilver::{
    geom::{Rectangle, Shape, Vector},
    graphics::{
        Background::{Blended, Col, Img},
        Color, Font, FontStyle, Image,
    },
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    pos: Vector,
    glyph: char,
    color: Color,
}

fn generate_map(size: Vector) -> Vec<Tile> {
    let width = size.x as usize;
    let height = size.y as usize;
    let mut map = Vec::with_capacity(width * height);
    for x in 0..width {
        for y in 0..height {
            let mut tile = Tile {
                pos: Vector::new(x as f32, y as f32),
                glyph: '.',
                color: Color::BLACK,
            };

            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                tile.glyph = '#';
            };
            map.push(tile);
        }
    }
    map
}

#[derive(Debug)]
struct Pos(Vector);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Health {
    max: i32,
    current: i32,
}

impl Health {
    fn new(value: i32) -> Self {
        Self {
            max: value,
            current: value,
        }
    }
}

impl Component for Health {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug)]
struct Render {
    glyph: char,
    color: Color,
}

impl Component for Render {
    type Storage = VecStorage<Self>;
}

fn generate_entities(world: &mut World) {
    world
        .create_entity()
        .with(Pos(Vector::new(9, 6)))
        .with(Health::new(1))
        .with(Render {
            glyph: 'g',
            color: Color::RED,
        })
        .build();

    world
        .create_entity()
        .with(Pos(Vector::new(2, 4)))
        .with(Health::new(1))
        .with(Render {
            glyph: 'g',
            color: Color::RED,
        })
        .build();

    world
        .create_entity()
        .with(Pos(Vector::new(7, 5)))
        .with(Render {
            glyph: '%',
            color: Color::PURPLE,
        })
        .build();

    world
        .create_entity()
        .with(Pos(Vector::new(4, 8)))
        .with(Render {
            glyph: '%',
            color: Color::PURPLE,
        })
        .build();
}

struct GameText {
    font: Font,
    title: Image,
    mononoki_info: Image,
    square_info: Image,
    inventory: Image,
}

struct Game {
    text: Asset<GameText>,
    map_size: Vector,
    map: Vec<Tile>,
    world: World,
    player: Entity,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        // The Mononoki font: https://madmalik.github.io/mononoki/
        // License: SIL Open Font License 1.1
        let font_mononoki = "mononoki-Regular.ttf";

        let font_mononoki = Font::load(font_mononoki);

        let text = Asset::new(font_mononoki.and_then(|font| {
            let title =
                font.render("Quicksilver Roguelike", &FontStyle::new(72.0, Color::BLACK))?;
            let mononoki_info = font.render(
                "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1",
                &FontStyle::new(20.0, Color::BLACK),
            )?;
            let square_info = font.render(
                "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
                &FontStyle::new(20.0, Color::BLACK),
            )?;

            let inventory = font.render(
                "Inventory:\n[A] Sword\n[B] Shield\n[C] Darts",
                &FontStyle::new(20.0, Color::BLACK),
            )?;

            Ok(GameText {
                font,
                title,
                mononoki_info,
                square_info,
                inventory,
            })
        }));

        let map_size = Vector::new(20, 15);
        let map = generate_map(map_size);

        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Health>();
        world.register::<Render>();

        generate_entities(&mut world);
        let player = world
            .create_entity()
            .with(Pos(Vector::new(5, 3)))
            .with(Health { max: 5, current: 3 })
            .with(Render {
                glyph: '@',
                color: Color::BLUE,
            })
            .build();

        // The Square font: http://strlen.com/square/?s[]=font
        // License: CC BY 3.0 https://creativecommons.org/licenses/by/3.0/deed.en_US
        let font_square = "square.ttf";
        let game_glyphs = "#@g.%";
        let tile_size_px = Vector::new(24, 24);
        let tileset = Asset::new(Font::load(font_square).and_then(move |text| {
            let tiles = text
                .render(game_glyphs, &FontStyle::new(tile_size_px.y, Color::WHITE))
                .expect("Could not render the font tileset.");
            let mut tileset = HashMap::new();
            for (index, glyph) in game_glyphs.chars().enumerate() {
                let pos = (index as i32 * tile_size_px.x as i32, 0);
                let tile = tiles.subimage(Rectangle::new(pos, tile_size_px));
                tileset.insert(glyph, tile);
            }
            Ok(tileset)
        }));

        Ok(Self {
            text,
            map_size,
            map,
            world,
            player,
            tileset,
            tile_size_px,
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        use quicksilver::input::ButtonState::*;

        if let Some(mut pos) = self.world.write_storage::<Pos>().get_mut(self.player) {
            if window.keyboard()[Key::Left] == Pressed {
                pos.0.x -= 1.0;
            }
            if window.keyboard()[Key::Right] == Pressed {
                pos.0.x += 1.0;
            }
            if window.keyboard()[Key::Up] == Pressed {
                pos.0.y -= 1.0;
            }
            if window.keyboard()[Key::Down] == Pressed {
                pos.0.y += 1.0;
            }
        }

        if window.keyboard()[Key::Escape].is_down() {
            window.close();
        }

        if window.keyboard()[Key::X].is_down() {
            self.text.execute(|text| {
                let inventory = text.font.render(
                    "Inventory:\n[A] Dagger\n[B] Buckler",
                    &FontStyle::new(20.0, Color::BLACK),
                );
                text.inventory = inventory?;
                Ok(())
            })?;
        }

        self.world.maintain();

        Ok(())
    }

    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        // Draw the game title
        self.text.execute(|text| {
            window.draw(
                &text
                    .title
                    .area()
                    .with_center((window.screen_size().x as i32 / 2, 40)),
                Img(&text.title),
            );
            Ok(())
        })?;

        // Draw the mononoki font credits
        self.text.execute(|text| {
            window.draw(
                &text
                    .mononoki_info
                    .area()
                    .translate((2, window.screen_size().y as i32 - 60)),
                Img(&text.mononoki_info),
            );
            Ok(())
        })?;

        // Draw the Square font credits
        self.text.execute(|text| {
            window.draw(
                &text
                    .square_info
                    .area()
                    .translate((2, window.screen_size().y as i32 - 30)),
                Img(&text.square_info),
            );
            Ok(())
        })?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new(50, 120);

        // Draw the map
        let (tileset, map) = (&mut self.tileset, &self.map);
        tileset.execute(|tileset| {
            for tile in map.iter() {
                if let Some(image) = tileset.get(&tile.glyph) {
                    let pos_px = tile.pos.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(offset_px + pos_px, image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        let pos_storage = self.world.read_storage::<Pos>();
        let render_storage = self.world.read_storage::<Render>();
        self.tileset.execute(|tileset| {
            for (pos, render) in (&pos_storage, &render_storage).join() {
                if let Some(image) = tileset.get(&render.glyph) {
                    let pos_px = offset_px + pos.0.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, render.color),
                    );
                }
            }

            Ok(())
        })?;

        let map_size_px = self.map_size.times(tile_size_px);
        let health_bar_pos_px = offset_px + Vector::new(map_size_px.x, 0.0);

        let health_storage = self.world.read_storage::<Health>();
        if let Some(player_health) = health_storage.get(self.player) {
            let full_health_width_px = 100.0;
            let current_health_width_px =
                (player_health.current as f32 / player_health.max as f32) * full_health_width_px;

            // Full health
            window.draw(
                &Rectangle::new(health_bar_pos_px, (full_health_width_px, tile_size_px.y)),
                Col(Color::RED.with_alpha(0.5)),
            );

            // Current health
            window.draw(
                &Rectangle::new(health_bar_pos_px, (current_health_width_px, tile_size_px.y)),
                Col(Color::RED),
            );
        }

        self.text.execute(|text| {
            window.draw(
                &text
                    .inventory
                    .area()
                    .translate(health_bar_pos_px + Vector::new(0, tile_size_px.y)),
                Img(&text.inventory),
            );
            Ok(())
        })?;

        Ok(())
    }
}

fn main() {
    // NOTE: Set HIDPI to 1.0 to get pixel-perfect rendering.
    // Otherwise the window resizes to whatever value the OS sets and
    // scales the contents.
    // https://docs.rs/glutin/0.19.0/glutin/dpi/index.html
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let settings = Settings {
        // If the graphics do need to be scaled (e.g. using
        // `with_center`), blur them. This looks better with fonts.
        scale: quicksilver::graphics::ImageScaleStrategy::Blur,
        ..Default::default()
    };
    run::<Game>("Quicksilver Roguelike", Vector::new(800, 600), settings);
}
