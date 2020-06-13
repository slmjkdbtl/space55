// wengwengweng

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use crate::*;

const VSPACE: f32 = 5.0;
const HSPACE: f32 = 6.0;
const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = FONT_SIZE + VSPACE * 2.0;

#[derive(Clone)]
pub struct FileBrowser {
	cur_path: PathBuf,
	entries: Vec<PathBuf>,
	cursor: Cursor,
	hide_hidden: bool,
	scroll_off: f32,
	scroll_remainder: f32,
	view_size: Option<(f32, f32)>,
}

#[derive(Clone, Copy, Debug)]
enum Cursor {
	Up,
	Entry(usize),
}

fn is_hidden(path: impl AsRef<Path>) -> bool {

	let path = path.as_ref();

	if let Some(fname) = path.file_name().and_then(OsStr::to_str) {
		return fname.chars().next() == Some('.');
	}

	return false;

}

impl FileBrowser {

	pub fn new(path: impl AsRef<Path>) -> Result<Self> {

		let path = path.as_ref();

		let mut fbrowse = Self {
			cur_path: path.to_owned(),
			entries: vec![],
			cursor: Cursor::Up,
			hide_hidden: true,
			scroll_off: 0.0,
			scroll_remainder: 0.0,
			view_size: None,
		};

		fbrowse.cd(path);

		return Ok(fbrowse);

	}

	pub fn path(&self) -> &PathBuf {
		return &self.cur_path;
	}

	pub fn entries(&self) -> &[PathBuf] {
		return &self.entries;
	}

	pub fn set_view_size(&mut self, w: f32, h: f32) {
		self.view_size = Some((w, h));
	}

	pub fn cd(&mut self, path: impl AsRef<Path>) {

		let path = path.as_ref();

		let mut dirs = vec![];
		let mut files = vec![];

		if let Ok(entries) = path.read_dir() {

			for e in entries {

				if let Ok(e) = e {

					let path = e.path();

					if self.hide_hidden {
						if is_hidden(&path) {
							continue;
						}
					}

					if path.is_dir() {
						dirs.push(path);
					} else {
						files.push(path);
					}

				}

			}

		}

		dirs.sort();
		files.sort();

		dirs.append(&mut files);

		self.cur_path = path.to_owned();
		self.entries = dirs;
		self.scroll_off = 0.0;

		self.cursor = if self.entries.is_empty() {
			Cursor::Up
		} else {
			Cursor::Entry(0)
		};

	}

	pub fn move_up(&mut self) {
		match self.cursor {
			Cursor::Entry(i) => {
				if i == 0 {
					self.cursor = Cursor::Up;
				} else {
					self.cursor = Cursor::Entry(i - 1);
				}
			},
			_ => {},
		}
	}

	pub fn move_down(&mut self) {
		match self.cursor {
			Cursor::Entry(i) => {
				if i < self.entries.len() - 1 {
					self.cursor = Cursor::Entry(i + 1);
				}
			},
			Cursor::Up => {
				if !self.entries.is_empty() {
					self.cursor = Cursor::Entry(0);
				}
			},
		}
	}

	pub fn back(&mut self) {

		let path = self.cur_path.clone();
		let success = self.cur_path.pop();

		if success {
			self.cd(&self.cur_path.clone());
		}

		if let Some(i) = self.entries.iter().position(|p| p == &path) {
			self.cursor = Cursor::Entry(i);
		}

	}

	pub fn enter(&mut self) -> Option<PathBuf> {

		match self.cursor {

			Cursor::Up => {
				self.back();
			},

			Cursor::Entry(i) => {
				if let Some(e) = self.entries.get(i) {
					if e.is_dir() {
						self.cd(e.clone());
					} else {
						return Some(e.clone());
					}
				}
			},

		}

		return None;

	}

	pub fn event(&mut self, _: &mut Ctx, e: &input::Event) -> Result<()> {

		use input::Event::*;

		match e {

			KeyPress(k) => {
				match *k {
					Key::Backspace => self.back(),
					_ => {},
				}
			},

			KeyPressRepeat(k) => {
				match *k {
					Key::J => self.move_down(),
					Key::K => self.move_up(),
					_ => {},
				}
			},

			Wheel(d, _) => {

				let y = d.y * 0.1;
				self.scroll_remainder = (y + self.scroll_remainder) % 1.0;
				let y = (y + self.scroll_remainder) as i32;

				for _ in 0..y.abs() {
					if y > 0 {
						self.move_down();
					} else if y < 0 {
						self.move_up();
					}
				}

			},

			_ => {},

		}

		return Ok(());

	}

	pub fn update(&mut self, d: &mut Ctx) -> Result<()> {

		// scrolling
		let (vw, vh) = self.view_size.unwrap_or((d.gfx.width() as f32, d.gfx.height() as f32));

		let height = LINE_HEIGHT * match self.cursor {
			Cursor::Up => 1.0,
			Cursor::Entry(i) => i as f32 + 2.0,
		};

		let y = height - self.scroll_off;

		if y > vh {
			self.scroll_off = height - vh;
		}

		if self.scroll_off > (height - LINE_HEIGHT) {
			self.scroll_off = height - LINE_HEIGHT;
		}

		return Ok(());

	}

	// TODO: only render visible parts
	pub fn draw(&self, gfx: &mut Gfx) -> Result<()> {

		let (vw, vh) = self.view_size.unwrap_or((gfx.width() as f32, gfx.height() as f32));
		let l1 = f32::floor(self.scroll_off / LINE_HEIGHT) as i32;
		let l2 = f32::ceil((self.scroll_off + vh) / LINE_HEIGHT) as i32;

		let cpos = match self.cursor {
			Cursor::Up => 0,
			Cursor::Entry(i) => i + 1,
		};

		gfx.push_t(mat4!().ty(self.scroll_off), |gfx| {

			// cursor
			gfx.draw(
				&shapes::rect(
					vec2!(0, cpos as f32 * -LINE_HEIGHT),
					vec2!(gfx.width(), (cpos + 1) as f32 * -LINE_HEIGHT),
				)
					.fill(rgba!(1, 1, 1, 0.2))
			)?;

			// up
			gfx.draw_t(
				mat4!()
					.t2(vec2!(HSPACE, -VSPACE))
					,
				&shapes::text("..")
					.size(FONT_SIZE)
					.align(gfx::Origin::TopLeft)
					.color(rgba!(1, 1, 0, 1))
					,
			)?;

			// entries
			for (i, e) in self.entries().iter().enumerate() {

				let color = if e.is_dir() {
					rgba!(0, 1, 1, 1)
				} else {
					rgba!(1, 1, 1, 1)
				};

				let suffix = if e.is_dir() {
					"/"
				} else {
					""
				};

				if let Some(fname) = e.file_name().and_then(OsStr::to_str) {
					gfx.draw_t(
						mat4!()
							.t2(vec2!(HSPACE, (i + 1) as f32 * -LINE_HEIGHT - VSPACE))
							,
						&shapes::text(&format!("{}{}", fname, suffix))
							.size(FONT_SIZE)
							.align(gfx::Origin::TopLeft)
							.color(color)
							,
					)?;
				}

			}

			return Ok(());

		})?;

		return Ok(());

	}

}

