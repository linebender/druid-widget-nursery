// TODO: survive a change in AppData.

use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

mod widget;

use druid::{Data, Env, ExtEventSink, PlatformError, Selector, Target};
use hot_reload_lib::HotReloadLib;
use widget::HotReloaderWidget;

mod hot_reload_lib;

const RELOAD: Selector<()> = Selector::new("druid-hot-reload.reload");

pub struct AppLauncher<T> {
    inner: druid::AppLauncher<T>,
}

// TODO: add more window customization
pub struct WindowDesc<T> {
    lib_path: &'static str,
    view: &'static str,
    _ty: PhantomData<*const T>,
}

impl<T: Data> WindowDesc<T> {
    pub fn new(lib_path: &'static str, view: &'static str) -> Self {
        Self {
            lib_path,
            view,
            _ty: PhantomData,
        }
    }

    fn build(self, sink: Arc<Mutex<Option<ExtEventSink>>>) -> druid::WindowDesc<T> {
        let lib_path = self.lib_path;
        let view_fn_name = self.view;
        let hot_reloader_widget = HotReloaderWidget {
            lib: HotReloadLib::new(lib_path, move || {
                let sink = sink.lock().unwrap();
                let sink = sink.as_ref().unwrap();
                sink.submit_command(RELOAD, (), Target::Global).unwrap();
            }),
            inner: None,
            view_fn_name,
        };
        druid::WindowDesc::new(hot_reloader_widget)
    }
}

impl<T: Data> AppLauncher<T> {
    /// Create a new `AppLauncher` with the provided window.
    pub fn with_window(window: WindowDesc<T>) -> Self {
        let sink = Arc::default();
        let window = window.build(Arc::clone(&sink));
        let inner = druid::AppLauncher::with_window(window);

        let mut sink = sink.lock().unwrap();
        *sink = Some(inner.get_external_handle());

        Self { inner }
    }

    /// Provide an optional closure that will be given mutable access to
    /// the environment and immutable access to the app state before launch.
    ///
    /// This can be used to set or override theme values.
    pub fn configure_env(mut self, f: impl Fn(&mut Env, &T) + 'static) -> Self {
        self.inner = self.inner.configure_env(f);
        self
    }

    // TODO: add delegate
    // /// Set the [`AppDelegate`].
    // ///
    // /// [`AppDelegate`]: trait.AppDelegate.html
    // pub fn delegate(mut self, delegate: impl AppDelegate<T> + 'static) -> Self {
    //     self.inner = self.inner.delegate(delegate);
    //     self
    // }

    /// Initialize a minimal logger for printing logs out to stderr.
    ///
    /// Meant for use during development only.
    ///
    /// # Panics
    ///
    /// Panics if the logger fails to initialize.
    pub fn log_to_console(mut self) -> Self {
        self.inner = self.inner.log_to_console();
        self
    }

    /// Returns an [`ExtEventSink`] that can be moved between threads,
    /// and can be used to submit commands back to the application.
    ///
    /// [`ExtEventSink`]: struct.ExtEventSink.html
    pub fn get_external_handle(&self) -> ExtEventSink {
        self.inner.get_external_handle()
    }

    /// Build the windows and start the runloop.
    ///
    /// Returns an error if a window cannot be instantiated. This is usually
    /// a fatal error.
    ///
    /// ## Safety
    /// This function can cause UB. don't use this except for development.
    pub unsafe fn launch(self, data: T) -> Result<(), PlatformError> {
        self.inner.launch(data)
    }
}
