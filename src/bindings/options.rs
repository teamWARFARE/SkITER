use super::library::Library;
use sciter::RuntimeOptions;
use sciter::GFX_LAYER;

pub struct Options;

impl Options {
    pub fn set_library(library: Library) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::LibraryPath(&library.path))
            .or(Err("Couldn't set library"))
    }

    pub fn set_gfx_layer(gfx_layer: GfxLayer) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::GfxLayer(gfx_layer.to_sciter()))
            .or(Err("Couldn't set gfx layer"))
    }

    pub fn set_ux_theming(value: bool) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::UxTheming(value)).or(Err("Couldn't set gfx layer"))
    }

    pub fn set_script_features(value: u8) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::ScriptFeatures(value))
            .or(Err("Couldn't set script features"))
    }

    pub fn set_debug_mode(value: bool) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::DebugMode(value)).or(Err("Couldn't set gfx layer"))
    }

    pub fn set_init_script(script: &str) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::InitScript(script)).or(Err("Couldn't set gfx layer"))
    }

    pub fn set_logical_pixels(value: bool) -> Result<(), &'static str> {
        sciter::set_options(RuntimeOptions::LogicalPixel(value)).or(Err("Couldn't set gfx layer"))
    }
}

#[derive(Copy, Clone)]
pub enum GfxLayer {
    Auto,
    Cpu,
    SkiaCpu,
    SkiaOpenGl,
}

impl GfxLayer {
    pub(crate) fn to_sciter(&self) -> GFX_LAYER {
        match self {
            GfxLayer::Auto => GFX_LAYER::AUTO,
            GfxLayer::Cpu => GFX_LAYER::CPU,
            GfxLayer::SkiaCpu => GFX_LAYER::SKIA_CPU,
            GfxLayer::SkiaOpenGl => GFX_LAYER::SKIA_OPENGL,
        }
    }
}
