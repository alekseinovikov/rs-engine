//! A GPU buffer that grows to fit whatever the frame uploads.
//!
//! A GPU buffer has a fixed size, chosen when it is created. The number of shapes changes from
//! frame to frame, so instead of guessing a maximum we start small and, when a frame needs more
//! room, replace the buffer with one at least twice as large. Doubling means a frame that keeps
//! growing causes only a handful of reallocations (1 KiB → 2 → 4 → … → 1 MiB is 10 steps), and
//! the buffer never shrinks, so a steady scene allocates nothing after the first frames. `Vec`
//! grows the same way.

/// The smallest buffer we create, in bytes.
const MIN_CAPACITY: u64 = 1024;

/// The capacity to grow to so that `needed` bytes fit, or `None` if `current` is already enough.
fn grown_capacity(current: u64, needed: u64) -> Option<u64> {
    if needed <= current {
        return None;
    }
    let mut capacity = current.max(MIN_CAPACITY);
    while capacity < needed {
        capacity *= 2;
    }
    Some(capacity)
}

/// A vertex or index buffer that is rewritten every frame and grows on demand.
#[derive(Debug)]
pub(crate) struct GrowableBuffer {
    label: &'static str,
    usage: wgpu::BufferUsages,
    buffer: wgpu::Buffer,
}

impl GrowableBuffer {
    /// Creates a buffer of the minimum size. `usage` says what it is for (`VERTEX` or `INDEX`);
    /// `COPY_DST` is added so the CPU can write into it.
    pub(crate) fn new(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        let buffer = create(device, label, usage, MIN_CAPACITY);
        Self {
            label,
            usage,
            buffer,
        }
    }

    /// Copies `data` to the start of the buffer, growing it first if it is too small.
    ///
    /// `queue.write_buffer` does not copy right away: wgpu stages the bytes and performs the copy
    /// at the start of the next `queue.submit`, before the frame's draw commands run.
    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        if let Some(capacity) = grown_capacity(self.buffer.size(), data.len() as u64) {
            log::debug!("growing the {} to {capacity} bytes", self.label);
            self.buffer = create(device, self.label, self.usage, capacity);
        }
        queue.write_buffer(&self.buffer, 0, data);
    }

    /// The underlying wgpu buffer, for binding it in a render pass.
    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

fn create(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    size: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_that_fits_needs_no_growth() {
        assert_eq!(grown_capacity(1024, 0), None);
        assert_eq!(grown_capacity(1024, 1024), None);
    }

    #[test]
    fn growth_doubles_until_the_data_fits() {
        assert_eq!(grown_capacity(1024, 1025), Some(2048));
        assert_eq!(grown_capacity(1024, 5000), Some(8192));
    }

    #[test]
    fn growth_starts_from_the_minimum_capacity() {
        assert_eq!(grown_capacity(0, 10), Some(MIN_CAPACITY));
    }
}
