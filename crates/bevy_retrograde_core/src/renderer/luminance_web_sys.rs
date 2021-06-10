//! This module is forked from the luminance_web_sys crate which we modify here
//! to use WebGL1 instead of WebGL2.
//!
//! # License
//! Copyright (c) 2020, Dimitri Sabadie <dimitri.sabadie@gmail.com>
//!
//! All rights reserved.
//!
//! Redistribution and use in source and binary forms, with or without
//! modification, are permitted provided that the following conditions are met:
//!
//!     * Redistributions of source code must retain the above copyright
//!       notice, this list of conditions and the following disclaimer.
//!
//!     * Redistributions in binary form must reproduce the above
//!       copyright notice, this list of conditions and the following
//!       disclaimer in the documentation and/or other materials provided
//!       with the distribution.
//!
//!     * Neither the name of Dimitri Sabadie <dimitri.sabadie@gmail.com> nor the names of other
//!       contributors may be used to endorse or promote products derived
//!       from this software without specific prior written permission.
//!
//! THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
//! "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
//! LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
//! A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
//! OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//! SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
//! LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
//! DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
//! THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
//! (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
//! OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use luminance::context::GraphicsContext;
use luminance::framebuffer::{Framebuffer, FramebufferError};
use luminance::texture::Dim2;
use luminance_glow::{Context, Glow, StateQueryError};

use std::fmt;
use wasm_bindgen::JsCast as _;
use web_sys::{Document, HtmlCanvasElement, Window};

/// web-sys errors that might occur while initializing and using the platform.
#[non_exhaustive]
#[derive(Debug)]
pub enum WebSysWebGLSurfaceError {
    CannotGrabWindow,
    CannotGrabDocument,
    CannotGrabWebGLContext,
    NoAvailableWebGLContext,
    StateQueryError(StateQueryError),
}

impl WebSysWebGLSurfaceError {
    fn cannot_grab_window() -> Self {
        WebSysWebGLSurfaceError::CannotGrabWindow
    }

    fn cannot_grab_document() -> Self {
        WebSysWebGLSurfaceError::CannotGrabDocument
    }

    fn cannot_grab_webgl_context() -> Self {
        WebSysWebGLSurfaceError::CannotGrabWebGLContext
    }

    fn no_available_webgl_context() -> Self {
        WebSysWebGLSurfaceError::NoAvailableWebGLContext
    }
}

impl fmt::Display for WebSysWebGLSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            WebSysWebGLSurfaceError::CannotGrabWindow => {
                f.write_str("cannot grab the window node")
            }
            WebSysWebGLSurfaceError::CannotGrabDocument => {
                f.write_str("cannot grab the document node")
            }
            WebSysWebGLSurfaceError::CannotGrabWebGLContext => {
                f.write_str("cannot grab WebGL2 context")
            }
            WebSysWebGLSurfaceError::NoAvailableWebGLContext => {
                f.write_str("no available WebGL2 context")
            }
            WebSysWebGLSurfaceError::StateQueryError(ref e) => {
                write!(f, "WebGL2 state query error: {}", e)
            }
        }
    }
}

impl std::error::Error for WebSysWebGLSurfaceError {}

impl From<StateQueryError> for WebSysWebGLSurfaceError {
    fn from(e: StateQueryError) -> Self {
        WebSysWebGLSurfaceError::StateQueryError(e)
    }
}

pub struct WebSysWebGLSurface {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,
    backend: Glow,
}

impl WebSysWebGLSurface {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Self, WebSysWebGLSurfaceError> {
        let window =
            web_sys::window().ok_or_else(|| WebSysWebGLSurfaceError::cannot_grab_window())?;
        let document = window
            .document()
            .ok_or_else(|| WebSysWebGLSurfaceError::cannot_grab_document())?;

        let webgl = canvas
            .get_context("webgl")
            .map_err(|_| WebSysWebGLSurfaceError::cannot_grab_webgl_context())?
            .ok_or_else(|| WebSysWebGLSurfaceError::no_available_webgl_context())?;

        let ctx = Context::from_webgl1_context(webgl.dyn_into().unwrap());

        // create the backend object and return the whole object
        let backend = Glow::from_context(ctx)?;

        Ok(Self {
            window,
            document,
            canvas,
            backend,
        })
    }

    /// Get the back buffer.
    pub fn back_buffer(&mut self) -> Result<Framebuffer<Glow, Dim2, (), ()>, FramebufferError> {
        let dim = [self.canvas.width(), self.canvas.height()];
        Framebuffer::back_buffer(self, dim)
    }
}

unsafe impl GraphicsContext for WebSysWebGLSurface {
    type Backend = Glow;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.backend
    }
}
