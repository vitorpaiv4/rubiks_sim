from PIL import Image, ImageDraw
import math
import os

def create_rubiks_icon(size=512):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Isometric projection parameters
    cx, cy = size / 2, size / 2 + (size * 0.04)
    scale = size * 0.28

    # Isometric basis vectors
    ux, uy = math.cos(math.radians(30)) * scale, -math.sin(math.radians(30)) * scale
    vx, vy = -math.cos(math.radians(30)) * scale, -math.sin(math.radians(30)) * scale
    wx, wy = 0, scale

    def to_screen(x, y, z):
        sx = cx + (x - 1.5) * ux + (y - 1.5) * vx
        sy = cy + (x - 1.5) * uy + (y - 1.5) * vy - (z - 1.5) * wy
        return (sx, sy)

    margin = 0.08

    # TOP face (z = 3) - White
    top_colors = [
        ["#FFFFFF", "#FFFFFF", "#FFFFFF"],
        ["#FFFFFF", "#EDEDED", "#FFFFFF"],
        ["#FFFFFF", "#FFFFFF", "#FFFFFF"]
    ]

    # FRONT face (y = 0) - Green
    front_colors = [
        ["#00D040", "#00B835", "#00D040"],
        ["#00D040", "#00D040", "#00A830"],
        ["#00B835", "#00D040", "#00D040"]
    ]

    # RIGHT face (x = 3) - Red
    right_colors = [
        ["#E82020", "#D01515", "#E82020"],
        ["#E82020", "#E82020", "#D01515"],
        ["#D01515", "#E82020", "#E82020"]
    ]

    # Draw Top Face Stickers (z = 3)
    for ix in range(3):
        for iy in range(3):
            p1 = to_screen(ix + margin, iy + margin, 3)
            p2 = to_screen(ix + 1 - margin, iy + margin, 3)
            p3 = to_screen(ix + 1 - margin, iy + 1 - margin, 3)
            p4 = to_screen(ix + margin, iy + 1 - margin, 3)
            b1 = to_screen(ix, iy, 3)
            b2 = to_screen(ix + 1, iy, 3)
            b3 = to_screen(ix + 1, iy + 1, 3)
            b4 = to_screen(ix, iy + 1, 3)
            draw.polygon([b1, b2, b3, b4], fill="#111111")
            draw.polygon([p1, p2, p3, p4], fill=top_colors[iy][ix])

    # Draw Front Face Stickers (y = 0)
    for ix in range(3):
        for iz in range(3):
            p1 = to_screen(ix + margin, 0, iz + margin)
            p2 = to_screen(ix + 1 - margin, 0, iz + margin)
            p3 = to_screen(ix + 1 - margin, 0, iz + 1 - margin)
            p4 = to_screen(ix + margin, 0, iz + 1 - margin)
            b1 = to_screen(ix, 0, iz)
            b2 = to_screen(ix + 1, 0, iz)
            b3 = to_screen(ix + 1, 0, iz + 1)
            b4 = to_screen(ix, 0, iz + 1)
            draw.polygon([b1, b2, b3, b4], fill="#111111")
            draw.polygon([p1, p2, p3, p4], fill=front_colors[iz][ix])

    # Draw Right Face Stickers (x = 3)
    for iy in range(3):
        for iz in range(3):
            p1 = to_screen(3, iy + margin, iz + margin)
            p2 = to_screen(3, iy + 1 - margin, iz + margin)
            p3 = to_screen(3, iy + 1 - margin, iz + 1 - margin)
            p4 = to_screen(3, iy + margin, iz + 1 - margin)
            b1 = to_screen(3, iy, iz)
            b2 = to_screen(3, iy + 1, iz)
            b3 = to_screen(3, iy + 1, iz + 1)
            b4 = to_screen(3, iy, iz + 1)
            draw.polygon([b1, b2, b3, b4], fill="#111111")
            draw.polygon([p1, p2, p3, p4], fill=right_colors[iz][iy])

    os.makedirs("assets", exist_ok=True)
    
    # 1. Save standard PNGs
    img.save("assets/icon.png", "PNG")
    img.save("icon.png", "PNG")

    # 2. Save multi-resolution Windows ICO
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    img.save("icon.ico", format="ICO", sizes=ico_sizes)
    img.save("assets/icon.ico", format="ICO", sizes=ico_sizes)

    # 3. Save Android mipmaps
    android_mipmaps = {
        "res/mipmap-mdpi": 48,
        "res/mipmap-hdpi": 72,
        "res/mipmap-xhdpi": 96,
        "res/mipmap-xxhdpi": 144,
        "res/mipmap-xxxhdpi": 192,
    }
    for folder, s in android_mipmaps.items():
        os.makedirs(folder, exist_ok=True)
        resized = img.resize((s, s), Image.Resampling.LANCZOS)
        resized.save(os.path.join(folder, "ic_launcher.png"), "PNG")

    print("All icons (PNG, ICO, Android mipmaps) generated successfully.")

if __name__ == "__main__":
    create_rubiks_icon(512)
