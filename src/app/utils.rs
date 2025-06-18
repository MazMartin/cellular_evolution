use winit::window::Icon;
use image::GenericImageView;

pub fn load_icon(path: &str) -> Icon {
    let image = image::open(path).expect("Failed to open icon");
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8().into_raw();
    println!(
        "Loaded icon: {}x{} ({} pixels)",
        width,
        height,
        rgba.len() / 4
    );
    
    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}