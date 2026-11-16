use std::cell::RefCell;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use ggez::graphics::{BlendMode, Canvas, DrawParam, Drawable, InstanceArray, Mesh, Rect};
use ggez::Context;

// ggez canvases don't batch, every canvas draw is its own gpu draw
// so runs of draws of the same mesh are queued here and sent as one instanced draw
// anything else drawn, or a canvas state change, sends the queue first to keep the order

// gpu draws since the last take
static DRAWS: AtomicUsize = AtomicUsize::new(0);

const MIN_POOL: usize = 16; // instance arrays to start a frame with

/// draws counted since the last call
pub fn take_draw_count() -> usize {
    DRAWS.swap(0, Ordering::Relaxed)
}

fn count(draws: usize) {
    DRAWS.fetch_add(draws, Ordering::Relaxed);
}

#[derive(Default)]
struct Batch {
    mesh: Option<(*const Mesh, Mesh)>, // the queued mesh, compared by address
    params: Vec<DrawParam>,
    // one array per batch in a frame: ggez uploads an array when it's drawn,
    // but the gpu reads it when the frame is submitted
    pool: Vec<InstanceArray>,
    used: usize,  // arrays drawn this frame
    short: usize, // batches drawn one by one for want of an array, the pool grows next frame
}

impl Batch {
    fn flush(&mut self, canvas: &mut Canvas) {
        let Some((_, mesh)) = self.mesh.take() else {
            return;
        };
        if self.params.len() > 1 {
            if let Some(instances) = self.pool.get_mut(self.used) {
                instances.set(self.params.drain(..));
                canvas.draw_instanced_mesh(mesh, instances, DrawParam::default());
                self.used += 1;
                count(1);
                return;
            }
            self.short += 1;
        }
        count(self.params.len());
        for param in self.params.drain(..) {
            canvas.draw(&mesh, param);
        }
    }
}

thread_local! {
    static BATCH: RefCell<Batch> = RefCell::default();
}

/// start a frame, sizing the pool for the last one
pub fn begin_frame(ctx: &Context) {
    BATCH.with_borrow_mut(|batch| {
        batch.mesh = None;
        batch.params.clear();
        let wanted = (batch.used + batch.short).max(MIN_POOL);
        while batch.pool.len() < wanted {
            batch.pool.push(InstanceArray::new(ctx, None));
        }
        batch.used = 0;
        batch.short = 0;
    });
}

/// send what's queued, before the canvas is finished
pub fn end_frame(canvas: &mut Canvas) {
    flush(canvas);
}

fn flush(canvas: &mut Canvas) {
    BATCH.with_borrow_mut(|batch| batch.flush(canvas));
}

/// draw a mesh, batched with the draws of the same mesh around it
pub(super) fn draw_mesh(canvas: &mut Canvas, mesh: &Mesh, param: impl Into<DrawParam>) {
    BATCH.with_borrow_mut(|batch| {
        if batch
            .mesh
            .as_ref()
            .is_some_and(|(queued, _)| !ptr::eq(*queued, mesh))
        {
            batch.flush(canvas);
        }
        if batch.mesh.is_none() {
            batch.mesh = Some((ptr::from_ref(mesh), mesh.clone()));
        }
        batch.params.push(param.into());
    });
}

/// draw anything else, after what's queued
pub(super) fn draw(canvas: &mut Canvas, drawable: &impl Drawable, param: impl Into<DrawParam>) {
    flush(canvas);
    count(1);
    canvas.draw(drawable, param);
}

/// blend mode for the draws after this
pub(super) fn set_blend_mode(canvas: &mut Canvas, mode: BlendMode) {
    flush(canvas);
    canvas.set_blend_mode(mode);
}

/// screen coordinates for the draws after this
pub(super) fn set_screen_coordinates(canvas: &mut Canvas, rect: Rect) {
    flush(canvas);
    canvas.set_screen_coordinates(rect);
}
