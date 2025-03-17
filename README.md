# DeskVision App

**Search images on your local PC using a local vision model with Ollama.**

<img src='./demo.png' width='550px'/>

## Features

- **Native application** for Windows, macOS, and Linux
- **Blazing fast file search** on disk (Rust-powered)
- **Connect to local or remote Ollama servers** via URL
- **Optimized thumbnail generation** for better performance
  - CPU usage: ~2% (without Ollama), Memory: 100-400MB RAM
- **Automatic labeling and description generation** for images
- **Multi-folder support** for searching, displaying, and labeling images

## TODO

- File format filtering
- Search within a specific directory
- Reverse image search (search by image)
- Reverse image search within a specific directory
- Manual label editing/creation for images
- Drag & Drop folders into the app
- Thumbnail generation in a separate thread
- Improved UI, better image display, etc.
- Open larger images
- Adjustable font size
- Custom system message/prompt settings for labeling
- AI agent for enhanced image search in directories (customized prompts for the vision model)
- Ollama settings UI to pull available vision models
- Image selection, deletion, copying, and moving
- Image cropping for reverse image search
- Support for custom AI agents to automate image searching, deleting, and moving

## Installation

> **Prerequisites:** Ensure you have [Ollama](https://ollama.ai/) installed with a compatible vision model.

```sh
# Clone the repository
git clone https://github.com/yourusername/DeskVision.git
cd DeskVision

# Run the application
cargo run --release
```

## Contribution

Feel free to submit pull requests or open issues to suggest new features or improvements!

## License

MIT License
