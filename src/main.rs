use quicksilver::{
    combinators::result,
    geom::{Rectangle, Shape, Vector},
    graphics::{
        Background::{Blended, Img},
        Color, Font, FontStyle, Image,
    },
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

use std::collections::HashMap;

struct Game {
    title: Asset<Image>,
    mononoki_font_info: Asset<Image>,
    square_font_info: Asset<Image>,
    tilemap: Asset<HashMap<char, Image>>,
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
        let (width, height) = (24, 24);
        let tilemap = Asset::new(Font::load(font_square).and_then(move |text| {
            let tiles = text
                .render(tilemap_source, &FontStyle::new(height as f32, Color::WHITE))
                .expect("Could not render the font tilemap.");
            let mut tilemap = HashMap::new();
            for (index, glyph) in tilemap_source.chars().enumerate() {
                let pos = (index as i32 * width, 0);
                let size = (width, height);
                let tile = tiles.subimage(Rectangle::new(pos, size));
                tilemap.insert(glyph, tile);
            }
            result(Ok(tilemap))
        }));

        Ok(Self {
            title,
            mononoki_font_info,
            square_font_info,
            tilemap,
        })
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

        self.tilemap.execute(|tilemap| {
            let tiles: &[(char, i32, i32, Color)] = &[
                ('#', 0, 0, Color::BLACK),
                ('#', 0, 1, Color::BLACK),
                ('#', 0, 2, Color::BLACK),
                ('#', 0, 3, Color::BLACK),
                ('g', 1, 1, Color::RED),
                ('.', 1, 2, Color::BLACK),
                ('.', 2, 3, Color::BLACK),
                ('.', 2, 1, Color::BLACK),
                ('.', 3, 2, Color::BLACK),
                ('g', 3, 1, Color::RED),
                ('@', 2, 2, Color::BLUE),
                ('g', 1, 3, Color::RED),
                ('g', 3, 3, Color::RED),
                ('#', 1, 0, Color::BLACK),
                ('#', 2, 0, Color::BLACK),
                ('#', 3, 0, Color::BLACK),
            ];
            let offset = Vector::new(50, 150);
            for (glyph, x, y, color) in tiles {
                if let Some(tile) = tilemap.get(glyph) {
                    let pos = (x * 24, y * 24);
                    window.draw(
                        &Rectangle::new(offset.translate(pos), tile.area().size()),
                        Blended(&tile, *color),
                    );
                }
            }
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
