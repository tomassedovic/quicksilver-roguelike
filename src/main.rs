use quicksilver::{
    combinators::result,
    geom::{Rectangle, Shape, Vector},
    graphics::{
        Background::{Blended, Col, Img},
        Color, Font, FontStyle, Image,
    },
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    x: i32,
    y: i32,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tile {
    x: i32,
    y: i32,
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
                x: x as i32,
                y: y as i32,
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

fn generate_entities() -> Vec<Entity> {
    vec![
        Entity {
            x: 9,
            y: 6,
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
        },
        Entity {
            x: 2,
            y: 4,
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
        },
    ]
}

struct Game {
    title: Asset<Image>,
    mononoki_font_info: Asset<Image>,
    square_font_info: Asset<Image>,
    tilemap: Asset<HashMap<char, Image>>,
    map_size: Vector,
    tile_size: Vector,
    map: Vec<Tile>,
    entities: Vec<Entity>,
    player_id: usize,
}

impl State for Game {
    fn new() -> Result<Self> {
        // The Mononoki font: https://madmalik.github.io/mononoki/
        // License: SIL Open Font License 1.1
        let font_mononoki = "mononoki-Regular.ttf";
        // The Square font: http://strlen.com/square/?s[]=font
        // License: CC BY 3.0 https://creativecommons.org/licenses/by/3.0/deed.en_US
        let font_square = "square.ttf";

        let title = Asset::new(Font::load(font_mononoki).and_then(|font| {
            font.render("Quicksilver Roguelike", &FontStyle::new(72.0, Color::BLACK))
        }));

        let text_style = FontStyle::new(20.0, Color::BLACK);
        let mononoki_font_info = Asset::new(Font::load(font_mononoki).and_then(move |font| {
            font.render(
                "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1",
                &text_style.clone(),
            )
        }));
        let square_font_info = Asset::new(Font::load(font_mononoki).and_then(move |font| {
            font.render(
                "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
                &text_style,
            )
        }));

        let tilemap_source = "#@g.";
        let map_size = Vector::new(20, 15);
        let tile_size = Vector::new(24, 24);
        let tilemap = Asset::new(Font::load(font_square).and_then(move |text| {
            let tiles = text
                .render(tilemap_source, &FontStyle::new(tile_size.y, Color::WHITE))
                .expect("Could not render the font tilemap.");
            let mut tilemap = HashMap::new();
            for (index, glyph) in tilemap_source.chars().enumerate() {
                let pos = (index as i32 * tile_size.x as i32, 0);
                let tile = tiles.subimage(Rectangle::new(pos, tile_size));
                tilemap.insert(glyph, tile);
            }
            result(Ok(tilemap))
        }));

        let map = generate_map(map_size);
        let mut entities = generate_entities();
        let player_id = entities.len();
        entities.push(Entity {
            x: 5,
            y: 3,
            glyph: '@',
            color: Color::BLUE,
            hp: 3,
            max_hp: 5,
        });

        Ok(Self {
            title,
            mononoki_font_info,
            square_font_info,
            tilemap,
            map_size,
            tile_size,
            map,
            entities,
            player_id,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        use quicksilver::input::ButtonState::*;

        let player = &mut self.entities[self.player_id];
        if window.keyboard()[Key::Left] == Pressed {
            player.x -= 1;
        }
        if window.keyboard()[Key::Right] == Pressed {
            player.x += 1;
        }
        if window.keyboard()[Key::Up] == Pressed {
            player.y -= 1;
        }
        if window.keyboard()[Key::Down] == Pressed {
            player.y += 1;
        }
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        self.title.execute(|image| {
            window.draw(
                &image
                    .area()
                    .with_center((window.screen_size().x as i32 / 2, 40)),
                Img(&image),
            );
            Ok(())
        })?;

        self.mononoki_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 60)),
                Img(&image),
            );
            Ok(())
        })?;

        self.square_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 30)),
                Img(&image),
            );
            Ok(())
        })?;

        let offset = Vector::new(50, 150);
        let map_size = self.map_size;
        let tile_size = self.tile_size;

        // NOTE: Need to do partial borrows here to prevent borrowing
        // the whole self as mutable.
        let (tilemap, map) = (&mut self.tilemap, &self.map);
        tilemap.execute(|tilemap| {
            for tile in map.iter() {
                if let Some(image) = tilemap.get(&tile.glyph) {
                    let pos = Vector::new(tile.x as f32 * tile_size.x, tile.y as f32 * tile_size.y);
                    window.draw(
                        &Rectangle::new(offset.translate(pos), image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        let (tilemap, entities) = (&mut self.tilemap, &self.entities);
        tilemap.execute(|tilemap| {
            for entity in entities.iter() {
                if let Some(image) = tilemap.get(&entity.glyph) {
                    let pos = (entity.x as f32 * tile_size.x, entity.y as f32 * tile_size.y);
                    window.draw(
                        &Rectangle::new(offset.translate(pos), image.area().size()),
                        Blended(&image, entity.color),
                    );
                }
            }
            Ok(())
        })?;

        // Draw the health bar
        let player = &self.entities[self.player_id];
        let full_health_width = 100.0;
        let current_health_width = (player.hp as f32 / player.max_hp as f32) * full_health_width;
        window.draw(
            &Rectangle::new((tile_size.x as f32, 0.0), (full_health_width, tile_size.y))
                .translate(offset)
                .translate(((map_size.x * tile_size.x) as f32, 0.0)),
            Col(Color::RED.with_alpha(0.5)),
        );
        window.draw(
            &Rectangle::new(
                (tile_size.x as f32, 0.0),
                (current_health_width, tile_size.y),
            )
            .translate(offset)
            .translate(((map_size.x * tile_size.x) as f32, 0.0)),
            Col(Color::RED),
        );

        Ok(())
    }
}

fn main() {
    // NOTE: Set HIDPI to 1.0 to get pixel-perfect rendering.
    // Otherwise the window resizes to whatever value the OS sets and
    // scales the contents.
    // https://docs.rs/glutin/0.19.0/glutin/dpi/index.html
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");

    let settings = Settings {
        // Don't scale the graphics when the window is resized
        resize: quicksilver::graphics::ResizeStrategy::Maintain,

        // If the graphics do need to be scaled (e.g. with
        // `with_center`), blur them. This looks better with fonts.
        scale: quicksilver::graphics::ImageScaleStrategy::Blur,
        ..Default::default()
    };
    run::<Game>("Quicksilver Roguelike", Vector::new(800, 600), settings);
}
