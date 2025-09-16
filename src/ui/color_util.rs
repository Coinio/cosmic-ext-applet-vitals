use cosmic::cosmic_theme::palette::Srgba;

pub struct ColorUtil;

impl ColorUtil {

    pub fn convert_srgba_to_hex_string(colour: Srgba) -> String {
        fn to_u8(x: f32) -> u8 {
            let v = (x * 255.0).round();
            v.clamp(0.0, 255.0) as u8
        }

        let red_u8 = to_u8(colour.red);
        let green_u8 = to_u8(colour.green);
        let blue_u8 = to_u8(colour.blue);
        let alpha_u8 = to_u8(colour.alpha);

        format!("#{:02x}{:02x}{:02x}{:02x}", red_u8, green_u8, blue_u8, alpha_u8)
    }

}
