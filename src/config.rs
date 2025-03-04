pub const SUPPORTED_IMAGE_FORMATS: [&str; 3] = ["png", "jpg", "jpeg"];

// pub const IMG_LABEL_PROMPT: &str = "List the main objects or elements in this image as very simple labels (use maximum 2.words for a label) separated by commas. Never return more then five labels.";
// pub const IMG_LABEL_PROMPT: &str = "List up to 5 main objects or elements in this image as simple labels, separated by commas. Do not list more than 5.";
pub const IMG_LABEL_PROMPT: &str = "List up to 5 main objects or elements in this image as simple labels, each 2-3 words max, separated by commas. Do not include 'and', '...', or extra text.";

// const COLORS_SIGNAL: [Color; 7] = [
//     Color::Red,
//     Color::LightRed,
//     Color::LightMagenta,
//     Color::Magenta,
//     Color::Yellow,
//     Color::LightGreen,
//     Color::Green,
// ];
//
// const COLORS_NAMES: [Color; 14] = [
//     Color::Rgb(244, 67, 54),
//     Color::Rgb(233, 30, 99),
//     Color::Rgb(156, 39, 176),
//     Color::Rgb(103, 58, 183),
//     Color::Rgb(63, 81, 184),
//     Color::Rgb(3, 169, 244),
//     Color::Rgb(0, 150, 136),
//     Color::Rgb(255, 235, 59),
//     Color::Rgb(255, 152, 0),
//     Color::Rgb(255, 87, 34),
//     Color::Rgb(121, 85, 72),
//     Color::Rgb(158, 158, 158),
//     Color::Rgb(96, 125, 139),
//     Color::White,
// ];
