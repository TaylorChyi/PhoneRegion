"""Create a bilingual landscape GIF from verified Windows captures."""

import argparse
import csv
import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageOps, ImageStat


WIDTH, HEIGHT = 1200, 660
SHOT_BOX = (18, 18, 868, 642)
PANEL_LEFT = 902
SCENES = [
    ("01-empty.png", 700, "输入手机号", "Enter a mobile number"),
    ("02-single-result.png", 1400, "查看单号结果", "See the result instantly"),
    ("03-prefix.png", 380, "支持中国区号", "Use the +86 country code"),
    ("04-area.png", 380, "号码可以分组", "Spaces are accepted"),
    ("05-group.png", 420, "继续输入号码", "Keep entering the number"),
    ("06-formatted-result.png", 2100, "直接显示归属地", "See carrier, province and city"),
    ("07-before-import.png", 1000, "批量导入", "Import Excel or CSV"),
    ("08-open-dialog.png", 1150, "选择号码文件", "Choose the input file"),
    ("09-save-dialog.png", 1150, "选择结果位置", "Choose where to save"),
    ("10-batch-complete.png", 1600, "导出完成", "See the processed row count"),
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
            "line": "#555555", "table": "#242424", "table_head": "#303030",
        }
    return {
        "canvas": "#fafafa", "ink": "#202020", "muted": "#555555",
        "line": "#d3d3d3", "table": "#ffffff", "table_head": "#f0f0f0",
    }


def check_captures(frames):
    for name, _, _, _ in SCENES:
        if name.endswith(".csv"):
            continue
        with Image.open(frames / name) as opened:
            shot = opened.convert("RGB")
        if min(shot.size) < 350 or max(ImageStat.Stat(shot).stddev) < 8:
            raise ValueError(f"Unusable Windows capture: {frames / name}")
        if name not in {"08-open-dialog.png", "09-save-dialog.png"}:
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


def draw_table(draw, rows, colours):
    x0, y0, x1, y1 = SHOT_BOX
    draw.rounded_rectangle((x0, y0, x1, y1), radius=13,
                           fill=colours["table"], outline=colours["line"], width=2)
    draw.text((x0 + 24, y0 + 38), "result.csv", font=font(26, True),
              fill=colours["ink"], anchor="lm")
    draw.rectangle((x0 + 2, y0 + 78, x1 - 2, y0 + 130), fill=colours["table_head"])
    column_x = (x0 + 20, x0 + 195, x0 + 352, x0 + 506)
    for x, value in zip(column_x, rows[0]):
        draw.text((x, y0 + 104), value, font=font(17, True),
                  fill=colours["ink"], anchor="lm")
    for index, row in enumerate(rows[1:]):
        y = y0 + 163 + index * 70
        for x, value in zip(column_x, row):
            draw.text((x, y), value, font=font(18), fill=colours["ink"], anchor="lm")
        draw.line((x0 + 18, y + 30, x1 - 18, y + 30), fill=colours["line"], width=1)
    draw.text((x0 + 20, y1 - 28), "格式错误的号码保留原值，查询字段留空",
              font=font(15), fill=colours["muted"], anchor="lm")


def render_scene(frames, scene, theme):
    name, duration, title, english = scene
    colours = theme_colours(theme)
    canvas = Image.new("RGB", (WIDTH, HEIGHT), colours["canvas"])
    draw = ImageDraw.Draw(canvas)
    draw.line((PANEL_LEFT - 18, 24, PANEL_LEFT - 18, HEIGHT - 24),
              fill=colours["line"], width=2)
    draw.text((PANEL_LEFT, 195), title, font=font(27, True),
              fill=colours["ink"], anchor="lm")
    draw.text((PANEL_LEFT, 240), english, font=font(17),
              fill=colours["muted"], anchor="lm")
    mode = "深色模式  Dark" if theme == "dark" else "浅色模式  Light"
    draw.text((PANEL_LEFT, HEIGHT - 49), mode, font=font(16),
              fill=colours["muted"], anchor="lm")

    if name.endswith(".csv"):
        draw_table(draw, read_result(frames / name), colours)
    else:
        with Image.open(frames / name) as opened:
            shot = opened.convert("RGB")
        x0, y0, x1, y1 = SHOT_BOX
        shot.thumbnail((x1 - x0, y1 - y0), Image.Resampling.LANCZOS)
        canvas.paste(shot, (x0 + (x1 - x0 - shot.width) // 2,
                            y0 + (y1 - y0 - shot.height) // 2))
    return canvas.quantize(colors=256, method=Image.Quantize.MEDIANCUT,
                           dither=Image.Dither.NONE), duration


def render(light_frames, output, dark_frames=None):
    check_captures(light_frames)
    if dark_frames:
        check_captures(dark_frames)
        check_theme_difference(light_frames, dark_frames)
        if read_result(light_frames / "result.csv") != read_result(dark_frames / "result.csv"):
            raise ValueError("Light and dark runs exported different CSV results")
    images = []
    durations = []
    for theme, frames in (("light", light_frames), ("dark", dark_frames)):
        if frames is None:
            continue
        for scene in SCENES:
            image, duration = render_scene(frames, scene, theme)
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
