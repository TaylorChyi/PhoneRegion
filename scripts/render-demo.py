"""Render the README animation from real Windows captures and their exported CSV."""
import argparse
import csv
import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

WIDTH, HEIGHT = 732, 618
INK, MUTED, ACCENT = '#172d31', '#435d60', '#087360'
BORDER, PAPER = '#c7d7d6', '#eef4f3'
SOURCE = (4, 1, 631, 506)
SCENES = [
    ('01-empty.png', 700, '输入一个手机号', 'Enter a mobile number', 0),
    ('03-prefix.png', 450, '输入一个手机号', 'Enter a mobile number', 0),
    ('04-area.png', 450, '输入一个手机号', 'Enter a mobile number', 0),
    ('05-group.png', 500, '输入一个手机号', 'Enter a mobile number', 0),
    ('06-formatted-result.png', 2900, '查看原号段运营商、省份和地市', 'See the original carrier, province and city', 0),
    ('07-before-import.png', 1550, '导入号码表', 'Import a list from Excel or CSV', 1),
    ('08-open-dialog.png', 1450, '选择首列是手机号的文件', 'Choose a file with numbers in the first column', 1),
    ('09-save-dialog.png', 1450, '另存查询结果', 'Choose where to save the results', 1),
    ('10-batch-complete.png', 2200, '查看处理结果', 'Check the totals after export', 2),
    ('result.csv', 4300, '打开四列结果表', 'Open the exported file', 2),
]


def font(size, bold=False):
    if os.name == 'nt':
        path = Path(os.environ['WINDIR']) / 'Fonts' / ('msyhbd.ttc' if bold else 'msyh.ttc')
    else:
        path = Path('/System/Library/Fonts/Hiragino Sans GB.ttc')
    return ImageFont.truetype(str(path), size)


def table(draw, csv_path):
    with csv_path.open(encoding='utf-8-sig', newline='') as source:
        header, *rows = csv.reader(source)
    left, right = 42, WIDTH - 42
    draw.rounded_rectangle((left, 135, right, 535), radius=10, fill='white', outline=BORDER)
    draw.text((left + 22, 162), csv_path.name, font=font(25, True), fill=INK, anchor='lm')
    draw.rectangle((left + 1, 202, right - 1, 244), fill='#e3f0eb')
    positions = [64, 245, 403, 544]
    for idx, field in enumerate(header):
        draw.text((positions[idx], 223), field, font=font(17, True), fill=ACCENT, anchor='lm')
    for idx, row in enumerate(rows):
        y = 266 + 45 * idx
        for x, value in zip(positions, row):
            draw.text((x, y), value, font=font(18), fill=INK, anchor='lm')
        draw.line((left + 17, y + 23, right - 17, y + 23), fill='#e4ecea')
    draw.text((left + 22, 489), '格式错误的号码会保留，查询字段留空。', font=font(15), fill=MUTED, anchor='lm')
    draw.text((left + 22, 513), 'Invalid inputs stay in the file with empty lookup fields.', font=font(14), fill=MUTED, anchor='lm')


def render(frames, output):
    images, durations = [], []
    for name, duration, title, caption, chapter in SCENES:
        image = Image.new('RGB', (WIDTH, HEIGHT), PAPER)
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, WIDTH, 85), fill='white')
        draw.text((31, 29), title, font=font(25, True), fill=INK, anchor='lm')
        draw.text((32, 61), caption, font=font(17), fill=MUTED, anchor='lm')
        for idx in range(3):
            start = idx * WIDTH // 3
            draw.rectangle((start + (2 if idx else 0), 84, (idx + 1) * WIDTH // 3 - 1, 86),
                           fill=ACCENT if idx <= chapter else BORDER)
        if name.endswith('.csv'):
            table(draw, frames / name)
        else:
            with Image.open(frames / name) as opened:
                shot = opened.convert('RGB')
            if shot.size == (635, 510):
                shot = shot.crop(SOURCE)  # Remove the desktop just outside the app border.
            shot.thumbnail((WIDTH - 40, HEIGHT - 106), Image.Resampling.LANCZOS)
            x, y = (WIDTH - shot.width) // 2, 88 + (HEIGHT - 88 - shot.height) // 2
            draw.rectangle((x - 1, y - 1, x + shot.width, y + shot.height), outline=BORDER)
            image.paste(shot, (x, y))
            if name == '07-before-import.png':
                draw.rounded_rectangle((x + 20, y + 298, x + 236, y + 344),
                                       radius=5, outline=ACCENT, width=3)
        images.append(image.quantize(colors=256, method=Image.Quantize.MEDIANCUT,
                                     dither=Image.Dither.NONE))
        durations.append(duration)
    output.parent.mkdir(exist_ok=True, parents=True)
    images[0].save(output, save_all=True, append_images=images[1:], duration=durations,
                   loop=0, disposal=1, optimize=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('frames', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    render(args.frames, args.output)
