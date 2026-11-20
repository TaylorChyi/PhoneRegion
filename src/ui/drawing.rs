//! Antialiased Windows GDI+ primitives. All dimensions use 96-DPI logical pixels.

use std::ffi::c_void;
use std::ptr::null_mut;

type GpHandle = *mut c_void;
const UNIT_PIXEL: i32 = 2;
const SMOOTHING_ANTIALIAS: i32 = 4;
const PIXEL_OFFSET_HIGH_QUALITY: i32 = 2;
const MATRIX_ORDER_APPEND: i32 = 1;

#[repr(C)]
struct StartupInput {
    version: u32,
    debug_callback: GpHandle,
    suppress_background_thread: i32,
    suppress_external_codecs: i32,
}

#[link(name = "gdiplus")]
unsafe extern "system" {
    fn GdiplusStartup(token: *mut usize, input: *const StartupInput, output: *mut c_void) -> i32;
    fn GdiplusShutdown(token: usize);
    fn GdipCreateFromHDC(hdc: GpHandle, graphics: *mut GpHandle) -> i32;
    fn GdipDeleteGraphics(graphics: GpHandle) -> i32;
    fn GdipSetSmoothingMode(graphics: GpHandle, mode: i32) -> i32;
    fn GdipSetPixelOffsetMode(graphics: GpHandle, mode: i32) -> i32;
    fn GdipScaleWorldTransform(graphics: GpHandle, sx: f32, sy: f32, order: i32) -> i32;
    fn GdipCreateSolidFill(argb: u32, brush: *mut GpHandle) -> i32;
    fn GdipDeleteBrush(brush: GpHandle) -> i32;
    fn GdipCreatePen1(argb: u32, width: f32, unit: i32, pen: *mut GpHandle) -> i32;
    fn GdipDeletePen(pen: GpHandle) -> i32;
    fn GdipCreatePath(fill_mode: i32, path: *mut GpHandle) -> i32;
    fn GdipDeletePath(path: GpHandle) -> i32;
    fn GdipAddPathArc(
        path: GpHandle,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        start: f32,
        sweep: f32,
    ) -> i32;
    fn GdipAddPathRectangle(path: GpHandle, x: f32, y: f32, w: f32, h: f32) -> i32;
    fn GdipClosePathFigure(path: GpHandle) -> i32;
    fn GdipFillPath(graphics: GpHandle, brush: GpHandle, path: GpHandle) -> i32;
    fn GdipDrawPath(graphics: GpHandle, pen: GpHandle, path: GpHandle) -> i32;
    fn GdipFillEllipse(graphics: GpHandle, brush: GpHandle, x: f32, y: f32, w: f32, h: f32) -> i32;
    fn GdipDrawEllipse(graphics: GpHandle, pen: GpHandle, x: f32, y: f32, w: f32, h: f32) -> i32;
    fn GdipDrawLine(graphics: GpHandle, pen: GpHandle, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
}

fn check(status: i32, action: &str) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("GDI+ {action} failed ({status})"))
    }
}

fn argb(rgb: u32) -> u32 {
    0xff00_0000 | (rgb & 0x00ff_ffff)
}

pub struct Canvas {
    graphics: GpHandle,
}

struct GdiPlus(usize);

impl GdiPlus {
    fn start() -> Result<Self, String> {
        let input = StartupInput {
            version: 1,
            debug_callback: null_mut(),
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        };
        let mut token = 0usize;
        // SAFETY: GDI+ receives a valid startup struct and an output token slot.
        check(
            unsafe { GdiplusStartup(&mut token, &input, null_mut()) },
            "startup",
        )?;
        Ok(Self(token))
    }
}

impl Drop for GdiPlus {
    fn drop(&mut self) {
        // SAFETY: This token uniquely belongs to the current UI thread's GDI+ runtime.
        unsafe { GdiplusShutdown(self.0) };
    }
}

thread_local! {
    // A Windows UI window and its child controls paint on the creating thread.
    // The runtime is started once and released when that thread exits.
    static GDIPLUS: Result<GdiPlus, String> = GdiPlus::start();
}

impl Canvas {
    /// `hdc` must remain valid until the canvas is dropped. Coordinates and pen widths
    /// are 96-DPI logical pixels and scale together for the supplied monitor DPI.
    pub fn new(hdc: GpHandle, dpi: u32) -> Result<Self, String> {
        GDIPLUS.with(|runtime| runtime.as_ref().map(|_| ()).map_err(Clone::clone))?;
        let mut graphics = null_mut();
        // SAFETY: The caller owns a live HDC for the duration of Canvas.
        check(
            unsafe { GdipCreateFromHDC(hdc, &mut graphics) },
            "create graphics",
        )?;
        let canvas = Self { graphics };
        let scale = dpi.max(1) as f32 / 96.0;
        // SAFETY: graphics is a live GDI+ graphics handle owned by Canvas.
        check(
            unsafe { GdipSetSmoothingMode(graphics, SMOOTHING_ANTIALIAS) },
            "smoothing",
        )?;
        check(
            unsafe { GdipSetPixelOffsetMode(graphics, PIXEL_OFFSET_HIGH_QUALITY) },
            "pixel offset",
        )?;
        check(
            unsafe { GdipScaleWorldTransform(graphics, scale, scale, MATRIX_ORDER_APPEND) },
            "DPI transform",
        )?;
        Ok(canvas)
    }

    pub fn fill_round_rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        color: u32,
    ) -> Result<(), String> {
        let path = self.round_rect_path(x, y, w, h, r)?;
        let brush = Brush::new(color)?;
        // SAFETY: All handles are live for this draw call.
        check(
            unsafe { GdipFillPath(self.graphics, brush.0, path.0) },
            "fill round rectangle",
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn stroke_round_rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        width: f32,
        color: u32,
    ) -> Result<(), String> {
        let path = self.round_rect_path(x, y, w, h, r)?;
        let pen = Pen::new(width, color)?;
        // SAFETY: All handles are live for this draw call.
        check(
            unsafe { GdipDrawPath(self.graphics, pen.0, path.0) },
            "stroke round rectangle",
        )
    }

    pub fn fill_circle(&self, cx: f32, cy: f32, r: f32, color: u32) -> Result<(), String> {
        let brush = Brush::new(color)?;
        // SAFETY: All handles are live for this draw call.
        check(
            unsafe { GdipFillEllipse(self.graphics, brush.0, cx - r, cy - r, 2.0 * r, 2.0 * r) },
            "fill circle",
        )
    }

    pub fn stroke_circle(
        &self,
        cx: f32,
        cy: f32,
        r: f32,
        width: f32,
        color: u32,
    ) -> Result<(), String> {
        let pen = Pen::new(width, color)?;
        // SAFETY: All handles are live for this draw call.
        check(
            unsafe { GdipDrawEllipse(self.graphics, pen.0, cx - r, cy - r, 2.0 * r, 2.0 * r) },
            "stroke circle",
        )
    }

    pub fn line(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: u32,
    ) -> Result<(), String> {
        let pen = Pen::new(width, color)?;
        // SAFETY: All handles are live for this draw call.
        check(
            unsafe { GdipDrawLine(self.graphics, pen.0, x1, y1, x2, y2) },
            "draw line",
        )
    }

    fn round_rect_path(&self, x: f32, y: f32, w: f32, h: f32, r: f32) -> Result<Path, String> {
        let path = Path::new()?;
        let radius = r.max(0.0).min(w.abs() / 2.0).min(h.abs() / 2.0);
        // SAFETY: path is valid for all following path operations.
        if radius == 0.0 {
            check(
                unsafe { GdipAddPathRectangle(path.0, x, y, w, h) },
                "add rectangle",
            )?;
        } else {
            let diameter = 2.0 * radius;
            check(
                unsafe {
                    GdipAddPathArc(path.0, x + w - diameter, y, diameter, diameter, -90.0, 90.0)
                },
                "add top right arc",
            )?;
            check(
                unsafe {
                    GdipAddPathArc(
                        path.0,
                        x + w - diameter,
                        y + h - diameter,
                        diameter,
                        diameter,
                        0.0,
                        90.0,
                    )
                },
                "add bottom right arc",
            )?;
            check(
                unsafe {
                    GdipAddPathArc(path.0, x, y + h - diameter, diameter, diameter, 90.0, 90.0)
                },
                "add bottom left arc",
            )?;
            check(
                unsafe { GdipAddPathArc(path.0, x, y, diameter, diameter, 180.0, 90.0) },
                "add top left arc",
            )?;
            check(unsafe { GdipClosePathFigure(path.0) }, "close path")?;
        }
        Ok(path)
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        // SAFETY: Canvas uniquely owns this graphics handle.
        unsafe {
            GdipDeleteGraphics(self.graphics);
        }
    }
}

struct Brush(GpHandle);
impl Brush {
    fn new(color: u32) -> Result<Self, String> {
        let mut handle = null_mut();
        // SAFETY: handle is an output slot and ARGB is passed by value.
        check(
            unsafe { GdipCreateSolidFill(argb(color), &mut handle) },
            "create brush",
        )?;
        Ok(Self(handle))
    }
}
impl Drop for Brush {
    fn drop(&mut self) {
        unsafe { GdipDeleteBrush(self.0) };
    }
}

struct Pen(GpHandle);
impl Pen {
    fn new(width: f32, color: u32) -> Result<Self, String> {
        let mut handle = null_mut();
        // SAFETY: handle is an output slot and width/color are passed by value.
        check(
            unsafe { GdipCreatePen1(argb(color), width.max(0.5), UNIT_PIXEL, &mut handle) },
            "create pen",
        )?;
        Ok(Self(handle))
    }
}
impl Drop for Pen {
    fn drop(&mut self) {
        unsafe { GdipDeletePen(self.0) };
    }
}

struct Path(GpHandle);
impl Path {
    fn new() -> Result<Self, String> {
        let mut handle = null_mut();
        // SAFETY: handle is an output slot. Fill mode 0 is Alternate.
        check(unsafe { GdipCreatePath(0, &mut handle) }, "create path")?;
        Ok(Self(handle))
    }
}
impl Drop for Path {
    fn drop(&mut self) {
        unsafe { GdipDeletePath(self.0) };
    }
}
