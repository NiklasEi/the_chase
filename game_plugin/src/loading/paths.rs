pub struct AssetPaths {
    pub fira_sans: &'static str,
    pub audio_fall: &'static str,
    pub audio_button_click: &'static str,
    pub audio_wall_moving: &'static str,
    pub audio_happy_background: &'static str,
    pub texture_player: &'static str,
    pub texture_button: &'static str,
    pub texture_button_active: &'static str,
}

pub const PATHS: AssetPaths = AssetPaths {
    fira_sans: "fonts/FiraSans-Bold.ttf",
    audio_fall: "audio/fall.ogg",
    audio_button_click: "audio/button_click.ogg",
    audio_wall_moving: "audio/wall_moving.ogg",
    audio_happy_background: "audio/happy_background.ogg",
    texture_player: "textures/player.png",
    texture_button: "textures/button.png",
    texture_button_active: "textures/button_active.png",
};
