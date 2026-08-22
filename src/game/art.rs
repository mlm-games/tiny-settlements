//! Card art baked at startup from `.ren` (Renamite) files, following the
//! floppy-warriors approach. Baking from `.ren` keeps the door open for
//! animated cards later — just scrub to another frame.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use renamite_io_ren::RenFile;
use renamite_render_bridge::SceneRenderer;
use renamite_render_offscreen::{OffscreenRenderer, fit_view};

use super::CARD_BG_PATH;
use super::card_defs::CardType;

/// Icon bake size (square). The frame bakes at 192x288.
const ICON_PX: u32 = 256;
const BG_W: u32 = 192;
const BG_H: u32 = 288;

/// Handles of card textures rasterized from `assets/images/cards/*.ren`.
/// Empty when baking was skipped/failed — sprites fall back to flat colors.
#[derive(Resource, Default)]
pub struct CardArt {
    pub bg: Option<Handle<Image>>,
    pub icons: HashMap<CardType, Handle<Image>>,
}

pub fn load_card_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let art = match bake_card_art(&mut images) {
        Ok(art) => art,
        Err(e) => {
            bevy::log::warn!("card art baking unavailable, using colored fallback: {e:#}");
            CardArt::default()
        }
    };
    commands.insert_resource(art);
}
fn load_ren(path: &str) -> anyhow::Result<RenFile> {
    // asset_path() is AssetServer-relative ("images/..."); raw fs reads need
    // the assets/ prefix
    let full = format!("assets/{path}");
    let text = std::fs::read_to_string(&full).map_err(|e| anyhow::anyhow!("read {full}: {e}"))?;
    Ok(renamite_io_ren::open(&text)?)
}

fn image_rgba8(w: u32, h: u32, data: Vec<u8>) -> Image {
    let mut img = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::Default;
    img
}

struct Baker {
    gpu: OffscreenRenderer,
    bridge: SceneRenderer,
    canvas_w: u32,
    canvas_h: u32,
}

impl Baker {
    fn new(w: u32, h: u32) -> anyhow::Result<Self> {
        Ok(Self {
            gpu: OffscreenRenderer::new_blocking(w, h, 1)?,
            bridge: SceneRenderer::new(),
            canvas_w: w,
            canvas_h: h,
        })
    }

    /// Rasterize one `.ren` artboard into an exactly-sized RGBA image.
    /// `border: (color, width, corner_radius)` strokes the artboard edge with
    /// repose's own `SceneNode::Border`, baking it into the texture.
    fn bake(
        &mut self,
        file: RenFile,
        w: u32,
        h: u32,
        border: Option<(repose_core::Color, f32, f32)>,
    ) -> anyhow::Result<Image> {
        let mut player = renamite_player::Player::new(file)?;
        player.engine.scrub(&player.project, 0.0);
        self.gpu.sync_document_images(&player.project.document)?;

        let artboard = player.project.document.compositions[player.project.document.main].size;
        let view = fit_view(artboard, w, h);
        let prepared = self.bridge.prepare(player.engine.scene(), &view);
        let mut scene = repose_core::Scene::default();
        self.bridge.append_repose_scene(&prepared, &mut scene);

        if let Some((color, bw, radius)) = border {
            scene.nodes.push(repose_core::SceneNode::Border {
                rect: repose_core::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: artboard.0 as f32,
                    h: artboard.1 as f32,
                },
                color,
                width: bw,
                radius: [radius; 4],
            });
        }

        // Transparent clear; un-premultiply so semi-transparent fills survive.
        let rgba = self.gpu.render_rgba(&scene, Some([0.0, 0.0, 0.0, 0.0]))?;
        let ox = view.offset.x.round().max(0.0) as u32;
        let oy = view.offset.y.round().max(0.0) as u32;
        let pw = ((w as f64) * view.scale).round() as u32;
        let ph = ((h as f64) * view.scale).round() as u32;
        let pw = pw.min(self.canvas_w.saturating_sub(ox)).min(w);
        let ph = ph.min(self.canvas_h.saturating_sub(oy)).min(h);

        let stride = self.canvas_w as usize;
        let mut data = Vec::with_capacity((pw * ph * 4) as usize);
        for y in 0..ph as usize {
            for x in 0..pw as usize {
                let i = ((oy as usize + y) * stride + ox as usize + x) * 4;
                let a = rgba[i + 3] as f32 / 255.0;
                if a > 0.0 {
                    let cv = |v: u8| ((v as f32 / 255.0 / a).min(1.0) * 255.0) as u8;
                    data.extend([cv(rgba[i]), cv(rgba[i + 1]), cv(rgba[i + 2]), rgba[i + 3]]);
                } else {
                    data.extend([0, 0, 0, 0]);
                }
            }
        }
        Ok(image_rgba8(pw, ph, data))
    }
}

const ICON_CARDS: [CardType; 23] = [
    CardType::Gardener,
    CardType::BioSubstrate,
    CardType::SporePod,
    CardType::NutrientSlime,
    CardType::BasicFungi,
    CardType::ProcessedNutrients,
    CardType::VineSeed,
    CardType::YoungVine,
    CardType::MatureVine,
    CardType::FlutterwingSpore,
    CardType::FlutterwingLarva,
    CardType::MatureFlutterwing,
    CardType::FertilizedVinePod,
    CardType::SymbioticAlgae,
    CardType::LuminaCrystal,
    CardType::GrazingSlugEgg,
    CardType::GrazingSlug,
    CardType::RichMulch,
    CardType::FertileSubstrate,
    CardType::WasteToxin,
    CardType::ApexSpore,
    CardType::GrowingApex,
    CardType::GenesisBloom,
];

fn bake_card_art(images: &mut Assets<Image>) -> anyhow::Result<CardArt> {
    let mut icons = Baker::new(ICON_PX, ICON_PX)?;
    let mut bg = Baker::new(BG_W, BG_H)?;

    let mut art = CardArt::default();

    let bg_file = load_ren(CARD_BG_PATH)?;
    // stroke matches the menus' panel border family (col(70,110,80)); width is
    // in artboard units (~0.92 downscale to the texture, ~2px on screen)
    let frame_border = Some((repose_core::Color::from_rgba(70, 110, 80, 255), 5.0, 12.0));
    art.bg = Some(images.add(bg.bake(bg_file, BG_W, BG_H, frame_border)?));

    for card in ICON_CARDS {
        let Some(path) = card.asset_path() else {
            continue;
        };
        let Ok(file) = load_ren(path) else {
            bevy::log::warn!("skipping card icon {path}");
            continue;
        };
        match icons.bake(file, ICON_PX, ICON_PX, None) {
            Ok(img) => {
                art.icons.insert(card, images.add(img));
            }
            Err(e) => bevy::log::warn!("skipping card icon {path}: {e:#}"),
        }
    }

    Ok(art)
}
