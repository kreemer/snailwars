#!/usr/bin/env python3
"""Generates original, code-drawn enemy artwork inspired by hand-drawn
crayon sketches (see raw/IMG_*.jpeg for the original inspiration
drawings - a colorful snail with a spiral shell, a simple slug, and a
larger snail), rather than converting the photographs themselves.

Each sprite is built from smooth wobbly outlines (Catmull-Rom splines
perturbed with low-frequency noise for a hand-drawn feel), rendered onto
a transparent canvas at a high supersampled resolution and then
downscaled for anti-aliasing, and cropped tightly to its content.

Usage: python3 generate.py
Run from assets/enemy/; writes snail.png, slug.png, big_snail.png.
"""

import math
import random
from dataclasses import dataclass, field
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).parent

SUPERSAMPLE = 4
CANVAS = 512 * SUPERSAMPLE
MAX_OUTPUT_SIZE = 384
CROP_PADDING = 14


def catmull_rom(points, samples_per_segment=24, closed=True):
    """Smoothly interpolate a closed loop of control points."""
    pts = list(points)
    n = len(pts)

    def get(i):
        return pts[i % n] if closed else pts[max(0, min(n - 1, i))]

    result = []
    segment_count = n if closed else n - 1
    for i in range(segment_count):
        p0, p1, p2, p3 = get(i - 1), get(i), get(i + 1), get(i + 2)
        for s in range(samples_per_segment):
            t = s / samples_per_segment
            t2 = t * t
            t3 = t2 * t
            x = 0.5 * (
                (2 * p1[0])
                + (-p0[0] + p2[0]) * t
                + (2 * p0[0] - 5 * p1[0] + 4 * p2[0] - p3[0]) * t2
                + (-p0[0] + 3 * p1[0] - 3 * p2[0] + p3[0]) * t3
            )
            y = 0.5 * (
                (2 * p1[1])
                + (-p0[1] + p2[1]) * t
                + (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2
                + (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3
            )
            result.append((x, y))
    return result


def wobble(points, amplitude, rng, closed=True, harmonics=3):
    """Perturb a smooth curve outward/inward with a few random
    low-frequency sine harmonics, along each point's local normal, to
    give an otherwise-perfect spline a hand-drawn "shaky pencil" look."""
    n = len(points)
    phases = [rng.uniform(0, math.tau) for _ in range(harmonics)]
    freqs = [rng.uniform(1.0, 2.5) * (i + 1) for i in range(harmonics)]
    amps = [amplitude / (i + 1.5) for i in range(harmonics)]

    out = []
    for i, (x, y) in enumerate(points):
        prev_pt = points[i - 1] if i > 0 else points[-1]
        next_pt = points[(i + 1) % n] if closed else points[min(n - 1, i + 1)]
        tx, ty = next_pt[0] - prev_pt[0], next_pt[1] - prev_pt[1]
        length = math.hypot(tx, ty) or 1.0
        nx, ny = -ty / length, tx / length

        t = i / n
        offset = sum(a * math.sin(f * math.tau * t + p) for a, f, p in zip(amps, freqs, phases))
        out.append((x + nx * offset, y + ny * offset))
    return out


def to_canvas(points):
    return [(x * CANVAS, y * CANVAS) for x, y in points]


@dataclass
class Palette:
    body: tuple
    body_shade: tuple
    shell: tuple
    shell_shade: tuple
    outline: tuple
    eye: tuple


def scribble_fill(draw, polygon, color, rng, stroke_count, width, bbox):
    """Fill a region with loose parallel-ish crayon strokes (instead of
    a flat color) for a scribbled, crayon-shaded texture. Strokes are
    clipped to the polygon's bounding box and drawn with varied length
    and slight angle jitter, similar to the reference drawings' shading."""
    x0, y0, x1, y1 = bbox
    base_angle = rng.uniform(-0.35, 0.35)
    for _ in range(stroke_count):
        angle = base_angle + rng.uniform(-0.12, 0.12)
        cx = rng.uniform(x0, x1)
        cy = rng.uniform(y0, y1)
        length = rng.uniform((x1 - x0) * 0.12, (x1 - x0) * 0.35)
        dx, dy = math.cos(angle) * length / 2, math.sin(angle) * length / 2
        draw.line(
            [(cx - dx, cy - dy), (cx + dx, cy + dy)],
            fill=color,
            width=width,
        )


def draw_wobbly_polygon(
    img,
    control_points,
    fill,
    outline,
    outline_width,
    rng,
    wobble_amplitude,
    shade_color=None,
    shade_strokes=0,
):
    smooth = catmull_rom(control_points, samples_per_segment=28)
    shaky = wobble(smooth, wobble_amplitude, rng)
    canvas_pts = to_canvas(shaky)

    draw = ImageDraw.Draw(img)
    draw.polygon(canvas_pts, fill=fill)

    if shade_color and shade_strokes:
        xs = [p[0] for p in canvas_pts]
        ys = [p[1] for p in canvas_pts]
        bbox = (min(xs), min(ys), max(xs), max(ys))
        mask = Image.new("L", img.size, 0)
        ImageDraw.Draw(mask).polygon(canvas_pts, fill=255)
        shade_layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
        scribble_fill(
            ImageDraw.Draw(shade_layer),
            canvas_pts,
            shade_color,
            rng,
            shade_strokes,
            width=max(2, int(6 * SUPERSAMPLE)),
            bbox=bbox,
        )
        shade_layer.putalpha(Image.composite(shade_layer.getchannel("A"), Image.new("L", img.size, 0), mask))
        img.alpha_composite(shade_layer)

    draw = ImageDraw.Draw(img)
    draw.line(canvas_pts + [canvas_pts[0]], fill=outline, width=outline_width, joint="curve")
    return canvas_pts


def draw_spiral(img, center, radius, turns, color, width, rng):
    points = []
    steps = 200
    for i in range(steps + 1):
        t = i / steps
        angle = t * turns * math.tau
        r = radius * t
        wob = 1.0 + 0.02 * math.sin(angle * 3 + rng.uniform(0, math.tau))
        x = center[0] + math.cos(angle) * r * wob
        y = center[1] + math.sin(angle) * r * wob
        points.append((x, y))
    ImageDraw.Draw(img).line(points, fill=color, width=width, joint="curve")


def draw_eye_stalk(img, base, tip, stalk_color, eye_color, stalk_width, eye_radius):
    draw = ImageDraw.Draw(img)
    draw.line([base, tip], fill=stalk_color, width=stalk_width, joint="curve")
    draw.ellipse(
        [tip[0] - eye_radius, tip[1] - eye_radius, tip[0] + eye_radius, tip[1] + eye_radius],
        fill=(20, 20, 20, 255),
    )


def make_body_control_points(hump_height, tail_length, tail_droop):
    """Unit-space (0..1) control polygon for a snail/slug body: a
    rounded hump on the right blending into a long tapering tail curving
    to the left, matching the silhouette of the reference sketches."""
    return [
        (0.62, 1.0 - hump_height),  # top of hump
        (0.78, 1.0 - hump_height * 0.75),
        (0.86, 0.78),  # neck/head area (right side)
        (0.84, 0.9),
        (0.7, 0.98),
        (0.5, 1.0),
        (0.5 - tail_length * 0.5, 1.0 + tail_droop * 0.3),
        (0.5 - tail_length, 1.0 + tail_droop),  # tail tip
        (0.5 - tail_length * 0.7, 0.9 + tail_droop * 0.5),
        (0.4, 0.85),
        (0.3, hump_height * 0.3 + 0.55),
        (0.4, 1.0 - hump_height * 1.05),
        (0.5, 1.0 - hump_height * 1.15),
    ]


def generate_snail(seed, palette, out_name, big=False):
    rng = random.Random(seed)
    img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))

    hump_height = 0.34 if not big else 0.4
    body_ctrl = make_body_control_points(hump_height, tail_length=0.42, tail_droop=0.05)
    draw_wobbly_polygon(
        img,
        body_ctrl,
        fill=palette.body,
        outline=palette.outline,
        outline_width=max(3, int(5 * SUPERSAMPLE)),
        rng=rng,
        wobble_amplitude=0.012,
        shade_color=palette.body_shade,
        shade_strokes=90,
    )

    shell_cx, shell_cy = 0.42 * CANVAS, (1.0 - hump_height * 1.05) * CANVAS * 0.62
    shell_radius = CANVAS * (0.3 if not big else 0.36)
    shell_ctrl = [
        (
            0.5 + math.cos(a) * 0.34 * (1.15 if not big else 1.3),
            0.55 + math.sin(a) * 0.34 * (1.15 if not big else 1.3) * 0.9 - hump_height * 0.5,
        )
        for a in [i * math.tau / 10 for i in range(10)]
    ]
    shell_canvas_pts = draw_wobbly_polygon(
        img,
        shell_ctrl,
        fill=palette.shell,
        outline=palette.outline,
        outline_width=max(3, int(5 * SUPERSAMPLE)),
        rng=rng,
        wobble_amplitude=0.01,
        shade_color=palette.shell_shade,
        shade_strokes=60,
    )
    sxs = [p[0] for p in shell_canvas_pts]
    sys_ = [p[1] for p in shell_canvas_pts]
    shell_center = (sum(sxs) / len(sxs), sum(sys_) / len(sys_))
    draw_spiral(
        img,
        shell_center,
        shell_radius * 0.62,
        turns=1.8,
        color=palette.outline,
        width=max(3, int(4 * SUPERSAMPLE)),
        rng=rng,
    )

    head_x, head_y = 0.83 * CANVAS, 0.66 * CANVAS
    for dx in (-0.045, 0.045):
        base = (head_x + dx * CANVAS * 0.3, head_y + 0.05 * CANVAS)
        tip = (head_x + dx * CANVAS * 1.6, head_y - 0.16 * CANVAS)
        draw_eye_stalk(
            img,
            base,
            tip,
            stalk_color=palette.outline,
            eye_color=palette.eye,
            stalk_width=max(3, int(4 * SUPERSAMPLE)),
            eye_radius=CANVAS * 0.018,
        )

    mouth_ctrl = [
        (head_x / CANVAS - 0.04, head_y / CANVAS + 0.05),
        (head_x / CANVAS, head_y / CANVAS + 0.08),
        (head_x / CANVAS + 0.04, head_y / CANVAS + 0.05),
    ]
    smooth_mouth = catmull_rom(mouth_ctrl, samples_per_segment=16, closed=False)
    ImageDraw.Draw(img).line(
        to_canvas(smooth_mouth), fill=palette.outline, width=max(2, int(3 * SUPERSAMPLE)), joint="curve"
    )

    save_cropped(img, out_name)


def generate_slug(seed, palette, out_name):
    rng = random.Random(seed)
    img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))

    body_ctrl = make_body_control_points(hump_height=0.16, tail_length=0.5, tail_droop=0.03)
    draw_wobbly_polygon(
        img,
        body_ctrl,
        fill=palette.body,
        outline=palette.outline,
        outline_width=max(3, int(5 * SUPERSAMPLE)),
        rng=rng,
        wobble_amplitude=0.014,
        shade_color=palette.body_shade,
        shade_strokes=100,
    )

    head_x, head_y = 0.83 * CANVAS, 0.66 * CANVAS
    for dx in (-0.045, 0.045):
        base = (head_x + dx * CANVAS * 0.3, head_y + 0.03 * CANVAS)
        tip = (head_x + dx * CANVAS * 1.6, head_y - 0.18 * CANVAS)
        draw_eye_stalk(
            img,
            base,
            tip,
            stalk_color=palette.outline,
            eye_color=palette.eye,
            stalk_width=max(3, int(4 * SUPERSAMPLE)),
            eye_radius=CANVAS * 0.018,
        )

    mouth_ctrl = [
        (head_x / CANVAS - 0.04, head_y / CANVAS + 0.05),
        (head_x / CANVAS, head_y / CANVAS + 0.08),
        (head_x / CANVAS + 0.04, head_y / CANVAS + 0.05),
    ]
    smooth_mouth = catmull_rom(mouth_ctrl, samples_per_segment=16, closed=False)
    ImageDraw.Draw(img).line(
        to_canvas(smooth_mouth), fill=palette.outline, width=max(2, int(3 * SUPERSAMPLE)), joint="curve"
    )

    save_cropped(img, out_name)


def save_cropped(img, out_name):
    alpha = img.getchannel("A")
    bbox = alpha.getbbox()
    if bbox is None:
        raise RuntimeError(f"generated image for {out_name} is empty")
    x0, y0, x1, y1 = bbox
    pad = CROP_PADDING * SUPERSAMPLE
    x0 = max(0, x0 - pad)
    y0 = max(0, y0 - pad)
    x1 = min(img.width, x1 + pad)
    y1 = min(img.height, y1 + pad)
    cropped = img.crop((x0, y0, x1, y1))

    w, h = cropped.size
    scale = MAX_OUTPUT_SIZE / max(w, h)
    final = cropped.resize((max(1, int(w * scale)), max(1, int(h * scale))), Image.LANCZOS)
    final.save(HERE / f"{out_name}.png")
    print(f"generated {out_name}.png ({final.size[0]}x{final.size[1]})")


def main():
    snail_palette = Palette(
        body=(196, 120, 46, 255),
        body_shade=(150, 88, 30, 130),
        shell=(224, 148, 44, 255),
        shell_shade=(163, 96, 20, 140),
        outline=(94, 51, 18, 255),
        eye=(30, 20, 10, 255),
    )
    slug_palette = Palette(
        body=(150, 90, 176, 255),
        body_shade=(108, 56, 138, 140),
        shell=(0, 0, 0, 0),
        shell_shade=(0, 0, 0, 0),
        outline=(78, 34, 102, 255),
        eye=(20, 10, 25, 255),
    )
    big_snail_palette = Palette(
        body=(176, 46, 40, 255),
        body_shade=(120, 24, 20, 140),
        shell=(196, 60, 52, 255),
        shell_shade=(132, 28, 24, 150),
        outline=(74, 16, 14, 255),
        eye=(20, 10, 10, 255),
    )

    generate_snail(seed=1, palette=snail_palette, out_name="snail", big=False)
    generate_slug(seed=2, palette=slug_palette, out_name="slug")
    generate_snail(seed=3, palette=big_snail_palette, out_name="big_snail", big=True)


if __name__ == "__main__":
    main()
