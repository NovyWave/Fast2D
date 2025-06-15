use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue, CommandEncoder, TextureView, Surface};

use crate::{examples::ExampleObjects, layout::Viewport};

/// A canvas region representing one of the three example canvases
pub struct CanvasRegion {
    id: usize,
    objects: ExampleObjects,
    canvas_wrapper: fast2d::CanvasWrapper,
    current_viewport: Viewport,
}

impl CanvasRegion {
    /// Create a new canvas region with a dedicated Fast2D CanvasWrapper
    pub async fn new(
        id: usize,
        objects: ExampleObjects,
        surface: Surface<'static>,
        device: Device,
        queue: Queue,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        println!("Creating canvas region {} with {} objects", id, objects.len());
        
        // Create Fast2D CanvasWrapper with native surface
        let mut canvas_wrapper = fast2d::CanvasWrapper::new_with_surface(
            surface, device, queue, width, height
        ).await;
        
        // Initialize with the objects
        let objects_clone = objects.clone();
        canvas_wrapper.update_objects(|canvas_objects| {
            *canvas_objects = objects_clone;
        });
        
        Ok(Self {
            id,
            objects,
            canvas_wrapper,
            current_viewport: Viewport {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
        })
    }
    
    /// Update the viewport for this canvas region
    pub fn update_viewport(&mut self, viewport: Viewport) -> Result<()> {
        self.current_viewport = viewport;
        
        // Resize the Fast2D canvas wrapper
        self.canvas_wrapper.resized(viewport.width as u32, viewport.height as u32);
        
        Ok(())
    }
    
    /// Update the objects and re-render
    pub fn update_objects(&mut self, objects: ExampleObjects) {
        self.objects = objects.clone();
        self.canvas_wrapper.update_objects(|canvas_objects| {
            *canvas_objects = objects;
        });
    }
    
    /// Get the objects in this canvas region
    pub fn objects(&self) -> &ExampleObjects {
        &self.objects
    }
    
    /// Get the current viewport
    pub fn viewport(&self) -> Viewport {
        self.current_viewport
    }
    
    /// Get the canvas wrapper width
    pub fn width(&self) -> u32 {
        self.canvas_wrapper.width()
    }
    
    /// Get the canvas wrapper height  
    pub fn height(&self) -> u32 {
        self.canvas_wrapper.height()
    }
}