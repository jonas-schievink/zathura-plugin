//! Wrapper around `document.h` functions acting on `zathura_document_t`.

use {
    crate::{sys, PageRef},
    std::{ffi::CStr, marker::PhantomData, str::Utf8Error},
};

/// A mutable reference to a Zathura document.
#[derive(Debug)]
pub struct DocumentRef<'a> {
    ptr: *mut sys::zathura_document_t,
    _p: PhantomData<&'a mut ()>,
}

impl DocumentRef<'_> {
    /// Creates a document reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// This method requires that `ptr` points to a valid Zathura document. The
    /// returned `DocumentRef` also has a dangling lifetime, which must be
    /// constrained by the caller so that it doesn't outlive the document.
    ///
    /// Only a single `DocumentRef` may be created for the same document, since
    /// it is effectively a mutable reference. A `DocumentRef` may not coexist
    /// with `PageRef`s to pages in the document either, since they can be
    /// mutably accessed via `page`.
    pub unsafe fn from_raw(ptr: *mut sys::zathura_document_t) -> Self {
        Self {
            ptr,
            _p: PhantomData,
        }
    }

    /// Returns the file path as a raw C string.
    ///
    /// If the document was loaded from a URI, this will return a temporary file
    /// path.
    pub fn path_raw(&self) -> &CStr {
        unsafe {
            let ptr = sys::zathura_document_get_path(self.ptr);

            assert!(!ptr.is_null(), "`zathura_document_get_path` returned NULL");

            CStr::from_ptr(ptr)
        }
    }

    /// Returns the file path from which this document was or will be loaded.
    ///
    /// If the document was loaded from a URI and not a local file path, this
    /// will return a temporary file path. Returns a `Utf8Error` when the raw
    /// path does not contain valid UTF-8.
    pub fn path_utf8(&self) -> Result<&str, Utf8Error> {
        self.path_raw().to_str()
    }

    /// Returns the URI this document was loaded from.
    ///
    /// If the document was loaded from a local file path instead of a URI, this
    /// will return `None`.
    pub fn uri_raw(&self) -> Option<&CStr> {
        unsafe {
            let ptr = sys::zathura_document_get_uri(self.ptr);

            // This isn't documented, but the URI will be NULL if the document
            // was loaded from a local path
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr))
            }
        }
    }

    /// Returns the URI from which this document was or will be loaded.
    ///
    /// Returns `None` if the document was loaded from a local file path and not
    /// a URI. Returns a `Utf8Error` when the raw URI does not contain valid
    /// UTF-8.
    pub fn uri_utf8(&self) -> Option<Result<&str, Utf8Error>> {
        self.uri_raw().map(CStr::to_str)
    }

    /// Returns the raw basename of the document's path.
    ///
    /// If the document was loaded from a URI, this will return the URI's
    /// basename. Otherwise, this will return the basename of the local file
    /// path.
    pub fn basename_raw(&self) -> &CStr {
        unsafe {
            let ptr = sys::zathura_document_get_basename(self.ptr);
            assert!(
                !ptr.is_null(),
                "`zathura_document_get_basename` returned NULL"
            );

            CStr::from_ptr(ptr)
        }
    }

    /// Returns the UTF-8 basename of the document's path.
    ///
    /// If the document was loaded from a URI, this will return the URI's
    /// basename. Otherwise, this will return the basename of the local file
    /// path.
    ///
    /// If the basename is not valid UTF-8, a `Utf8Error` will be returned.
    pub fn basename_utf8(&self) -> Result<&str, Utf8Error> {
        self.basename_raw().to_str()
    }

    /// Returns the zoom level of the document.
    ///
    /// The zoom level is exclusively user-controlled and does not take PPI of
    /// the screen or target surface into account.
    pub fn zoom(&self) -> f64 {
        unsafe { sys::zathura_document_get_zoom(self.ptr) }
    }

    /// Returns the render scale in device pixels per point.
    ///
    /// This takes the zoom level and device PPI into account. Note that even
    /// with a zoom of `1.0` and on a non-HiDPI screen this will not return
    /// `1.0`: Zathura assumes that documents use 72 points per inch, while
    /// non-HiDPI monitors usually have a resolution of 90-100 pixels per inch.
    pub fn scale(&self) -> f64 {
        unsafe { sys::zathura_document_get_scale(self.ptr) }
    }

    /// Returns the viewport rotation in degrees.
    ///
    /// Zathura only supports rotation by a multiple of 90°. Thus this value is
    /// limited to the range 0-270 in multiples of 90 (ie. it can be 0, 90,
    /// 180 or 270).
    pub fn rotation(&self) -> u32 {
        unsafe { sys::zathura_document_get_rotation(self.ptr) }
    }

    /// Returns the viewport's Pixels Per Inch.
    ///
    /// This value might be 0.0 if no PPI information is available.
    pub fn viewport_ppi(&self) -> f64 {
        unsafe { sys::zathura_document_get_viewport_ppi(self.ptr) }
    }

    /// Returns the device scaling factors to map from window coordinates to
    /// pixel coordinates.
    ///
    /// Currently, Zathura uses `gtk_widget_get_scale_factor` for this, so this
    /// will return integer values only, and doesn't distinguish between X and Y
    /// axis, so both returned values will be the same.
    ///
    /// On a "normal" screen, this will return `(1.0, 1.0)`, while on a HiDPI
    /// screen it will return higher values (eg. `(2.0, 2.0)`). The returned
    /// values will never be `0.0`.
    pub fn scaling_factors(&self) -> (f64, f64) {
        unsafe {
            let factors = sys::zathura_document_get_device_factors(self.ptr);
            (factors.x, factors.y)
        }
    }

    /// Returns the size of a page cell in the document (in device pixels).
    ///
    /// Since pages can vary in size, this will return the largest cell size
    /// needed to fit all pages.
    ///
    /// This takes scaling and rotation into account.
    pub fn cell_size(&self) -> (u32, u32) {
        unsafe {
            let (mut height, mut width) = (0, 0);
            sys::zathura_document_get_cell_size(self.ptr, &mut height, &mut width);
            (width, height)
        }
    }

    /// Returns the number of pages in this document.
    pub fn page_count(&self) -> u32 {
        unsafe { sys::zathura_document_get_number_of_pages(self.ptr) as u32 }
    }

    /// Obtains a mutable reference to the page at `index`.
    ///
    /// Returns `None` when `index` is larger than or equal to the number of
    /// pages in this document.
    pub fn page<'a>(&'a mut self, index: usize) -> Option<PageRef<'a>> {
        unsafe {
            let ptr = sys::zathura_document_get_page(self.ptr, index as _);
            if ptr.is_null() {
                None
            } else {
                Some(PageRef::from_raw(ptr))
            }
        }
    }

    /// Returns the currently focused page index.
    ///
    /// Multiple pages can be displayed at the same time, in both X and Y
    /// direction. The returned index usually belongs to the "most-visible"
    /// page in the viewport.
    pub fn current_page_index(&self) -> u32 {
        unsafe { sys::zathura_document_get_current_page_number(self.ptr) as u32 }
    }

    /// Unsafely sets the page count in the document to another value.
    ///
    /// This will not allocate any pages.
    ///
    /// # Safety
    ///
    /// This enables UB via dangling pointer deref through methods like `page`.
    /// It should only be used when the situation will be corrected immediately
    /// (eg. by allocating the right number of pages).
    pub unsafe fn set_page_count(&mut self, count: u32) {
        sys::zathura_document_set_number_of_pages(self.ptr, count as _)
    }

    /// Returns the plugin-controlled pointer associated with the document.
    ///
    /// This is mostly for internal use by this library and is usually unsafe
    /// to dereference.
    pub fn plugin_data(&self) -> *mut () {
        unsafe { sys::zathura_document_get_data(self.ptr) as *mut () }
    }

    /// Sets the custom plugin data pointer to `data`.
    ///
    /// # Safety
    ///
    /// This method is unsafe and should not be used by plugins. Instead, the
    /// `ZathuraPlugin` trait already provides an associated `DocumentData`
    /// type, which can be used instead.
    ///
    /// This library will assume that the plugin data is a
    /// `*mut Plugin::DocumentData` obtained using `Box::into_raw`, and will
    /// free the data automatically.
    pub unsafe fn set_plugin_data(&mut self, data: *mut ()) {
        sys::zathura_document_set_data(self.ptr, data as *mut _)
    }
}
