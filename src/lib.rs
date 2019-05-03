//! A Rust wrapper around Zathura's plugin API, allowing plugin development in
//! Rust.
//!
//! This library wraps the plugin interface and exposes the [`ZathuraPlugin`]
//! trait as the primary way to implement a Rust plugin for Zathura.
//!
//! # Examples
//!
//! ```
//! # use zathura_plugin::*;
//! struct PluginType {}
//!
//! impl ZathuraPlugin for PluginType {
//!     type DocumentData = ();
//!     type PageData = ();
//!
//!     fn document_open(doc: DocumentRef<'_>) -> Result<DocumentInfo<Self>, PluginError> {
//!         unimplemented!()
//!     }
//!
//!     fn page_init(page: PageRef<'_>, doc_data: &mut ()) -> Result<PageInfo<Self>, PluginError> {
//!         unimplemented!()
//!     }
//!
//!     fn page_render(
//!         page: PageRef<'_>,
//!         doc_data: &mut Self::DocumentData,
//!         page_data: &mut Self::PageData,
//!         cairo: &mut cairo::Context,
//!         printing: bool,
//!     ) -> Result<(), PluginError> {
//!         unimplemented!()
//!     }
//! }
//!
//! plugin_entry!("MyPlugin", PluginType, ["text/plain", "application/pdf"]);
//! ```
//!
//! [`ZathuraPlugin`]: trait.ZathuraPlugin.html

#![doc(html_root_url = "https://docs.rs/zathura-plugin/0.2.1")]
#![warn(missing_debug_implementations, rust_2018_idioms)]

mod document;
mod error;
mod page;

pub use {
    self::{document::*, error::*, page::*},
    zathura_plugin_sys as sys,
};

/// Information needed to configure a Zathura document.
#[derive(Debug)]
pub struct DocumentInfo<P: ZathuraPlugin + ?Sized> {
    /// Number of pages to create in the document.
    pub page_count: u32,

    /// Plugin-specific data to attach to the document.
    pub plugin_data: P::DocumentData,
}

/// Information needed to configure a document page.
#[derive(Debug)]
pub struct PageInfo<P: ZathuraPlugin + ?Sized> {
    pub width: f64,
    pub height: f64,
    pub plugin_data: P::PageData,
}

/// Trait to be implemented by Zathura plugins.
pub trait ZathuraPlugin {
    /// Plugin-specific data attached to Zathura documents.
    ///
    /// If the plugin doesn't need to associate custom data with the document,
    /// this can be set to `()`.
    type DocumentData;

    /// Plugin-specific data attached to every document page.
    ///
    /// If the plugin doesn't need to associate custom data with every page,
    /// this can be set to `()`.
    type PageData;

    /// Open a document and read its metadata.
    ///
    /// This function has to determine and return the number of pages in the
    /// document. Zathura will create that number of pages and call the plugin's
    /// page initialization and rendering methods.
    fn document_open(doc: DocumentRef<'_>) -> Result<DocumentInfo<Self>, PluginError>;

    /// Additional hook called before freeing the document resources.
    ///
    /// It is not necessary for plugins to implement this. The library will take
    /// care of freeing the `DocumentData` attached to the document, and Zathura
    /// itself will free the actual document.
    fn document_free(
        doc: DocumentRef<'_>,
        doc_data: &mut Self::DocumentData,
    ) -> Result<(), PluginError> {
        let _ = (doc, doc_data);
        Ok(())
    }

    /// Initialize a document page and obtain its properties.
    ///
    /// This is called once per page when the document is loaded initially.
    ///
    /// The plugin has to return a `PageInfo` structure containing page
    /// properties that will be applied by the library.
    fn page_init(
        page: PageRef<'_>,
        doc_data: &mut Self::DocumentData,
    ) -> Result<PageInfo<Self>, PluginError>;

    /// Additional hook called before freeing page resources.
    ///
    /// This doesn't have to be implemented by a plugin. The library already
    /// takes care of freeing the `PageData` associated with the page, and
    /// Zathura will free the page itself.
    fn page_free(
        page: PageRef<'_>,
        doc_data: &mut Self::DocumentData,
        page_data: &mut Self::PageData,
    ) -> Result<(), PluginError> {
        let _ = (page, doc_data, page_data);
        Ok(())
    }

    /// Render a document page to a Cairo context.
    ///
    /// # Parameters
    ///
    /// * **`page`**: Mutable reference to the page to render.
    /// * **`doc_data`**: Plugin-specific data attached to the document.
    /// * **`page_data`**: Plugin-specific data attached to the page.
    /// * **`cairo`**: The Cairo context to render to.
    /// * **`printing`**: Whether the page is being rendered for printing
    ///   (`true`) or viewing (`false`).
    fn page_render(
        page: PageRef<'_>,
        doc_data: &mut Self::DocumentData,
        page_data: &mut Self::PageData,
        cairo: &mut cairo::Context,
        printing: bool,
    ) -> Result<(), PluginError>;
}

/// `extern "C"` functions wrapping the Rust `ZathuraPlugin` functions.
///
/// This is not public API and is only intended to be used by the
/// `plugin_entry!` macro.
#[doc(hidden)]
pub mod wrapper {
    use {
        crate::{sys::*, *},
        cairo,
        std::{
            ffi::c_void,
            panic::{catch_unwind, AssertUnwindSafe},
        },
    };

    trait ResultExt {
        fn to_zathura(self) -> zathura_error_t;
    }

    impl ResultExt for Result<(), PluginError> {
        fn to_zathura(self) -> zathura_error_t {
            match self {
                Ok(()) => 0,
                Err(e) => e as zathura_error_t,
            }
        }
    }

    fn wrap(f: impl FnOnce() -> Result<(), PluginError>) -> Result<(), PluginError> {
        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(r) => r,
            Err(_) => Err(PluginError::Unknown),
        }
    }

    /// Open a document and set the number of pages to create in `document`.
    pub unsafe extern "C" fn document_open<P: ZathuraPlugin>(
        document: *mut zathura_document_t,
    ) -> zathura_error_t {
        wrap(|| {
            let doc = DocumentRef::from_raw(document);
            let info = P::document_open(doc)?;
            let mut doc = DocumentRef::from_raw(document);
            doc.set_plugin_data(Box::into_raw(Box::new(info.plugin_data)) as *mut _);
            doc.set_page_count(info.page_count);
            Ok(())
        })
        .to_zathura()
    }

    /// Free plugin-specific data in `document`.
    ///
    /// This is called by `zathura_document_free` and thus must not attempt to
    /// free the document again.
    pub unsafe extern "C" fn document_free<P: ZathuraPlugin>(
        document: *mut zathura_document_t,
        _data: *mut c_void,
    ) -> zathura_error_t {
        wrap(|| {
            let doc = DocumentRef::from_raw(document);
            let doc_data = &mut *(doc.plugin_data() as *mut P::DocumentData);
            let result = P::document_free(doc, doc_data);
            let doc = DocumentRef::from_raw(document);
            let plugin_data = doc.plugin_data() as *mut P::DocumentData;
            drop(Box::from_raw(plugin_data));
            result
        })
        .to_zathura()
    }

    /// Initialize a page and set its dimensions.
    ///
    /// If the page size is not set, rendering on it has no effect and the page
    /// appears invisible.
    pub unsafe extern "C" fn page_init<P: ZathuraPlugin>(
        page: *mut zathura_page_t,
    ) -> zathura_error_t {
        wrap(|| {
            let mut p = PageRef::from_raw(page);

            // Obtaining the document data is safe, since there is no other way to get access to it
            // while this function executes.
            let doc_data = p.document().plugin_data() as *mut P::DocumentData;

            let info = P::page_init(p, &mut *doc_data)?;
            let mut p = PageRef::from_raw(page);
            p.set_width(info.width);
            p.set_height(info.height);
            p.set_plugin_data(Box::into_raw(Box::new(info.plugin_data)) as *mut _);
            Ok(())
        })
        .to_zathura()
    }

    /// Deallocate plugin-specific page data.
    ///
    /// If this function is missing, the *document* will not be freed.
    pub unsafe extern "C" fn page_clear<P: ZathuraPlugin>(
        page: *mut zathura_page_t,
        _data: *mut c_void,
    ) -> zathura_error_t {
        wrap(|| {
            let result = {
                let mut p = PageRef::from_raw(page);
                let doc_data = &mut *(p.document().plugin_data() as *mut P::DocumentData);
                let page_data = &mut *(p.plugin_data() as *mut P::PageData);
                P::page_free(p, doc_data, page_data)
            };

            // Free the `PageData`
            let p = PageRef::from_raw(page);
            let plugin_data = p.plugin_data() as *mut P::PageData;
            drop(Box::from_raw(plugin_data));

            result
        })
        .to_zathura()
    }

    /// Render a page to a Cairo context.
    pub unsafe extern "C" fn page_render_cairo<P: ZathuraPlugin>(
        page: *mut zathura_page_t,
        _data: *mut c_void,
        cairo: *mut sys::cairo_t,
        printing: bool,
    ) -> zathura_error_t {
        wrap(|| {
            let mut p = PageRef::from_raw(page);
            let page_data = &mut *(p.plugin_data() as *mut P::PageData);
            let doc_data = &mut *(p.document().plugin_data() as *mut P::DocumentData);
            let mut cairo = cairo::Context::from_raw_borrow(cairo as *mut _);
            P::page_render(p, doc_data, page_data, &mut cairo, printing)
        })
        .to_zathura()
    }
}

/// Declares this library as a Zathura plugin.
///
/// A crate can only provide one Zathura plugin, so this macro may only be
/// called once per crate.
///
/// For this to work, this crate must be built as a `cdylib` and the result put
/// somewhere Zathura can find it. An easy way to iterate on a plugin is running
/// this in the workspace root after any changes:
///
/// ```notrust
/// cargo build && zathura -p target/debug/ <file>
/// ```
///
/// # Examples
///
/// For a usage example of this macro, refer to the crate-level docs.
#[macro_export]
macro_rules! plugin_entry {
    (
        $name:literal,
        $plugin_ty:ty,
        [
            $($mime:literal),+
            $(,)?
        ]
    ) => {
        #[doc(hidden)]
        #[repr(transparent)]
        #[allow(warnings)]
        pub struct __AssertSync<T>(T);

        unsafe impl<T> Sync for __AssertSync<T> {}

        #[doc(hidden)]
        #[no_mangle]
        pub static mut zathura_plugin_3_4: /* API=3, ABI=4 */
        __AssertSync<$crate::sys::zathura_plugin_definition_t> = __AssertSync({
            use $crate::sys::*;
            use $crate::wrapper::*;

            zathura_plugin_definition_t {
                name: concat!($name, "\0").as_ptr() as *const _,
                version: zathura_plugin_version_t {
                    // TODO: use the Cargo env vars to determine the version
                    major: 0,
                    minor: 0,
                    rev: 0,
                },
                mime_types_size: {
                    // Sum up as many 1s as there are entries in `$mime`. The
                    // `$mime;` tells the compiler which syntax variable to
                    // iterate over; it is disposed with no effect.
                    0 $(+ {
                        $mime;
                        1
                    })+
                },
                mime_types: [
                    $(
                        concat!($mime, "\0").as_ptr() as *const _,
                    )+
                ].as_ptr() as *mut _, // assuming Zathura never mutates this
                functions: zathura_plugin_functions_t {
                    document_open: Some(document_open::<$plugin_ty>),
                    document_free: Some(document_free::<$plugin_ty>),
                    document_index_generate: None,
                    document_save_as: None,
                    document_attachments_get: None,
                    document_attachment_save: None,
                    document_get_information: None,
                    page_init: Some(page_init::<$plugin_ty>),
                    page_clear: Some(page_clear::<$plugin_ty>),
                    page_search_text: None,
                    page_links_get: None,
                    page_form_fields_get: None,
                    page_images_get: None,
                    page_image_get_cairo: None,
                    page_get_text: None,
                    page_render: None, // no longer used?
                    page_render_cairo: Some(page_render_cairo::<$plugin_ty>),
                    page_get_label: None,
                },
            }
        });
    };
}

#[cfg(feature = "testplugin")]
struct TestPlugin;

#[cfg(feature = "testplugin")]
impl ZathuraPlugin for TestPlugin {
    type DocumentData = ();
    type PageData = ();

    fn document_open(doc: DocumentRef<'_>) -> Result<DocumentInfo<Self>, PluginError> {
        println!("open: {:?}", doc.basename());
        println!("path: {:?}", doc.path());
        println!("url:  {:?}", doc.uri());
        println!("{} pages", doc.page_count());
        Ok(DocumentInfo {
            page_count: 5,
            plugin_data: (),
        })
    }

    fn document_free(doc: DocumentRef<'_>) -> Result<(), PluginError> {
        println!("free! {:?}", doc);
        Ok(())
    }

    fn page_init(page: PageRef<'_>, _doc_data: &mut ()) -> Result<PageInfo<Self>, PluginError> {
        println!("page init: {:?}", page);

        Ok(PageInfo {
            width: 75.0,
            height: 100.0,
            plugin_data: (),
        })
    }

    fn page_free(page: PageRef<'_>) -> Result<(), PluginError> {
        println!("page free: {:?}", page);
        Ok(())
    }

    fn page_render(
        mut page: PageRef<'_>,
        _doc_data: &mut Self::DocumentData,
        _page_data: &mut Self::PageData,
        cairo: &mut cairo::Context,
        printing: bool,
    ) -> Result<(), PluginError> {
        println!(
            "render! {:?}, index={:?}, {}x{}, {:?}",
            page,
            page.index(),
            page.width(),
            page.height(),
            printing
        );

        {
            let doc = page.document();
            println!(
                "doc: zoom={}, scale={}, rotation={}°, ppi={}, scale={:?}, cell-size={:?}",
                doc.zoom(),
                doc.scale(),
                doc.rotation(),
                doc.viewport_ppi(),
                doc.scaling_factors(),
                doc.cell_size(),
            );
        }

        println!(
            "cairo: scale={:?}, 50,50={:?}",
            cairo.get_target().get_device_scale(),
            cairo.user_to_device(50.0, 50.0),
        );

        if page.index() == 0 {
            cairo.move_to(10.0, 10.0);
            cairo.show_text("Wello!");
            cairo.set_source_rgb(0.0, 1.0, 1.0);
            cairo.set_line_width(1.0);
            cairo.move_to(0.0, 0.0);
            cairo.line_to(10.5, 50.5);
            cairo.stroke();
        }
        Ok(())
    }
}

#[cfg(feature = "testplugin")]
plugin_entry!("TestPlugin", TestPlugin, ["text/plain"]);
