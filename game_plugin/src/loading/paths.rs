pub struct AssetPaths {
    pub fira_sans: &'static str,
    pub audio_flying: &'static str,
    pub texture_bevy: &'static str,
    pub texture_wall: &'static str,
    pub texture_path: &'static str,
}

pub const PATHS: AssetPaths = AssetPaths {
    fira_sans: "fonts/FiraSans-Bold.ttf",
    audio_flying: "audio/flying.ogg",
    texture_bevy: "textures/bevy.png",
    texture_wall: "textures/wall.png",
    texture_path: "textures/path.png",
};
