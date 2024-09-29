use std::{
	env,
	path::{Path, PathBuf},
	sync::LazyLock,
};

use fs_err as fs;

pub const DUMP_DIR: &str = "lua_dumps";
pub const LOG_DIR: &str = "logs";
pub const INCLUDE_DIR: &str = "scripts";
pub const PLUGIN_DIR: &str = "plugins";
pub const BIN_DIR: &str = "bin";

pub const AUTORUN_PATH: &str = "autorun.lua";
pub const HOOK_PATH: &str = "hook.lua";
pub const SETTINGS_PATH: &str = "settings.toml";

mod path;
pub use path::FSPath;

static HOME_DIR: LazyLock<PathBuf> = LazyLock::new(|| unsafe {
	let kernel32 = libloading::Library::new("kernel32").expect("Couldn't load kernel32 module!");

	if kernel32
		.get::<usize>(c"wine_get_unix_file_name".to_bytes_with_nul())
		.is_ok()
	{
		PathBuf::from("/home")
			.join(env::var("USER").expect("Failed to get $USER environment variable!"))
	} else {
		home::home_dir().expect("Couldn't get your home directory!")
	}
});

pub fn home_dir() -> &'static PathBuf {
	unsafe { &HOME_DIR }
}

pub fn in_autorun<S: AsRef<Path>>(path: S) -> PathBuf {
	home_dir().join("autorun").join(path.as_ref())
}

pub fn base() -> PathBuf {
	home_dir().join("autorun")
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
	use std::io::Read;

	let mut file = fs::File::open(in_autorun(path.as_ref()))?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;

	Ok(contents)
}

// Reads a directory at a path local to the 'autorun' directory,
// And then returns results *also* truncated to be local to the 'autorun' directory
pub fn traverse_dir<P: AsRef<Path>, F: FnMut(&FSPath, fs::DirEntry)>(
	path: P,
	mut rt: F,
) -> std::io::Result<()> {
	let p = in_autorun(path.as_ref());
	let ar_base = base();

	for entry in fs::read_dir(&p)?.flatten() {
		let path = entry.path();
		let path = path.strip_prefix(&ar_base).unwrap_or(&path);

		rt(&FSPath::from(path), entry);
	}

	Ok(())
}

pub fn create_dir(path: &FSPath) -> std::io::Result<()> {
	fs::create_dir(in_autorun(path))
}

pub fn create_file(path: &FSPath) -> std::io::Result<fs::File> {
	fs::File::create(in_autorun(path))
}

pub fn remove_dir(path: &FSPath) -> std::io::Result<()> {
	fs::remove_dir_all(in_autorun(path))
}
