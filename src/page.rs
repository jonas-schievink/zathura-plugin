use {
    crate::{sys, DocumentRef},
    std::marker::PhantomData,
};

/// Reference to a page in a document.
///
/// This does not expose the `get/set_visibility` functions because they return
/// wildly inconsistent data between runs and even within the same Zathura
/// instance. Zathura only calls the render callback on visible pages, so this
/// API serves little purpose anyways.
#[derive(Debug)]
pub struct PageRef<'a> {
    ptr: *mut sys::zathura_page_t,
    _p: PhantomData<&'a mut ()>,
}

impl PageRef<'_> {
    /// Creates a page reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// This method requires that `ptr` points to a valid `zathura_page_t`. The
    /// returned `PageRef` also has a dangling lifetime, which must be
    /// constrained by the caller so that it doesn't outlive the page or
    /// document.
    ///
    /// Only a single `PageRef` may be exist for the same page at any given
    /// time, since it is effectively a mutable reference. While a `PageRef`
    /// exists, no independent `DocumentRef`s to the document containing the
    /// page may exist.
    pub unsafe fn from_raw(ptr: *mut sys::zathura_page_t) -> Self {
        Self {
            ptr,
            _p: PhantomData,
        }
    }

    /// Obtains a reference to the Zathura document containing this page.
    ///
    /// The document reference is mutable, so it will borrow `self` until it
    /// goes out of scope.
    pub fn document<'a>(&'a mut self) -> DocumentRef<'a> {
        unsafe { DocumentRef::from_raw(sys::zathura_page_get_document(self.ptr)) }
    }

    /// Returns the index of this page in the document (zero-based).
    pub fn index(&self) -> usize {
        unsafe { sys::zathura_page_get_index(self.ptr) as usize }
    }

    pub fn width(&self) -> f64 {
        unsafe { sys::zathura_page_get_width(self.ptr) }
    }

    pub fn set_width(&mut self, width: f64) {
        unsafe { sys::zathura_page_set_width(self.ptr, width) }
    }

    pub fn height(&self) -> f64 {
        unsafe { sys::zathura_page_get_height(self.ptr) }
    }

    pub fn set_height(&mut self, height: f64) {
        unsafe { sys::zathura_page_set_height(self.ptr, height) }
    }

    pub fn plugin_data(&self) -> *mut () {
        unsafe { sys::zathura_page_get_data(self.ptr) as *mut () }
    }

    /// Sets the custom plugin data pointer to `data`.
    ///
    /// # Safety
    ///
    /// This method is unsafe and should not be used by plugins. Instead, the
    /// `ZathuraPlugin` trait already provides an associated `PageData` type,
    /// which can be used instead.
    ///
    /// This library will assume that the plugin data is a
    /// `*mut Plugin::PageData` obtained from `Box::into_raw`, and frees the
    /// data automatically.
    pub unsafe fn set_plugin_data(&mut self, data: *mut ()) {
        sys::zathura_page_set_data(self.ptr, data as *mut _)
    }
}
