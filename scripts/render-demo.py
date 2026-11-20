"""Create a bilingual landscape GIF from verified Windows captures."""

import argparse
import csv
import os
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFont, ImageStat


CAPTION_HEIGHT = 76
SCENES = [
    ("01-empty.png", 700, "输入手机号", "Enter a mobile number"),
    ("02-single-result.png", 1400, "查看单号结果", "See the result instantly"),
    ("03-prefix.png", 380, "支持中国区号", "Use the +86 country code"),
    ("04-area.png", 380, "号码可以分组", "Spaces are accepted"),
    ("05-group.png", 420, "继续输入号码", "Keep entering the number"),
    ("06-formatted-result.png", 2100, "直接显示归属地", "See carrier, province and city"),
    ("07-before-import.png", 1000, "批量导入", "Import Excel or CSV"),
    ("08-open-dialog.png", 1150, "选择号码文件", "Choose the input file"),
    ("09-ready-to-export.png", 1500, "处理完成，可导出结果", "Ready to export the results"),
    ("10-save-dialog.png", 1150, "选择保存位置", "Choose where to save the results"),
    ("11-batch-complete.png", 1600, "导出完成，可再次导出", "Export complete; export again if needed"),
    ("result.csv", 3100, "四列查询结果", "Open the exported results"),
]


def font(size, bold=False):
    if os.name == "nt":
        directory = Path(os.environ["WINDIR"]) / "Fonts"
        names = ("msyhbd.ttc", "msyh.ttc", "simhei.ttf", "simsun.ttc") if bold else (
            "msyh.ttc", "msyhbd.ttc", "simsun.ttc"
        )
    else:
        directory = Path("/System/Library/Fonts")
        names = ("Hiragino Sans GB.ttc",)
    for name in names:
        path = directory / name
        if path.exists():
            return ImageFont.truetype(str(path), size)
    raise FileNotFoundError("A Chinese font is needed to render the bilingual demo")


def theme_colours(theme):
    if theme == "dark":
        return {
            "canvas": "#1e1e1e", "ink": "#f3f3f3", "muted": "#c4c4c4",
            "line": "#555555", "table_head": "#303030",
        }
    return {
        "canvas": "#fafafa", "ink": "#202020", "muted": "#555555",
        "line": "#d3d3d3", "table_head": "#f0f0f0",
    }


def check_captures(frames):
    for name, _, _, _ in SCENES:
        if name.endswith(".csv"):
            continue
        with Image.open(frames / name) as opened:
            shot = opened.convert("RGB")
        if min(shot.size) < 350 or max(ImageStat.Stat(shot).stddev) < 8:
            raise ValueError(f"Unusable Windows capture: {frames / name}")
        if name not in {"08-open-dialog.png", "10-save-dialog.png"}:
            contaminated_rows = 0
            for y in range(shot.height - 3, shot.height):
                blue_pixels = sum(
                    blue > red + 40 and blue > green + 15
                    for red, green, blue in (shot.getpixel((x, y)) for x in range(shot.width))
                )
                contaminated_rows += blue_pixels > shot.width * 0.75
            if contaminated_rows >= 2:
                raise ValueError(f"Desktop is visible below the application: {frames / name}")


def check_theme_difference(light, dark):
    with Image.open(light / "01-empty.png") as opened:
        light_shot = opened.convert("L")
    with Image.open(dark / "01-empty.png") as opened:
        dark_shot = opened.convert("L")
    if light_shot.size != dark_shot.size:
        raise ValueError("Light and dark captures have different window dimensions")
    width, height = light_shot.size
    box = (width // 10, height // 6, width * 9 // 10, height * 9 // 10)
    light_mean = ImageStat.Stat(light_shot.crop(box)).mean[0]
    dark_mean = ImageStat.Stat(dark_shot.crop(box)).mean[0]
    if light_mean - dark_mean < 45:
        raise ValueError(
            f"Application themes look too similar ({light_mean:.1f} versus {dark_mean:.1f})"
        )


def read_result(path):
    with path.open(encoding="utf-8-sig", newline="") as source:
        rows = list(csv.reader(source))
    if not rows or rows[0] != ["手机号", "运营商", "归属地省份", "归属地地级市"]:
        raise ValueError(f"Unexpected result columns in {path}")
    if len(rows) != 6 or any(len(row) != 4 for row in rows):
        raise ValueError(f"Expected five complete demo result rows in {path}")
    return rows


def convert_native_hero(frames):
    bitmap = frames / "window-2x.bmp"
    if not bitmap.exists():
        return
    with Image.open(bitmap) as opened:
        hero = opened.convert("RGB")
        deviation = max(ImageStat.Stat(hero).stddev)
        if min(hero.size) < 1000 or deviation < 8:
            colours = hero.getcolors(maxcolors=65536)
            colour_count = len(colours) if colours is not None else ">65536"
            corner = Image.new("RGB", hero.size, hero.getpixel((0, 0)))
            content_bbox = ImageChops.difference(hero, corner).getbbox()
            raise ValueError(
                f"Unusable native 192 DPI capture: {bitmap}; "
                f"size={hero.size}, colours={colour_count}, "
                f"non-corner bbox={content_bbox}, channel deviation={deviation:.2f}"
            )
        hero.save(frames / "hero.png", format="PNG")


def draw_table(draw, rows, colours, width, height):
    margin = max(16, width // 40)
    draw.text((margin, 44), "result.csv", font=font(27, True),
              fill=colours["ink"], anchor="lm")
    draw.rectangle((0, 80, width, 132), fill=colours["table_head"])
    columns = (margin, width * 25 // 100, width * 49 // 100, width * 73 // 100)
    table_font = 23 if width >= 840 else 18
    for x, value in zip(columns, rows[0]):
        draw.text((x, 106), value, font=font(table_font, True),
                  fill=colours["ink"], anchor="lm")
    row_step = min(73, max(55, (height - 183) // 6))
    for index, row in enumerate(rows[1:]):
        y = 165 + index * row_step
        for x, value in zip(columns, row):
            draw.text((x, y), value, font=font(table_font), fill=colours["ink"], anchor="lm")
        draw.line((margin, y + 28, width - margin, y + 28), fill=colours["line"], width=1)
    draw.text((margin, height - 27), "格式错误的号码保留原值，查询字段留空",
              font=font(15), fill=colours["muted"], anchor="lm")


def render_scene(frames, scene, theme, window_size):
    name, duration, title, english = scene
    colours = theme_colours(theme)
    width, window_height = window_size
    canvas = Image.new("RGB", (width, window_height + CAPTION_HEIGHT), colours["canvas"])
    draw = ImageDraw.Draw(canvas)

    if name.endswith(".csv"):
        draw_table(draw, read_result(frames / name), colours, width, window_height)
    else:
        with Image.open(frames / name) as opened:
            shot = opened.convert("RGB")
        shot.thumbnail(window_size, Image.Resampling.LANCZOS)
        canvas.paste(shot, ((width - shot.width) // 2,
                            (window_height - shot.height) // 2))
    draw.line((0, window_height, width, window_height), fill=colours["line"], width=1)
    draw.text((18, window_height + 23), title, font=font(23, True),
              fill=colours["ink"], anchor="lm")
    draw.text((18, window_height + 51), english, font=font(15),
              fill=colours["muted"], anchor="lm")
    mode = "Dark" if theme == "dark" else "Light"
    draw.text((width - 18, window_height + 23), mode, font=font(14),
              fill=colours["muted"], anchor="rm")
    return canvas.quantize(colors=256, method=Image.Quantize.MEDIANCUT,
                           dither=Image.Dither.NONE), duration


def render(light_frames, output, dark_frames=None):
    check_captures(light_frames)
    convert_native_hero(light_frames)
    with Image.open(light_frames / "01-empty.png") as opened:
        window_size = opened.size
    if dark_frames:
        check_captures(dark_frames)
        convert_native_hero(dark_frames)
        check_theme_difference(light_frames, dark_frames)
        if read_result(light_frames / "result.csv") != read_result(dark_frames / "result.csv"):
            raise ValueError("Light and dark runs exported different CSV results")
    images = []
    durations = []
    for theme, frames in (("light", light_frames), ("dark", dark_frames)):
        if frames is None:
            continue
        for scene in SCENES:
            image, duration = render_scene(frames, scene, theme, window_size)
            images.append(image)
            durations.append(duration)
    output.parent.mkdir(exist_ok=True, parents=True)
    images[0].save(output, save_all=True, append_images=images[1:],
                   duration=durations, loop=0, disposal=2, optimize=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("frames", type=Path, help="directory with the light captures")
    parser.add_argument("output", type=Path)
    parser.add_argument("--dark-frames", type=Path)
    args = parser.parse_args()
    render(args.frames, args.output, args.dark_frames)
