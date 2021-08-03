use libloading::{Library, Symbol};
use notify5::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};

pub struct HotReloadLib {
    lib_path: PathBuf,
    temp_path: PathBuf,
    library: Option<Library>,
    _watcher: RecommendedWatcher,
}

impl HotReloadLib {
    pub fn new(path: impl AsRef<Path>, on_reload: impl Fn() + Send + Sync + 'static) -> Self {
        let lib_path = path.as_ref().canonicalize().unwrap();
        let (lib, temp_path) = copy_and_load_library(&lib_path);
        let mut watcher = notify5::recommended_watcher({
            let lib_path = lib_path.clone();
            move |ev| {
                let ev: Event = match ev {
                    Ok(ev) => ev,
                    _ => return,
                };

                if ev.paths.contains(&lib_path) && ev.kind.is_create() {
                    on_reload();
                }
            }
        })
        .unwrap();
        watcher
            .watch(lib_path.parent().unwrap(), RecursiveMode::NonRecursive)
            .unwrap();

        HotReloadLib {
            temp_path,
            lib_path,
            library: Some(lib),
            _watcher: watcher,
        }
    }

    pub unsafe fn load_symbol<Signature>(&self, symbol_name: &str) -> Option<Symbol<Signature>> {
        let lib = self.library.as_ref().unwrap();
        lib.get(symbol_name.as_bytes()).ok()
    }

    pub fn update(&mut self) {
        self.library = None;
        fs::remove_file(&self.temp_path).unwrap();
        let (library, path) = copy_and_load_library(&self.lib_path);
        self.library = Some(library);
        self.temp_path = path;
    }
}

impl Drop for HotReloadLib {
    fn drop(&mut self) {
        fs::remove_file(&self.temp_path).unwrap();
    }
}

fn copy_and_load_library(lib_path: &Path) -> (Library, PathBuf) {
    let unique_path = {
        let mut path = std::env::temp_dir();
        path.push(rand::random::<u64>().to_string());
        path.set_extension(lib_path.extension().unwrap());
        path
    };
    fs::copy(&lib_path, &unique_path).expect("Failed to copy lib to unique path");
    let lib = Library::new(unique_path.as_os_str()).expect("Failed to load library");

    (lib, unique_path)
}
