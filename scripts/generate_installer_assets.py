import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

def create_header_bmp(logo_path, out_path):
    width, height = 150, 57
    img = Image.new("RGB", (width, height), (11, 12, 16))
    draw = ImageDraw.Draw(img)
    
    # Subtle gradient background from left to right: (11, 12, 16) to (18, 19, 26)
    for x in range(width):
        ratio = x / width
        r = int(11 + (18 - 11) * ratio)
        g = int(12 + (19 - 12) * ratio)
        b = int(16 + (26 - 16) * ratio)
        draw.line([(x, 0), (x, height)], fill=(r, g, b))
        
    # Open new logo
    logo = Image.open(logo_path).convert("RGBA")
    logo_size = 46
    logo_resized = logo.resize((logo_size, logo_size), Image.Resampling.LANCZOS)
    
    # Place on right side: y centered (4), x at 150 - 46 - 8 = 96
    x_pos = width - logo_size - 8
    y_pos = (height - logo_size) // 2
    
    # Soft glow behind logo
    glow = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    center_x = x_pos + logo_size // 2
    center_y = y_pos + logo_size // 2
    glow_draw.ellipse([center_x - 28, center_y - 28, center_x + 28, center_y + 28], fill=(220, 150, 40, 45))
    glow = glow.filter(ImageFilter.GaussianBlur(10))
    
    # Composite glow and logo
    img_rgba = img.convert("RGBA")
    img_rgba = Image.alpha_composite(img_rgba, glow)
    img_rgba.paste(logo_resized, (x_pos, y_pos), logo_resized)
    
    final = img_rgba.convert("RGB")
    final.save(out_path, format="BMP")
    print(f"Saved header BMP: {out_path} ({width}x{height})")

def create_wizard_bmp(logo_path, out_path):
    width, height = 164, 314
    img = Image.new("RGB", (width, height), (11, 12, 16))
    draw = ImageDraw.Draw(img)
    
    # Vertical gradient: subtle dark tones
    for y in range(height):
        # subtle radial/vertical glow around y=120
        dist = abs(y - 120) / 160.0
        factor = max(0.0, 1.0 - dist)
        r = int(11 + factor * 22)
        g = int(12 + factor * 14)
        b = int(16 + factor * 18)
        draw.line([(0, y), (width, y)], fill=(r, g, b))
        
    # Subtle decorative geometric lines / border on right edge
    draw.line([(width - 1, 0), (width - 1, height)], fill=(32, 35, 48))
    
    # Open new logo
    logo = Image.open(logo_path).convert("RGBA")
    logo_size = 110
    logo_resized = logo.resize((logo_size, logo_size), Image.Resampling.LANCZOS)
    
    x_pos = (width - logo_size) // 2
    y_pos = 65
    
    # Ambient warm glow behind logo
    glow = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    cx = width // 2
    cy = y_pos + logo_size // 2
    glow_draw.ellipse([cx - 65, cy - 65, cx + 65, cy + 65], fill=(230, 140, 30, 40))
    glow_draw.ellipse([cx - 45, cy - 45, cx + 45, cy + 45], fill=(255, 60, 40, 30))
    glow = glow.filter(ImageFilter.GaussianBlur(16))
    
    img_rgba = img.convert("RGBA")
    img_rgba = Image.alpha_composite(img_rgba, glow)
    img_rgba.paste(logo_resized, (x_pos, y_pos), logo_resized)
    
    # Add typography below the logo
    txt_draw = ImageDraw.Draw(img_rgba)
    
    # Draw branded text
    # Since fonts might vary, draw clean stylized banner or text
    try:
        font_large = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 12)
        font_sub = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 9)
    except Exception:
        font_large = ImageFont.load_default()
        font_sub = ImageFont.load_default()
        
    title = "DIGITAL VIGITAL"
    subtitle = "MINECRAFT LAUNCHER"
    version_tag = "v1.0.0 RELEASE"
    
    # Center text
    bbox1 = txt_draw.textbbox((0, 0), title, font=font_large)
    t1_w = bbox1[2] - bbox1[0]
    txt_draw.text(((width - t1_w) // 2, 195), title, fill=(245, 245, 250), font=font_large)
    
    bbox2 = txt_draw.textbbox((0, 0), subtitle, font=font_sub)
    t2_w = bbox2[2] - bbox2[0]
    txt_draw.text(((width - t2_w) // 2, 215), subtitle, fill=(160, 165, 185), font=font_sub)
    
    # Subtle badge for version
    badge_w, badge_h = 96, 18
    bx = (width - badge_w) // 2
    by = 240
    txt_draw.rounded_rectangle([bx, by, bx + badge_w, by + badge_h], radius=4, fill=(24, 26, 36), outline=(50, 55, 75))
    
    bbox3 = txt_draw.textbbox((0, 0), version_tag, font=font_sub)
    t3_w = bbox3[2] - bbox3[0]
    txt_draw.text(((width - t3_w) // 2, by + 3), version_tag, fill=(235, 175, 65), font=font_sub)
    
    final = img_rgba.convert("RGB")
    final.save(out_path, format="BMP")
    print(f"Saved wizard BMP: {out_path} ({width}x{height})")

if __name__ == "__main__":
    create_header_bmp("assets/logo.png", "packaging/windows/header.bmp")
    create_wizard_bmp("assets/logo.png", "packaging/windows/wizard.bmp")
