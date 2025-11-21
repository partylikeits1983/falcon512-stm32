//! Memory-optimized workspace for Falcon operations
//!
//! This module provides reusable buffer structures to reduce heap allocations
//! and memory fragmentation during cryptographic operations.

use crate::falcon_field::Felt;
use alloc::vec::Vec;
use num_complex::Complex64;

/// Reusable workspace for Falcon-512 operations
///
/// This struct pre-allocates buffers that can be reused across multiple
/// cryptographic operations, significantly reducing memory allocations
/// and peak memory usage.
#[derive(Clone)]
pub struct FalconWorkspace512 {
    /// FFT buffer for Complex64 operations (512 elements)
    pub fft_buffer: Vec<Complex64>,
    /// Temporary polynomial buffer for i16 coefficients
    pub temp_poly_i16: Vec<i16>,
    /// Temporary polynomial buffer for i32 coefficients
    pub temp_poly_i32: Vec<i32>,
    /// Temporary polynomial buffer for Felt coefficients
    pub(crate) temp_poly_felt: Vec<Felt>,
    /// Additional FFT buffer for intermediate calculations
    pub fft_buffer2: Vec<Complex64>,
}

impl FalconWorkspace512 {
    /// Create a new workspace with pre-allocated buffers for Falcon-512
    pub fn new() -> Self {
        const N: usize = 512;
        Self {
            fft_buffer: alloc::vec![Complex64::new(0.0, 0.0); N],
            temp_poly_i16: alloc::vec![0i16; N],
            temp_poly_i32: alloc::vec![0i32; N],
            temp_poly_felt: alloc::vec![Felt::new(0); N],
            fft_buffer2: alloc::vec![Complex64::new(0.0, 0.0); N],
        }
    }

    /// Clear all buffers (zero them out) for reuse
    pub fn clear(&mut self) {
        for elem in self.fft_buffer.iter_mut() {
            *elem = Complex64::new(0.0, 0.0);
        }
        for elem in self.temp_poly_i16.iter_mut() {
            *elem = 0;
        }
        for elem in self.temp_poly_i32.iter_mut() {
            *elem = 0;
        }
        for elem in self.temp_poly_felt.iter_mut() {
            *elem = Felt::new(0);
        }
        for elem in self.fft_buffer2.iter_mut() {
            *elem = Complex64::new(0.0, 0.0);
        }
    }
}

impl Default for FalconWorkspace512 {
    fn default() -> Self {
        Self::new()
    }
}

/// Reusable workspace for Falcon-1024 operations
#[derive(Clone)]
pub struct FalconWorkspace1024 {
    /// FFT buffer for Complex64 operations (1024 elements)
    pub fft_buffer: Vec<Complex64>,
    /// Temporary polynomial buffer for i16 coefficients
    pub temp_poly_i16: Vec<i16>,
    /// Temporary polynomial buffer for i32 coefficients
    pub temp_poly_i32: Vec<i32>,
    /// Temporary polynomial buffer for Felt coefficients
    pub(crate) temp_poly_felt: Vec<Felt>,
    /// Additional FFT buffer for intermediate calculations
    pub fft_buffer2: Vec<Complex64>,
}

impl FalconWorkspace1024 {
    /// Create a new workspace with pre-allocated buffers for Falcon-1024
    pub fn new() -> Self {
        const N: usize = 1024;
        Self {
            fft_buffer: alloc::vec![Complex64::new(0.0, 0.0); N],
            temp_poly_i16: alloc::vec![0i16; N],
            temp_poly_i32: alloc::vec![0i32; N],
            temp_poly_felt: alloc::vec![Felt::new(0); N],
            fft_buffer2: alloc::vec![Complex64::new(0.0, 0.0); N],
        }
    }

    /// Clear all buffers (zero them out) for reuse
    pub fn clear(&mut self) {
        for elem in self.fft_buffer.iter_mut() {
            *elem = Complex64::new(0.0, 0.0);
        }
        for elem in self.temp_poly_i16.iter_mut() {
            *elem = 0;
        }
        for elem in self.temp_poly_i32.iter_mut() {
            *elem = 0;
        }
        for elem in self.temp_poly_felt.iter_mut() {
            *elem = Felt::new(0);
        }
        for elem in self.fft_buffer2.iter_mut() {
            *elem = Complex64::new(0.0, 0.0);
        }
    }
}

impl Default for FalconWorkspace1024 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace512_creation() {
        let workspace = FalconWorkspace512::new();
        assert_eq!(workspace.fft_buffer.len(), 512);
        assert_eq!(workspace.temp_poly_i16.len(), 512);
        assert_eq!(workspace.temp_poly_i32.len(), 512);
        assert_eq!(workspace.temp_poly_felt.len(), 512);
    }

    #[test]
    fn test_workspace1024_creation() {
        let workspace = FalconWorkspace1024::new();
        assert_eq!(workspace.fft_buffer.len(), 1024);
        assert_eq!(workspace.temp_poly_i16.len(), 1024);
        assert_eq!(workspace.temp_poly_i32.len(), 1024);
        assert_eq!(workspace.temp_poly_felt.len(), 1024);
    }

    #[test]
    fn test_workspace_clear() {
        let mut workspace = FalconWorkspace512::new();

        // Modify some values
        workspace.fft_buffer[0] = Complex64::new(1.0, 2.0);
        workspace.temp_poly_i16[0] = 42;
        workspace.temp_poly_i32[0] = 100;

        // Clear the workspace
        workspace.clear();

        // Verify all values are reset
        assert_eq!(workspace.fft_buffer[0], Complex64::new(0.0, 0.0));
        assert_eq!(workspace.temp_poly_i16[0], 0);
        assert_eq!(workspace.temp_poly_i32[0], 0);
    }
}
