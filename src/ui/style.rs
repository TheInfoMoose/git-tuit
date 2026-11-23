use crate::ui::syntax_text::DEFAULT_SYNTAX_THEME;
use anyhow::Result;
use asyncgit::{DiffLineType, StatusItemType};
use ratatui::style::{Color, Modifier, Style};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf, rc::Rc};
use struct_patch::Patch;

pub type SharedTheme = Rc<Theme>;

const AMBER_PRIMARY: Color = Color::Rgb(255, 176, 0);
const AMBER_BRIGHT: Color = Color::Rgb(255, 216, 102);
const AMBER_DIM: Color = Color::Rgb(153, 99, 0);
const AMBER_DARK: Color = Color::Rgb(92, 58, 0);
const AMBER_ALERT: Color = Color::Rgb(255, 85, 0);
const PANEL_BG: Color = Color::Rgb(12, 8, 0);
const GREEN_ADD: Color = Color::Rgb(127, 217, 98);
const SELECTION_BG: Color = Color::Rgb(42, 26, 0);

#[derive(Serialize, Deserialize, Debug, Clone, Patch)]
#[serde(default)]
#[patch(attribute(derive(Serialize, Deserialize)))]
pub struct Theme {
	selected_tab: Color,
	command_fg: Color,
	selection_bg: Color,
	selection_fg: Color,
	use_selection_fg: bool,
	cmdbar_bg: Color,
	disabled_fg: Color,
	diff_line_add: Color,
	diff_line_delete: Color,
	diff_file_added: Color,
	diff_file_removed: Color,
	diff_file_moved: Color,
	diff_file_modified: Color,
	commit_hash: Color,
	commit_time: Color,
	commit_author: Color,
	danger_fg: Color,
	push_gauge_bg: Color,
	push_gauge_fg: Color,
	tag_fg: Color,
	branch_fg: Color,
	line_break: String,
	block_title_focused: Color,
	border_focused_fg: Color,
	scroll_track_fg: Color,
	panel_bg: Color,
	attention_fg: Color,
	syntax: String,
}

impl Theme {
	pub fn popup(&self) -> Style {
		Style::default().bg(self.panel_bg)
	}

	pub fn scroll_bar_pos(&self) -> Style {
		Style::default().fg(self.selection_bg)
	}

	pub fn scroll_bar_track(&self) -> Style {
		Style::default().fg(self.scroll_track_fg)
	}

	pub fn block(&self, focus: bool) -> Style {
		if focus {
			Style::default().fg(self.border_focused_fg)
		} else {
			Style::default().fg(self.disabled_fg)
		}
	}

	pub fn title(&self, focused: bool) -> Style {
		if focused {
			Style::default()
				.fg(self.block_title_focused)
				.add_modifier(Modifier::BOLD)
		} else {
			Style::default().fg(self.disabled_fg)
		}
	}

	pub fn branch(&self, selected: bool, head: bool) -> Style {
		let branch = if head {
			Style::default().add_modifier(Modifier::BOLD)
		} else {
			Style::default()
		}
		.fg(self.branch_fg);

		if selected {
			branch.patch(Style::default().bg(self.selection_bg))
		} else {
			branch
		}
	}

	pub fn tab(&self, selected: bool) -> Style {
		if selected {
			self.text(true, false)
				.fg(self.selected_tab)
				.add_modifier(Modifier::UNDERLINED)
		} else {
			self.text(false, false)
		}
	}

	pub fn tags(&self, selected: bool) -> Style {
		Style::default()
			.fg(self.tag_fg)
			.add_modifier(Modifier::BOLD)
			.bg(if selected {
				self.selection_bg
			} else {
				Color::Reset
			})
	}

	pub fn text(&self, enabled: bool, selected: bool) -> Style {
		match (enabled, selected) {
			(false, false) => Style::default().fg(self.disabled_fg),
			(false, true) => Style::default().bg(self.selection_bg),
			(true, false) => Style::default(),
			(true, true) => Style::default()
				.fg(self.command_fg)
				.bg(self.selection_bg),
		}
	}

	pub fn item(&self, typ: StatusItemType, selected: bool) -> Style {
		let style = match typ {
			StatusItemType::New => {
				Style::default().fg(self.diff_file_added)
			}
			StatusItemType::Modified => {
				Style::default().fg(self.diff_file_modified)
			}
			StatusItemType::Deleted => {
				Style::default().fg(self.diff_file_removed)
			}
			StatusItemType::Renamed => {
				Style::default().fg(self.diff_file_moved)
			}
			StatusItemType::Conflicted => Style::default()
				.fg(self.diff_file_modified)
				.add_modifier(Modifier::BOLD),
			StatusItemType::Typechange => Style::default(),
		};

		self.apply_select(style, selected)
	}

	pub fn file_tree_item(
		&self,
		is_folder: bool,
		selected: bool,
	) -> Style {
		let style = if is_folder {
			Style::default()
		} else {
			Style::default().fg(self.diff_file_modified)
		};

		self.apply_select(style, selected)
	}

	const fn apply_select(
		&self,
		style: Style,
		selected: bool,
	) -> Style {
		if selected {
			if self.use_selection_fg {
				style.bg(self.selection_bg).fg(self.selection_fg)
			} else {
				style.bg(self.selection_bg)
			}
		} else {
			style
		}
	}

	pub fn option(&self, on: bool) -> Style {
		if on {
			Style::default().fg(self.diff_line_add)
		} else {
			Style::default().fg(self.diff_line_delete)
		}
	}

	pub fn diff_hunk_marker(&self, selected: bool) -> Style {
		if selected {
			Style::default().bg(self.selection_bg)
		} else {
			Style::default().fg(self.disabled_fg)
		}
	}

	pub fn diff_line(
		&self,
		typ: DiffLineType,
		selected: bool,
	) -> Style {
		let style = match typ {
			DiffLineType::Add => {
				Style::default().fg(self.diff_line_add)
			}
			DiffLineType::Delete => {
				Style::default().fg(self.diff_line_delete)
			}
			DiffLineType::Header => Style::default()
				.fg(self.disabled_fg)
				.add_modifier(Modifier::BOLD),
			DiffLineType::None => Style::default().fg(if selected {
				self.command_fg
			} else {
				Color::Reset
			}),
		};

		self.apply_select(style, selected)
	}

	pub fn text_danger(&self) -> Style {
		Style::default().fg(self.danger_fg)
	}

	pub fn line_break(&self) -> String {
		self.line_break.clone()
	}

	pub fn commandbar(&self, enabled: bool) -> Style {
		if enabled {
			Style::default().fg(self.command_fg)
		} else {
			Style::default().fg(self.disabled_fg)
		}
		.bg(self.cmdbar_bg)
	}

	pub fn commit_hash(&self, selected: bool) -> Style {
		self.apply_select(
			Style::default().fg(self.commit_hash),
			selected,
		)
	}

	pub fn commit_unhighlighted(&self) -> Style {
		Style::default().fg(self.disabled_fg)
	}

	pub fn log_marker(&self, selected: bool) -> Style {
		let mut style = Style::default()
			.fg(self.commit_author)
			.add_modifier(Modifier::BOLD);

		style = self.apply_select(style, selected);

		style
	}

	pub fn commit_time(&self, selected: bool) -> Style {
		self.apply_select(
			Style::default().fg(self.commit_time),
			selected,
		)
	}

	pub fn commit_author(&self, selected: bool) -> Style {
		self.apply_select(
			Style::default().fg(self.commit_author),
			selected,
		)
	}

	pub fn commit_hash_in_blame(
		&self,
		is_blamed_commit: bool,
	) -> Style {
		if is_blamed_commit {
			Style::default()
				.fg(self.commit_hash)
				.add_modifier(Modifier::BOLD)
		} else {
			Style::default().fg(self.commit_hash)
		}
	}

	pub fn push_gauge(&self) -> Style {
		Style::default()
			.fg(self.push_gauge_fg)
			.bg(self.push_gauge_bg)
	}

	pub fn attention_block_style(&self) -> Style {
		Style::default().fg(self.attention_fg)
	}

	fn load_patch(theme_path: &PathBuf) -> Result<ThemePatch> {
		let file = File::open(theme_path)?;

		Ok(ron::de::from_reader(file)?)
	}

	fn load_old_theme(theme_path: &PathBuf) -> Result<Self> {
		let old_file = File::open(theme_path)?;

		Ok(ron::de::from_reader::<File, Self>(old_file)?)
	}

	// This is supposed to be called when theme.ron doesn't already exists.
	fn save_patch(&self, theme_path: &PathBuf) -> Result<()> {
		let mut file = File::create(theme_path)?;
		let patch = self.clone().into_patch_by_diff(Self::default());
		let data = to_string_pretty(&patch, PrettyConfig::default())?;

		file.write_all(data.as_bytes())?;

		Ok(())
	}

	pub fn get_syntax(&self) -> String {
		self.syntax.clone()
	}

	pub fn init(theme_path: &PathBuf) -> Self {
		let mut theme = Self::default();

		if let Ok(patch) = Self::load_patch(theme_path).map_err(|e| {
			log::error!("theme error [{theme_path:?}]: {e}");
			e
		}) {
			theme.apply(patch);
		} else if let Ok(old_theme) = Self::load_old_theme(theme_path)
		{
			theme = old_theme;

			if theme.save_patch(theme_path).is_ok() {
				log::info!("Converted old theme to new format. ({theme_path:?})");
			} else {
				log::warn!("Failed to save theme in new format. ({theme_path:?})");
			}
		}

		theme
	}
}

impl Default for Theme {
	fn default() -> Self {
		Self {
			selected_tab: AMBER_BRIGHT,
			command_fg: AMBER_PRIMARY,
			selection_bg: SELECTION_BG,
			selection_fg: AMBER_BRIGHT,
			use_selection_fg: true,
			cmdbar_bg: PANEL_BG,
			disabled_fg: AMBER_DIM,
			diff_line_add: GREEN_ADD,
			diff_line_delete: AMBER_ALERT,
			diff_file_added: GREEN_ADD,
			diff_file_removed: AMBER_ALERT,
			diff_file_moved: AMBER_BRIGHT,
			diff_file_modified: AMBER_PRIMARY,
			commit_hash: AMBER_BRIGHT,
			commit_time: AMBER_DIM,
			commit_author: AMBER_PRIMARY,
			danger_fg: AMBER_ALERT,
			push_gauge_bg: AMBER_DARK,
			push_gauge_fg: AMBER_BRIGHT,
			tag_fg: AMBER_BRIGHT,
			branch_fg: AMBER_PRIMARY,
			line_break: "¶".to_string(),
			block_title_focused: AMBER_BRIGHT,
			border_focused_fg: AMBER_PRIMARY,
			scroll_track_fg: AMBER_DARK,
			panel_bg: PANEL_BG,
			attention_fg: AMBER_ALERT,
			// Available themes can be found in:
			// [ThemeSet::load_defaults function](https://github.com/trishume/syntect/blob/7fe13c0fd53cdfa0f9fea1aa14c5ba37f81d8b71/src/dumps.rs#L215).
			syntax: DEFAULT_SYNTAX_THEME.to_string(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use pretty_assertions::assert_eq;
	use tempfile::NamedTempFile;

	#[test]
	fn test_smoke() {
		let _ = env_logger::builder()
			.is_test(true)
			.filter_level(log::LevelFilter::Trace)
			.try_init();

		let mut file = NamedTempFile::new().unwrap();

		writeln!(
			file,
			r##"
(
	selection_bg: Some("Black"),
	selection_fg: Some("#ffffff"),
	use_selection_fg: Some(false),
	syntax: Some("InspiredGitHub")
)
"##
		)
		.unwrap();

		let theme = Theme::init(&file.path().to_path_buf());

		assert_eq!(theme.selected_tab, Theme::default().selected_tab);

		assert_ne!(theme.selection_bg, Theme::default().selection_bg);
		assert_ne!(theme.syntax, Theme::default().syntax);
		assert_eq!(theme.selection_bg, Color::Black);
		assert_eq!(theme.selection_fg, Color::Rgb(255, 255, 255));
		assert_eq!(theme.syntax, "InspiredGitHub");
	}

	fn assert_theme_eq(actual: &Theme, expected: &Theme) {
		assert_eq!(actual.selected_tab, expected.selected_tab);
		assert_eq!(actual.command_fg, expected.command_fg);
		assert_eq!(actual.selection_bg, expected.selection_bg);
		assert_eq!(actual.selection_fg, expected.selection_fg);
		assert_eq!(
			actual.use_selection_fg,
			expected.use_selection_fg
		);
		assert_eq!(actual.cmdbar_bg, expected.cmdbar_bg);
		assert_eq!(actual.disabled_fg, expected.disabled_fg);
		assert_eq!(actual.diff_line_add, expected.diff_line_add);
		assert_eq!(
			actual.diff_line_delete,
			expected.diff_line_delete
		);
		assert_eq!(actual.diff_file_added, expected.diff_file_added);
		assert_eq!(
			actual.diff_file_removed,
			expected.diff_file_removed
		);
		assert_eq!(actual.diff_file_moved, expected.diff_file_moved);
		assert_eq!(
			actual.diff_file_modified,
			expected.diff_file_modified
		);
		assert_eq!(actual.commit_hash, expected.commit_hash);
		assert_eq!(actual.commit_time, expected.commit_time);
		assert_eq!(actual.commit_author, expected.commit_author);
		assert_eq!(actual.danger_fg, expected.danger_fg);
		assert_eq!(actual.push_gauge_bg, expected.push_gauge_bg);
		assert_eq!(actual.push_gauge_fg, expected.push_gauge_fg);
		assert_eq!(actual.tag_fg, expected.tag_fg);
		assert_eq!(actual.branch_fg, expected.branch_fg);
		assert_eq!(actual.line_break, expected.line_break);
		assert_eq!(
			actual.block_title_focused,
			expected.block_title_focused
		);
		assert_eq!(actual.syntax, expected.syntax);
	}

	#[test]
	fn t12a_default_is_amber() {
		let theme = Theme::default();

		assert_eq!(theme.selected_tab, Color::Rgb(255, 216, 102));
		assert_eq!(theme.command_fg, Color::Rgb(255, 176, 0));
		assert_eq!(theme.selection_bg, Color::Rgb(42, 26, 0));
		assert_eq!(theme.selection_fg, Color::Rgb(255, 216, 102));
		assert!(theme.use_selection_fg);
		assert_eq!(theme.cmdbar_bg, Color::Rgb(12, 8, 0));
		assert_eq!(theme.disabled_fg, Color::Rgb(153, 99, 0));
		assert_eq!(theme.diff_line_add, Color::Rgb(127, 217, 98));
		assert_eq!(theme.diff_line_delete, Color::Rgb(255, 85, 0));
		assert_eq!(theme.diff_file_added, Color::Rgb(127, 217, 98));
		assert_eq!(theme.diff_file_removed, Color::Rgb(255, 85, 0));
		assert_eq!(theme.diff_file_moved, Color::Rgb(255, 216, 102));
		assert_eq!(theme.diff_file_modified, Color::Rgb(255, 176, 0));
		assert_eq!(theme.commit_hash, Color::Rgb(255, 216, 102));
		assert_eq!(theme.commit_time, Color::Rgb(153, 99, 0));
		assert_eq!(theme.commit_author, Color::Rgb(255, 176, 0));
		assert_eq!(theme.danger_fg, Color::Rgb(255, 85, 0));
		assert_eq!(theme.push_gauge_bg, Color::Rgb(92, 58, 0));
		assert_eq!(theme.push_gauge_fg, Color::Rgb(255, 216, 102));
		assert_eq!(theme.tag_fg, Color::Rgb(255, 216, 102));
		assert_eq!(theme.branch_fg, Color::Rgb(255, 176, 0));
		assert_eq!(theme.line_break, "¶");
		assert_eq!(
			theme.block_title_focused,
			Color::Rgb(255, 216, 102)
		);
		assert_eq!(theme.syntax, DEFAULT_SYNTAX_THEME);
	}

	#[test]
	fn t12a_partial_old_theme_loads() {
		let mut file = NamedTempFile::new().unwrap();
		writeln!(file, r#"(selection_bg: "Black", line_break: "~")"#)
			.unwrap();
		let path = file.path().to_path_buf();

		let expected = Theme {
			selection_bg: Color::Black,
			line_break: "~".to_string(),
			..Theme::default()
		};

		let first = Theme::init(&path);
		assert_theme_eq(&first, &expected);

		let second = Theme::init(&path);
		assert_theme_eq(&second, &expected);
	}

	#[test]
	fn t12a_bad_file_gives_default() {
		let mut file = NamedTempFile::new().unwrap();
		writeln!(file, "not ron").unwrap();
		let path = file.path().to_path_buf();

		let theme = Theme::init(&path);
		assert_theme_eq(&theme, &Theme::default());
	}

	#[test]
	fn t12b_focused_border_uses_theme() {
		let theme = Theme::default();

		assert_eq!(
			theme.block(true),
			Style::default().fg(Color::Rgb(255, 176, 0))
		);
		assert_eq!(
			theme.block(false),
			Style::default().fg(Color::Rgb(153, 99, 0))
		);
	}

	#[test]
	fn t12b_patch_sets_border_and_track() {
		let mut override_file = NamedTempFile::new().unwrap();
		writeln!(
			override_file,
			r#"(border_focused_fg: Some("Green"), scroll_track_fg: Some("Red"))"#
		)
		.unwrap();

		let overridden =
			Theme::init(&override_file.path().to_path_buf());
		assert_eq!(overridden.block(true).fg, Some(Color::Green));
		assert_eq!(
			overridden.scroll_bar_track().fg,
			Some(Color::Red)
		);

		let mut partial_file = NamedTempFile::new().unwrap();
		writeln!(partial_file, r#"(selection_bg: Some("Black"))"#)
			.unwrap();

		let partial = Theme::init(&partial_file.path().to_path_buf());
		assert_eq!(
			partial.block(true).fg,
			Some(Color::Rgb(255, 176, 0))
		);
		assert_eq!(
			partial.scroll_bar_track().fg,
			Some(Color::Rgb(92, 58, 0))
		);
	}

	#[test]
	fn t12c_panel_bg_default_patch_and_old_format() {
		let default = Theme::default();
		assert_eq!(default.panel_bg, Color::Rgb(12, 8, 0));
		assert_eq!(
			default.popup(),
			Style::default().bg(Color::Rgb(12, 8, 0))
		);

		let mut override_file = NamedTempFile::new().unwrap();
		writeln!(override_file, r#"(panel_bg: Some("Blue"))"#)
			.unwrap();
		let overridden =
			Theme::init(&override_file.path().to_path_buf());
		assert_eq!(overridden.popup().bg, Some(Color::Blue));

		let mut old_file = NamedTempFile::new().unwrap();
		writeln!(old_file, r#"(selection_bg: "Black")"#).unwrap();
		let old_loaded = Theme::init(&old_file.path().to_path_buf());
		assert_eq!(old_loaded.popup().bg, Some(Color::Rgb(12, 8, 0)));
	}

	#[test]
	fn t12d_patch_sets_attention_fg() {
		let default = Theme::default();
		assert_eq!(
			default.attention_block_style().fg,
			Some(Color::Rgb(255, 85, 0))
		);

		let mut override_file = NamedTempFile::new().unwrap();
		writeln!(override_file, r#"(attention_fg: Some("Yellow"))"#)
			.unwrap();
		let overridden =
			Theme::init(&override_file.path().to_path_buf());
		assert_eq!(
			overridden.attention_block_style().fg,
			Some(Color::Yellow)
		);

		let mut partial_file = NamedTempFile::new().unwrap();
		writeln!(partial_file, r#"(selection_bg: Some("Black"))"#)
			.unwrap();
		let partial = Theme::init(&partial_file.path().to_path_buf());
		assert_eq!(
			partial.attention_block_style().fg,
			Some(Color::Rgb(255, 85, 0))
		);
	}
}
