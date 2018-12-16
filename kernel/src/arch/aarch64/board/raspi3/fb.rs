//! Framebuffer

use super::mailbox;
use alloc::string::String;
use core::fmt;
use lazy_static::lazy_static;
use log::*;
use once::*;
use spin::Mutex;

/// Framebuffer information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// visible width
    pub xres: u32,
    /// visible height
    pub yres: u32,
    /// virtual width
    pub xres_virtual: u32,
    /// virtual height
    pub yres_virtual: u32,
    /// virtual offset x
    pub xoffset: u32,
    /// virtual offset y
    pub yoffset: u32,

    /// bits per pixel
    pub depth: u32,
    /// bytes per line
    pub pitch: u32,

    /// bus address, starts from 0xC0000000/0x40000000
    /// (see https://github.com/raspberrypi/firmware/wiki/Accessing-mailboxes)
    pub bus_addr: u32,
    /// screen buffer size
    pub screen_size: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ColorFormat {
    BGR565 = 16,
    RGBA8888 = 32,
}
use self::ColorFormat::*;

#[repr(C)]
union ColorBuffer {
    base_addr: u32,
    buf16: &'static mut [u16],
    buf32: &'static mut [u32],
}

impl ColorBuffer {
    fn new(color_format: ColorFormat, bus_addr: u32, size: u32) -> ColorBuffer {
        unsafe {
            match color_format {
                BGR565 => ColorBuffer {
                    buf16: core::slice::from_raw_parts_mut(
                        bus_addr as *mut u16,
                        (size / 2) as usize,
                    ),
                },
                RGBA8888 => ColorBuffer {
                    buf32: core::slice::from_raw_parts_mut(
                        bus_addr as *mut u32,
                        (size / 4) as usize,
                    ),
                },
            }
        }
    }

    #[inline]
    fn read16(&self, index: u32) -> u16 {
        unsafe { self.buf16[index as usize] }
    }

    #[inline]
    fn read32(&self, index: u32) -> u32 {
        unsafe { self.buf32[index as usize] }
    }

    #[inline]
    fn write16(&mut self, index: u32, pixel: u16) {
        unsafe { self.buf16[index as usize] = pixel }
    }

    #[inline]
    fn write32(&mut self, index: u32, pixel: u32) {
        unsafe { self.buf32[index as usize] = pixel }
    }
}

/// Frambuffer structure
pub struct Framebuffer {
    pub fb_info: FramebufferInfo,
    pub color_format: ColorFormat,
    buf: ColorBuffer,
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Framebuffer");
        f.field("fb_info", &self.fb_info);
        f.field("color_format", &self.color_format);
        f.field("base_addr", unsafe { &self.buf.base_addr });
        f.finish()
    }
}

impl Framebuffer {
    fn new(width: u32, height: u32, depth: u32) -> Result<Framebuffer, String> {
        assert_has_not_been_called!("Framebuffer::new must be called only once");

        let (width, height) = if width == 0 || height == 0 {
            mailbox::framebuffer_get_physical_size()?
        } else {
            (width, height)
        };
        let depth = if depth == 0 {
            mailbox::framebuffer_get_depth()?
        } else {
            depth
        };

        let info = mailbox::framebuffer_alloc(width, height, depth)?;
        let color_format = match info.depth {
            16 => BGR565,
            32 => RGBA8888,
            _ => Err(format!("unsupported color depth {}", info.depth))?,
        };

        if info.bus_addr == 0 || info.screen_size == 0 {
            Err(format!("mailbox call returned an invalid address/size"))?;
        }
        if info.pitch == 0 || info.pitch != info.xres * info.depth / 8 {
            Err(format!(
                "mailbox call returned an invalid pitch value {}",
                info.pitch
            ))?;
        }

        let paddr = info.bus_addr & !0xC0000000;
        Ok(Framebuffer {
            buf: ColorBuffer::new(color_format, paddr, info.screen_size),
            color_format,
            fb_info: info,
        })
    }

    #[inline]
    pub fn base_addr(&self) -> u32 {
        unsafe { self.buf.base_addr }
    }

    #[inline]
    pub fn read(&self, x: u32, y: u32) -> u32 {
        match self.color_format {
            BGR565 => self.buf.read16(y * self.fb_info.xres + x) as u32,
            RGBA8888 => self.buf.read32(y * self.fb_info.xres + x),
        }
    }

    #[inline]
    pub fn write(&mut self, x: u32, y: u32, pixel: u32) {
        match self.color_format {
            BGR565 => self.buf.write16(y * self.fb_info.xres + x, pixel as u16),
            RGBA8888 => self.buf.write32(y * self.fb_info.xres + x, pixel),
        }
    }
}

lazy_static! {
    pub static ref FRAME_BUFFER: Mutex<Option<Framebuffer>> = Mutex::new(None);
}

/// Initialize framebuffer
pub fn init() {
    match Framebuffer::new(0, 0, 0) {
        Ok(fb) => {
            let info = fb.fb_info;
            info!("framebuffer: init end\n{:#x?}", fb);
            *FRAME_BUFFER.lock() = Some(fb);
        }
        Err(err) => error!("framebuffer init failed: {}", err),
    }
}
