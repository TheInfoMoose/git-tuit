use crate::ui::syntax_text::DEFAULT_SYNTAX_THEME;
use anyhow::Result;
use asyncgit::{DiffLineType, StatusItemType};
use ratatui::style::{Color, Modifier, Style};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
	fs::{self, File},
	io::{ErrorKind, Write},
	path::Path,
	rc::Rc,
};
use struct_patch::Patch;

pub type SharedTheme = Rc<Theme>;

const AMBER_PRIMARY: Color = Color::Rgb(255, 176, 0);
const AMBER_BRIGHT: Color = Color::Rgb(255, 216, 102);
const AMBER_DIM: Color = Color::Rgb(153, 99, 0);
const AMBER_DARK: Color = Color::Rgb(92, 58, 0);
const AMBER_ALERT: Color = Color::Rgb(255, 85, 0);
const BG_BLACK: Color = Color::Rgb(0, 0, 0);
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
	scroll_thumb_fg: Color,
	panel_bg: Color,
	attention_fg: Color,
	canvas_bg: Color,
	rule_fg: Color,
	syntax: String,
}

#[derive(Deserialize)]
struct ThemePatchFile {
	#[serde(
		default,
		deserialize_with = "deserialize_optional_string"
	)]
	base: Option<String>,
	#[serde(default)]
	selected_tab: Option<Color>,
	#[serde(default)]
	command_fg: Option<Color>,
	#[serde(default)]
	selection_bg: Option<Color>,
	#[serde(default)]
	selection_fg: Option<Color>,
	#[serde(default)]
	use_selection_fg: Option<bool>,
	#[serde(default)]
	cmdbar_bg: Option<Color>,
	#[serde(default)]
	disabled_fg: Option<Color>,
	#[serde(default)]
	diff_line_add: Option<Color>,
	#[serde(default)]
	diff_line_delete: Option<Color>,
	#[serde(default)]
	diff_file_added: Option<Color>,
	#[serde(default)]
	diff_file_removed: Option<Color>,
	#[serde(default)]
	diff_file_moved: Option<Color>,
	#[serde(default)]
	diff_file_modified: Option<Color>,
	#[serde(default)]
	commit_hash: Option<Color>,
	#[serde(default)]
	commit_time: Option<Color>,
	#[serde(default)]
	commit_author: Option<Color>,
	#[serde(default)]
	danger_fg: Option<Color>,
	#[serde(default)]
	push_gauge_bg: Option<Color>,
	#[serde(default)]
	push_gauge_fg: Option<Color>,
	#[serde(default)]
	tag_fg: Option<Color>,
	#[serde(default)]
	branch_fg: Option<Color>,
	#[serde(default)]
	line_break: Option<String>,
	#[serde(default)]
	block_title_focused: Option<Color>,
	#[serde(default)]
	border_focused_fg: Option<Color>,
	#[serde(default)]
	scroll_track_fg: Option<Color>,
	#[serde(default)]
	scroll_thumb_fg: Option<Color>,
	#[serde(default)]
	panel_bg: Option<Color>,
	#[serde(default)]
	attention_fg: Option<Color>,
	#[serde(default)]
	canvas_bg: Option<Color>,
	#[serde(default)]
	rule_fg: Option<Color>,
	#[serde(default)]
	syntax: Option<String>,
}

impl ThemePatchFile {
	fn into_parts(self) -> (Option<String>, ThemePatch) {
		(
			self.base,
			ThemePatch {
				selected_tab: self.selected_tab,
				command_fg: self.command_fg,
				selection_bg: self.selection_bg,
				selection_fg: self.selection_fg,
				use_selection_fg: self.use_selection_fg,
				cmdbar_bg: self.cmdbar_bg,
				disabled_fg: self.disabled_fg,
				diff_line_add: self.diff_line_add,
				diff_line_delete: self.diff_line_delete,
				diff_file_added: self.diff_file_added,
				diff_file_removed: self.diff_file_removed,
				diff_file_moved: self.diff_file_moved,
				diff_file_modified: self.diff_file_modified,
				commit_hash: self.commit_hash,
				commit_time: self.commit_time,
				commit_author: self.commit_author,
				danger_fg: self.danger_fg,
				push_gauge_bg: self.push_gauge_bg,
				push_gauge_fg: self.push_gauge_fg,
				tag_fg: self.tag_fg,
				branch_fg: self.branch_fg,
				line_break: self.line_break,
				block_title_focused: self.block_title_focused,
				border_focused_fg: self.border_focused_fg,
				scroll_track_fg: self.scroll_track_fg,
				scroll_thumb_fg: self.scroll_thumb_fg,
				panel_bg: self.panel_bg,
				attention_fg: self.attention_fg,
				canvas_bg: self.canvas_bg,
				rule_fg: self.rule_fg,
				syntax: self.syntax,
			},
		)
	}
}

fn deserialize_optional_string<'de, D>(
	deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
	D: Deserializer<'de>,
{
	struct OptionalStringVisitor;

	impl<'de> serde::de::Visitor<'de> for OptionalStringVisitor {
		type Value = Option<String>;

		fn expecting(
			&self,
			formatter: &mut std::fmt::Formatter<'_>,
		) -> std::fmt::Result {
			formatter.write_str("a string, Some(string), or None")
		}

		fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(None)
		}

		fn visit_some<D>(
			self,
			deserializer: D,
		) -> std::result::Result<Self::Value, D::Error>
		where
			D: Deserializer<'de>,
		{
			String::deserialize(deserializer).map(Some)
		}

		fn visit_str<E>(
			self,
			v: &str,
		) -> std::result::Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(Some(v.to_owned()))
		}

		fn visit_string<E>(
			self,
			v: String,
		) -> std::result::Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(Some(v))
		}
	}

	deserializer.deserialize_any(OptionalStringVisitor)
}

#[derive(Default)]
enum OldBaseField {
	#[default]
	Absent,
	Present(Option<String>),
}

fn deserialize_old_base<'de, D>(
	deserializer: D,
) -> std::result::Result<OldBaseField, D::Error>
where
	D: Deserializer<'de>,
{
	deserialize_optional_string(deserializer)
		.map(OldBaseField::Present)
}

fn deserialize_some<'de, D, T>(
	deserializer: D,
) -> std::result::Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
struct OldThemeFile {
	#[serde(default, deserialize_with = "deserialize_old_base")]
	base: OldBaseField,
	#[serde(default, deserialize_with = "deserialize_some")]
	selected_tab: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	command_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	selection_bg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	selection_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	use_selection_fg: Option<bool>,
	#[serde(default, deserialize_with = "deserialize_some")]
	cmdbar_bg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	disabled_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_line_add: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_line_delete: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_file_added: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_file_removed: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_file_moved: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	diff_file_modified: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	commit_hash: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	commit_time: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	commit_author: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	danger_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	push_gauge_bg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	push_gauge_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	tag_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	branch_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	line_break: Option<String>,
	#[serde(default, deserialize_with = "deserialize_some")]
	block_title_focused: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	border_focused_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	scroll_track_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	scroll_thumb_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	panel_bg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	attention_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	canvas_bg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	rule_fg: Option<Color>,
	#[serde(default, deserialize_with = "deserialize_some")]
	syntax: Option<String>,
}

impl OldThemeFile {
	fn into_parts(self) -> (OldBaseField, ThemePatch) {
		(
			self.base,
			ThemePatch {
				selected_tab: self.selected_tab,
				command_fg: self.command_fg,
				selection_bg: self.selection_bg,
				selection_fg: self.selection_fg,
				use_selection_fg: self.use_selection_fg,
				cmdbar_bg: self.cmdbar_bg,
				disabled_fg: self.disabled_fg,
				diff_line_add: self.diff_line_add,
				diff_line_delete: self.diff_line_delete,
				diff_file_added: self.diff_file_added,
				diff_file_removed: self.diff_file_removed,
				diff_file_moved: self.diff_file_moved,
				diff_file_modified: self.diff_file_modified,
				commit_hash: self.commit_hash,
				commit_time: self.commit_time,
				commit_author: self.commit_author,
				danger_fg: self.danger_fg,
				push_gauge_bg: self.push_gauge_bg,
				push_gauge_fg: self.push_gauge_fg,
				tag_fg: self.tag_fg,
				branch_fg: self.branch_fg,
				line_break: self.line_break,
				block_title_focused: self.block_title_focused,
				border_focused_fg: self.border_focused_fg,
				scroll_track_fg: self.scroll_track_fg,
				scroll_thumb_fg: self.scroll_thumb_fg,
				panel_bg: self.panel_bg,
				attention_fg: self.attention_fg,
				canvas_bg: self.canvas_bg,
				rule_fg: self.rule_fg,
				syntax: self.syntax,
			},
		)
	}
}

impl Theme {
	pub fn classic() -> Self {
		Self {
			selected_tab: Color::Reset,
			command_fg: Color::White,
			selection_bg: Color::Blue,
			selection_fg: Color::White,
			use_selection_fg: true,
			cmdbar_bg: Color::Blue,
			disabled_fg: Color::DarkGray,
			diff_line_add: Color::Green,
			diff_line_delete: Color::Red,
			diff_file_added: Color::LightGreen,
			diff_file_removed: Color::LightRed,
			diff_file_moved: Color::LightMagenta,
			diff_file_modified: Color::Yellow,
			commit_hash: Color::Magenta,
			commit_time: Color::LightCyan,
			commit_author: Color::Green,
			danger_fg: Color::Red,
			push_gauge_bg: Color::Blue,
			push_gauge_fg: Color::Reset,
			tag_fg: Color::LightMagenta,
			branch_fg: Color::LightYellow,
			line_break: "¶".to_string(),
			block_title_focused: Color::Reset,
			border_focused_fg: Color::Reset,
			scroll_track_fg: Color::Reset,
			scroll_thumb_fg: Color::Blue,
			panel_bg: Color::Reset,
			attention_fg: Color::Yellow,
			canvas_bg: Color::Reset,
			rule_fg: Color::DarkGray,
			syntax: DEFAULT_SYNTAX_THEME.to_string(),
		}
	}

	pub fn canvas(&self) -> Style {
		Style::default().bg(self.canvas_bg)
	}

	pub fn rule(&self) -> Style {
		Style::default().fg(self.rule_fg)
	}

	pub fn popup(&self) -> Style {
		Style::default().bg(self.panel_bg)
	}

	pub fn scroll_bar_pos(&self) -> Style {
		Style::default().fg(self.scroll_thumb_fg)
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
				.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
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

	fn apply_classic_patch(patch: ThemePatch) -> Self {
		let thumb_set = patch.scroll_thumb_fg.is_some();
		let rule_set = patch.rule_fg.is_some();
		let mut theme = Self::classic();
		theme.apply(patch);
		if !thumb_set {
			theme.scroll_thumb_fg = theme.selection_bg;
		}
		if !rule_set {
			theme.rule_fg = theme.disabled_fg;
		}
		theme
	}

	// This is supposed to be called when converting an old full-format theme.
	fn save_patch(&self, theme_path: &Path) -> Result<()> {
		let mut file = File::create(theme_path)?;
		let mut patch =
			self.clone().into_patch_by_diff(Self::classic());
		if self.scroll_thumb_fg != self.selection_bg {
			patch.scroll_thumb_fg = Some(self.scroll_thumb_fg);
		}
		patch.rule_fg = (self.rule_fg != self.disabled_fg)
			.then_some(self.rule_fg);
		let data = to_string_pretty(&patch, PrettyConfig::default())?;

		file.write_all(data.as_bytes())?;

		Ok(())
	}

	pub fn get_syntax(&self) -> String {
		self.syntax.clone()
	}

	fn is_truecolor(colorterm: Option<&str>) -> bool {
		colorterm.is_some_and(|val| {
			let trimmed = val.trim();
			trimmed.eq_ignore_ascii_case("truecolor")
				|| trimmed.eq_ignore_ascii_case("24bit")
		})
	}

	fn resolve_patch(
		base: Option<&str>,
		patch: ThemePatch,
		theme_path: &Path,
	) -> (Self, Option<String>) {
		match base {
			None => (
				Self::apply_classic_patch(patch),
				Some(format!(
					"theme {theme_path:?} has no base; unset fields use the classic colors. Add base: Some(\"amber\") to use the amber theme."
				)),
			),
			Some(name) => {
				let trimmed = name.trim();
				if trimmed.eq_ignore_ascii_case("amber") {
					let mut theme = Self::default();
					theme.apply(patch);
					(theme, None)
				} else if trimmed.eq_ignore_ascii_case("classic") {
					(Self::apply_classic_patch(patch), None)
				} else {
					log::warn!(
						"unknown theme base {name:?} in {theme_path:?}; using classic"
					);
					(Self::apply_classic_patch(patch), None)
				}
			}
		}
	}

	fn load_with_notice(
		theme_path: &Path,
		colorterm: Option<&str>,
	) -> (Self, Option<String>) {
		let content = match fs::read_to_string(theme_path) {
			Ok(content) => content,
			Err(e) if e.kind() == ErrorKind::NotFound => {
				let theme = if Self::is_truecolor(colorterm) {
					Self::default()
				} else {
					Self::classic()
				};
				return (theme, None);
			}
			Err(e) => {
				log::error!("theme error [{theme_path:?}]: {e}");
				return (Self::classic(), None);
			}
		};

		match ron::de::from_str::<ThemePatchFile>(&content) {
			Ok(patch_file) => {
				let (base, patch) = patch_file.into_parts();
				Self::resolve_patch(
					base.as_deref(),
					patch,
					theme_path,
				)
			}
			Err(patch_err) => {
				ron::de::from_str::<OldThemeFile>(&content)
					.map_or_else(
						|_| {
							log::error!(
								"theme error [{theme_path:?}]: {patch_err}"
							);
							(Self::classic(), None)
						},
						|old_theme| {
							let (base, patch) =
								old_theme.into_parts();
							match base {
								OldBaseField::Present(base) => {
									Self::resolve_patch(
										base.as_deref(),
										patch,
										theme_path,
									)
								}
								OldBaseField::Absent => {
									let theme =
										Self::apply_classic_patch(
											patch,
										);

									if theme
										.save_patch(theme_path)
										.is_ok()
									{
										log::info!(
											"Converted old theme to new format. ({theme_path:?})"
										);
									} else {
										log::warn!(
											"Failed to save theme in new format. ({theme_path:?})"
										);
									}

									(theme, None)
								}
							}
						},
					)
			}
		}
	}

	fn init_for(theme_path: &Path, colorterm: Option<&str>) -> Self {
		let (theme, notice) =
			Self::load_with_notice(theme_path, colorterm);
		if let Some(notice) = notice {
			log::info!("{notice}");
		}
		theme
	}

	pub fn init(theme_path: &Path) -> Self {
		let colorterm = std::env::var("COLORTERM").ok();
		Self::init_for(theme_path, colorterm.as_deref())
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
			scroll_thumb_fg: AMBER_BRIGHT,
			panel_bg: PANEL_BG,
			attention_fg: AMBER_ALERT,
			canvas_bg: BG_BLACK,
			rule_fg: AMBER_DARK,
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

		let theme = Theme::init(file.path());

		assert_eq!(theme.selected_tab, Theme::classic().selected_tab);

		assert_ne!(theme.selection_bg, Theme::default().selection_bg);
		assert_ne!(theme.syntax, Theme::default().syntax);
		assert_eq!(theme.selection_bg, Color::Black);
		assert_eq!(theme.selection_fg, Color::Rgb(255, 255, 255));
		assert_eq!(theme.syntax, "InspiredGitHub");
	}

	fn assert_theme_eq(actual: &Theme, expected: &Theme) {
		let Theme {
			selected_tab,
			command_fg,
			selection_bg,
			selection_fg,
			use_selection_fg,
			cmdbar_bg,
			disabled_fg,
			diff_line_add,
			diff_line_delete,
			diff_file_added,
			diff_file_removed,
			diff_file_moved,
			diff_file_modified,
			commit_hash,
			commit_time,
			commit_author,
			danger_fg,
			push_gauge_bg,
			push_gauge_fg,
			tag_fg,
			branch_fg,
			line_break,
			block_title_focused,
			border_focused_fg,
			scroll_track_fg,
			scroll_thumb_fg,
			panel_bg,
			attention_fg,
			canvas_bg,
			rule_fg,
			syntax,
		} = actual;

		assert_eq!(selected_tab, &expected.selected_tab);
		assert_eq!(command_fg, &expected.command_fg);
		assert_eq!(selection_bg, &expected.selection_bg);
		assert_eq!(selection_fg, &expected.selection_fg);
		assert_eq!(use_selection_fg, &expected.use_selection_fg);
		assert_eq!(cmdbar_bg, &expected.cmdbar_bg);
		assert_eq!(disabled_fg, &expected.disabled_fg);
		assert_eq!(diff_line_add, &expected.diff_line_add);
		assert_eq!(diff_line_delete, &expected.diff_line_delete);
		assert_eq!(diff_file_added, &expected.diff_file_added);
		assert_eq!(diff_file_removed, &expected.diff_file_removed);
		assert_eq!(diff_file_moved, &expected.diff_file_moved);
		assert_eq!(diff_file_modified, &expected.diff_file_modified);
		assert_eq!(commit_hash, &expected.commit_hash);
		assert_eq!(commit_time, &expected.commit_time);
		assert_eq!(commit_author, &expected.commit_author);
		assert_eq!(danger_fg, &expected.danger_fg);
		assert_eq!(push_gauge_bg, &expected.push_gauge_bg);
		assert_eq!(push_gauge_fg, &expected.push_gauge_fg);
		assert_eq!(tag_fg, &expected.tag_fg);
		assert_eq!(branch_fg, &expected.branch_fg);
		assert_eq!(line_break, &expected.line_break);
		assert_eq!(
			block_title_focused,
			&expected.block_title_focused
		);
		assert_eq!(border_focused_fg, &expected.border_focused_fg);
		assert_eq!(scroll_track_fg, &expected.scroll_track_fg);
		assert_eq!(scroll_thumb_fg, &expected.scroll_thumb_fg);
		assert_eq!(panel_bg, &expected.panel_bg);
		assert_eq!(attention_fg, &expected.attention_fg);
		assert_eq!(canvas_bg, &expected.canvas_bg);
		assert_eq!(rule_fg, &expected.rule_fg);
		assert_eq!(syntax, &expected.syntax);
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
		assert_eq!(theme.border_focused_fg, Color::Rgb(255, 176, 0));
		assert_eq!(theme.scroll_track_fg, Color::Rgb(92, 58, 0));
		assert_eq!(theme.scroll_thumb_fg, Color::Rgb(255, 216, 102));
		assert_eq!(theme.panel_bg, Color::Rgb(12, 8, 0));
		assert_eq!(theme.attention_fg, Color::Rgb(255, 85, 0));
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
			scroll_thumb_fg: Color::Black,
			line_break: "~".to_string(),
			..Theme::classic()
		};

		let first = Theme::init(&path);
		assert_theme_eq(&first, &expected);

		let second = Theme::init(&path);
		assert_theme_eq(&second, &expected);
	}

	#[test]
	fn t13b_bad_file_gives_classic() {
		let mut file = NamedTempFile::new().unwrap();
		writeln!(file, "not ron").unwrap();
		let path = file.path().to_path_buf();

		let theme = Theme::init(&path);
		assert_theme_eq(&theme, &Theme::classic());
		assert_eq!(fs::read_to_string(&path).unwrap(), "not ron\n");
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

		let overridden = Theme::init(override_file.path());
		assert_eq!(overridden.block(true).fg, Some(Color::Green));
		assert_eq!(
			overridden.scroll_bar_track().fg,
			Some(Color::Red)
		);

		let mut partial_file = NamedTempFile::new().unwrap();
		writeln!(partial_file, r#"(selection_bg: Some("Black"))"#)
			.unwrap();

		let partial = Theme::init(partial_file.path());
		assert_eq!(partial.block(true).fg, Some(Color::Reset));
		assert_eq!(partial.scroll_bar_track().fg, Some(Color::Reset));
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
		let overridden = Theme::init(override_file.path());
		assert_eq!(overridden.popup().bg, Some(Color::Blue));

		let mut old_file = NamedTempFile::new().unwrap();
		writeln!(old_file, r#"(selection_bg: "Black")"#).unwrap();
		let old_loaded = Theme::init(old_file.path());
		assert_eq!(old_loaded.popup().bg, Some(Color::Reset));
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
		let overridden = Theme::init(override_file.path());
		assert_eq!(
			overridden.attention_block_style().fg,
			Some(Color::Yellow)
		);

		let mut partial_file = NamedTempFile::new().unwrap();
		writeln!(partial_file, r#"(selection_bg: Some("Black"))"#)
			.unwrap();
		let partial = Theme::init(partial_file.path());
		assert_eq!(
			partial.attention_block_style().fg,
			Some(Color::Yellow)
		);
	}

	#[test]
	fn t13b_classic_matches_launch() {
		// Values from impl Default for Theme at commit 389ae89b,
		// plus the five fields added in Phase 12/13A.
		let theme = Theme::classic();

		assert_eq!(theme.selected_tab, Color::Reset);
		assert_eq!(theme.command_fg, Color::White);
		assert_eq!(theme.selection_bg, Color::Blue);
		assert_eq!(theme.selection_fg, Color::White);
		assert!(theme.use_selection_fg);
		assert_eq!(theme.cmdbar_bg, Color::Blue);
		assert_eq!(theme.disabled_fg, Color::DarkGray);
		assert_eq!(theme.diff_line_add, Color::Green);
		assert_eq!(theme.diff_line_delete, Color::Red);
		assert_eq!(theme.diff_file_added, Color::LightGreen);
		assert_eq!(theme.diff_file_removed, Color::LightRed);
		assert_eq!(theme.diff_file_moved, Color::LightMagenta);
		assert_eq!(theme.diff_file_modified, Color::Yellow);
		assert_eq!(theme.commit_hash, Color::Magenta);
		assert_eq!(theme.commit_time, Color::LightCyan);
		assert_eq!(theme.commit_author, Color::Green);
		assert_eq!(theme.danger_fg, Color::Red);
		assert_eq!(theme.push_gauge_bg, Color::Blue);
		assert_eq!(theme.push_gauge_fg, Color::Reset);
		assert_eq!(theme.tag_fg, Color::LightMagenta);
		assert_eq!(theme.branch_fg, Color::LightYellow);
		assert_eq!(theme.line_break, "¶");
		assert_eq!(theme.block_title_focused, Color::Reset);
		assert_eq!(theme.border_focused_fg, Color::Reset);
		assert_eq!(theme.scroll_track_fg, Color::Reset);
		assert_eq!(theme.scroll_thumb_fg, Color::Blue);
		assert_eq!(theme.panel_bg, Color::Reset);
		assert_eq!(theme.attention_fg, Color::Yellow);
		assert_eq!(theme.syntax, DEFAULT_SYNTAX_THEME);
	}

	#[test]
	fn t13b_patch_without_base_is_classic() {
		let mut sel_file = NamedTempFile::new().unwrap();
		writeln!(sel_file, r#"(selection_bg: Some("LightBlue"))"#)
			.unwrap();
		let sel_theme = Theme::init(sel_file.path());
		assert_theme_eq(
			&sel_theme,
			&Theme {
				selection_bg: Color::LightBlue,
				scroll_thumb_fg: Color::LightBlue,
				..Theme::classic()
			},
		);

		let mut empty_file = NamedTempFile::new().unwrap();
		writeln!(empty_file, "()").unwrap();
		let empty_theme = Theme::init(empty_file.path());
		assert_theme_eq(&empty_theme, &Theme::classic());
	}

	#[test]
	fn t13b_patch_base_amber() {
		let mut file = NamedTempFile::new().unwrap();
		writeln!(
			file,
			r#"(base: Some("amber"), selection_bg: Some("Black"))"#
		)
		.unwrap();
		let theme = Theme::init(file.path());
		assert_theme_eq(
			&theme,
			&Theme {
				selection_bg: Color::Black,
				..Theme::default()
			},
		);
	}

	#[test]
	fn t13b_patch_base_classic_and_unknown() {
		let mut classic_file = NamedTempFile::new().unwrap();
		writeln!(classic_file, r#"(base: Some("classic"))"#).unwrap();
		let classic_theme = Theme::init(classic_file.path());
		assert_theme_eq(&classic_theme, &Theme::classic());

		let mut unknown_file = NamedTempFile::new().unwrap();
		writeln!(
			unknown_file,
			r#"(base: Some("neon"), tag_fg: Some("Red"))"#
		)
		.unwrap();
		let unknown_theme = Theme::init(unknown_file.path());
		assert_theme_eq(
			&unknown_theme,
			&Theme {
				tag_fg: Color::Red,
				..Theme::classic()
			},
		);
	}

	#[test]
	fn t13b_patch_thumb_wins() {
		let mut file = NamedTempFile::new().unwrap();
		writeln!(
			file,
			r#"(selection_bg: Some("LightBlue"), scroll_thumb_fg: Some("Cyan"))"#
		)
		.unwrap();
		let theme = Theme::init(file.path());
		assert_theme_eq(
			&theme,
			&Theme {
				selection_bg: Color::LightBlue,
				scroll_thumb_fg: Color::Cyan,
				..Theme::classic()
			},
		);
	}

	#[test]
	fn t13c_missing_file_truecolor_is_amber() {
		let dir = tempfile::tempdir().unwrap();
		let missing = dir.path().join("does-not-exist.ron");
		for val in ["truecolor", "24bit", " TrueColor "] {
			let theme = Theme::init_for(&missing, Some(val));
			assert_theme_eq(&theme, &Theme::default());
		}
	}

	#[test]
	fn t13c_missing_file_no_truecolor_is_classic() {
		let dir = tempfile::tempdir().unwrap();
		let missing = dir.path().join("does-not-exist.ron");
		for val in [None, Some(""), Some("256color")] {
			let theme = Theme::init_for(&missing, val);
			assert_theme_eq(&theme, &Theme::classic());
		}
	}

	#[test]
	fn t13c_file_ignores_colorterm() {
		let mut amber_file = NamedTempFile::new().unwrap();
		writeln!(amber_file, r#"(base: Some("amber"))"#).unwrap();
		let amber = Theme::init_for(amber_file.path(), None);
		assert_theme_eq(&amber, &Theme::default());

		let mut tag_file = NamedTempFile::new().unwrap();
		writeln!(tag_file, r#"(tag_fg: Some("Red"))"#).unwrap();
		let tagged =
			Theme::init_for(tag_file.path(), Some("truecolor"));
		assert_theme_eq(
			&tagged,
			&Theme {
				tag_fg: Color::Red,
				..Theme::classic()
			},
		);

		let mut bad_file = NamedTempFile::new().unwrap();
		writeln!(bad_file, "not ron").unwrap();
		let bad = Theme::init_for(bad_file.path(), Some("truecolor"));
		assert_theme_eq(&bad, &Theme::classic());

		let dir = tempfile::tempdir().unwrap();
		let unreadable =
			Theme::init_for(dir.path(), Some("truecolor"));
		assert_theme_eq(&unreadable, &Theme::classic());
	}

	#[test]
	fn t14a_bare_base_amber() {
		let bare_text = "(base: \"amber\")\n";
		let mut bare_file = NamedTempFile::new().unwrap();
		bare_file.write_all(bare_text.as_bytes()).unwrap();
		for colorterm in [Some("truecolor"), None] {
			let theme = Theme::init_for(bare_file.path(), colorterm);
			assert_theme_eq(&theme, &Theme::default());
			assert_eq!(
				fs::read_to_string(bare_file.path()).unwrap(),
				bare_text
			);
		}

		let patch_text =
			"(base: \"amber\", selection_bg: Some(\"Black\"))\n";
		let mut patch_file = NamedTempFile::new().unwrap();
		patch_file.write_all(patch_text.as_bytes()).unwrap();
		let expected = Theme {
			selection_bg: Color::Black,
			..Theme::default()
		};
		for colorterm in [Some("truecolor"), None] {
			let theme = Theme::init_for(patch_file.path(), colorterm);
			assert_theme_eq(&theme, &expected);
			assert_eq!(
				fs::read_to_string(patch_file.path()).unwrap(),
				patch_text
			);
		}
	}

	#[test]
	fn t14a_base_ignores_case() {
		for text in [
			r#"(base: Some("Amber"))"#,
			r#"(base: Some(" AMBER "))"#,
			r#"(base: "Amber")"#,
		] {
			let mut file = NamedTempFile::new().unwrap();
			writeln!(file, "{text}").unwrap();
			for colorterm in [Some("truecolor"), None] {
				let theme = Theme::init_for(file.path(), colorterm);
				assert_theme_eq(&theme, &Theme::default());
			}
		}

		let mut classic_file = NamedTempFile::new().unwrap();
		writeln!(
			classic_file,
			r#"(base: Some("CLASSIC"), tag_fg: Some("Red"))"#
		)
		.unwrap();
		let expected_classic = Theme {
			tag_fg: Color::Red,
			..Theme::classic()
		};
		for colorterm in [Some("truecolor"), None] {
			let theme =
				Theme::init_for(classic_file.path(), colorterm);
			assert_theme_eq(&theme, &expected_classic);
		}

		let mut unknown_file = NamedTempFile::new().unwrap();
		writeln!(unknown_file, r#"(base: "neon")"#).unwrap();
		for colorterm in [Some("truecolor"), None] {
			let theme =
				Theme::init_for(unknown_file.path(), colorterm);
			assert_theme_eq(&theme, &Theme::classic());
		}

		let mut none_file = NamedTempFile::new().unwrap();
		writeln!(none_file, "(base: None)").unwrap();
		for colorterm in [Some("truecolor"), None] {
			let theme = Theme::init_for(none_file.path(), colorterm);
			assert_theme_eq(&theme, &Theme::classic());
		}
	}

	#[test]
	fn t14a_base_file_never_rewritten() {
		for text in [
			"(base: 5)\n",
			"(base: Some(5), selection_bg: \"Black\")\n",
		] {
			for colorterm in [Some("truecolor"), None] {
				let mut file = NamedTempFile::new().unwrap();
				file.write_all(text.as_bytes()).unwrap();
				let theme = Theme::init_for(file.path(), colorterm);
				assert_theme_eq(&theme, &Theme::classic());
				assert_eq!(
					fs::read_to_string(file.path()).unwrap(),
					text
				);
			}
		}
	}

	#[test]
	fn t15a_mixed_base_keeps_colors() {
		let cases = [
			(
				"(base: \"amber\", selection_bg: \"Black\")\n",
				Theme {
					selection_bg: Color::Black,
					..Theme::default()
				},
			),
			(
				"(base: Some(\"amber\"), selection_bg: \"Black\")\n",
				Theme {
					selection_bg: Color::Black,
					..Theme::default()
				},
			),
			(
				"(base: \" AMBER \", tag_fg: \"Red\")\n",
				Theme {
					tag_fg: Color::Red,
					..Theme::default()
				},
			),
			(
				"(base: \"classic\", selection_bg: \"Black\")\n",
				Theme {
					selection_bg: Color::Black,
					scroll_thumb_fg: Color::Black,
					..Theme::classic()
				},
			),
			(
				"(base: \"neon\", tag_fg: \"Red\")\n",
				Theme {
					tag_fg: Color::Red,
					..Theme::classic()
				},
			),
			(
				"(base: None, tag_fg: \"Red\")\n",
				Theme {
					tag_fg: Color::Red,
					..Theme::classic()
				},
			),
		];

		for (text, expected) in cases {
			for colorterm in [Some("truecolor"), None] {
				let mut file = NamedTempFile::new().unwrap();
				file.write_all(text.as_bytes()).unwrap();
				let theme = Theme::init_for(file.path(), colorterm);
				assert_theme_eq(&theme, &expected);
				assert_eq!(
					fs::read_to_string(file.path()).unwrap(),
					text
				);
			}
		}
	}

	#[test]
	fn t14a_old_format_with_base_text_converts() {
		let original = "(selection_bg: \"Black\", syntax: \"base16-ocean.dark\", line_break: \"base\")\n";
		let mut file = NamedTempFile::new().unwrap();
		file.write_all(original.as_bytes()).unwrap();
		let path = file.path().to_path_buf();

		let expected = Theme {
			selection_bg: Color::Black,
			scroll_thumb_fg: Color::Black,
			syntax: "base16-ocean.dark".to_string(),
			line_break: "base".to_string(),
			..Theme::classic()
		};

		let first = Theme::init_for(&path, Some("truecolor"));
		assert_theme_eq(&first, &expected);

		let rewritten = fs::read_to_string(&path).unwrap();
		assert_ne!(rewritten, original);

		let second = Theme::init_for(&path, None);
		assert_theme_eq(&second, &expected);
	}

	#[test]
	fn t14b_baseless_file_untouched() {
		let cases = [
			(
				"(tag_fg: Some(\"Red\"))\n",
				Theme {
					tag_fg: Color::Red,
					..Theme::classic()
				},
			),
			("()\n", Theme::classic()),
			("(base: None)\n", Theme::classic()),
		];

		for (text, expected) in cases {
			let mut file = NamedTempFile::new().unwrap();
			file.write_all(text.as_bytes()).unwrap();
			let theme =
				Theme::init_for(file.path(), Some("truecolor"));
			assert_theme_eq(&theme, &expected);
			assert_eq!(
				fs::read_to_string(file.path()).unwrap(),
				text
			);
		}
	}

	#[test]
	fn t15b_notice_only_without_base() {
		let notice_cases = [
			"(tag_fg: Some(\"Red\"))\n",
			"()\n",
			"(base: None)\n",
			"(base: None, tag_fg: \"Red\")\n",
		];

		for text in notice_cases {
			for colorterm in [Some("truecolor"), None] {
				let mut file = NamedTempFile::new().unwrap();
				file.write_all(text.as_bytes()).unwrap();
				let path = file.path();
				let expected_notice = format!(
					"theme {path:?} has no base; unset fields use the classic colors. Add base: Some(\"amber\") to use the amber theme."
				);

				let (theme, notice) =
					Theme::load_with_notice(path, colorterm);
				assert_eq!(
					notice.as_deref(),
					Some(expected_notice.as_str())
				);
				assert_theme_eq(
					&theme,
					&Theme::init_for(path, colorterm),
				);
			}
		}

		let no_notice_cases = [
			"(base: \"amber\")\n",
			"(base: Some(\"classic\"))\n",
			"(base: \"neon\")\n",
			"(base: \"amber\", selection_bg: \"Black\")\n",
			"not ron\n",
			"(base: 5)\n",
			"(selection_bg: \"Black\")\n",
		];

		for text in no_notice_cases {
			for colorterm in [Some("truecolor"), None] {
				let mut file = NamedTempFile::new().unwrap();
				file.write_all(text.as_bytes()).unwrap();
				let path = file.path();

				let (theme, notice) =
					Theme::load_with_notice(path, colorterm);
				assert_eq!(notice, None);
				assert_theme_eq(
					&theme,
					&Theme::init_for(path, colorterm),
				);
			}
		}

		let dir = tempfile::tempdir().unwrap();
		let missing = dir.path().join("does-not-exist.ron");
		for path in [missing.as_path(), dir.path()] {
			for colorterm in [Some("truecolor"), None] {
				let (theme, notice) =
					Theme::load_with_notice(path, colorterm);
				assert_eq!(notice, None);
				assert_theme_eq(
					&theme,
					&Theme::init_for(path, colorterm),
				);
			}
		}
	}

	#[test]
	fn t15d_canvas_bg_from_theme() {
		assert_eq!(Theme::default().canvas_bg, Color::Rgb(0, 0, 0));
		assert_eq!(
			Theme::default().canvas(),
			Style::default().bg(Color::Rgb(0, 0, 0))
		);
		assert_eq!(Theme::classic().canvas_bg, Color::Reset);
		assert_eq!(
			Theme::classic().canvas(),
			Style::default().bg(Color::Reset)
		);

		let mut patch_classic = NamedTempFile::new().unwrap();
		writeln!(patch_classic, r#"(canvas_bg: Some("Black"))"#)
			.unwrap();
		assert_theme_eq(
			&Theme::init_for(patch_classic.path(), None),
			&Theme {
				canvas_bg: Color::Black,
				..Theme::classic()
			},
		);

		let mut patch_amber = NamedTempFile::new().unwrap();
		writeln!(
			patch_amber,
			r#"(base: "amber", canvas_bg: Some("Red"))"#
		)
		.unwrap();
		assert_theme_eq(
			&Theme::init_for(patch_amber.path(), None),
			&Theme {
				canvas_bg: Color::Red,
				..Theme::default()
			},
		);

		let mut old_format = NamedTempFile::new().unwrap();
		writeln!(old_format, r#"(canvas_bg: "Black")"#).unwrap();
		let expected_old = Theme {
			canvas_bg: Color::Black,
			..Theme::classic()
		};
		let first = Theme::init_for(old_format.path(), None);
		assert_theme_eq(&first, &expected_old);
		let second = Theme::init_for(old_format.path(), None);
		assert_theme_eq(&second, &expected_old);
	}

	#[test]
	fn t15e_rule_fg_from_theme() {
		assert_eq!(Theme::default().rule_fg, Color::Rgb(92, 58, 0));
		assert_eq!(
			Theme::default().rule(),
			Style::default().fg(Color::Rgb(92, 58, 0))
		);
		assert_eq!(Theme::classic().rule_fg, Color::DarkGray);
		assert_eq!(
			Theme::classic().rule(),
			Style::default().fg(Color::DarkGray)
		);

		let amber_tab = Theme::default().tab(true);
		assert_eq!(amber_tab.fg, Some(Color::Rgb(255, 216, 102)));
		assert!(amber_tab
			.add_modifier
			.contains(Modifier::BOLD | Modifier::UNDERLINED));
		let classic_tab = Theme::classic().tab(true);
		assert_eq!(classic_tab.fg, Some(Color::Reset));
		assert!(classic_tab
			.add_modifier
			.contains(Modifier::BOLD | Modifier::UNDERLINED));

		// Edge case d: classic file follows disabled_fg unless rule_fg set
		let mut classic_follow = NamedTempFile::new().unwrap();
		writeln!(classic_follow, r#"(disabled_fg: Some("Red"))"#)
			.unwrap();
		assert_theme_eq(
			&Theme::init_for(classic_follow.path(), None),
			&Theme {
				disabled_fg: Color::Red,
				rule_fg: Color::Red,
				..Theme::classic()
			},
		);

		let mut classic_explicit = NamedTempFile::new().unwrap();
		writeln!(
			classic_explicit,
			r#"(disabled_fg: Some("Red"), rule_fg: Some("Green"))"#
		)
		.unwrap();
		assert_theme_eq(
			&Theme::init_for(classic_explicit.path(), None),
			&Theme {
				disabled_fg: Color::Red,
				rule_fg: Color::Green,
				..Theme::classic()
			},
		);

		// Edge case e: amber base has no fallback to disabled_fg
		let mut amber_override = NamedTempFile::new().unwrap();
		writeln!(
			amber_override,
			r#"(base: "amber", disabled_fg: Some("Red"))"#
		)
		.unwrap();
		assert_theme_eq(
			&Theme::init_for(amber_override.path(), None),
			&Theme {
				disabled_fg: Color::Red,
				..Theme::default()
			},
		);

		// Edge case f: old-format conversion
		let mut old_follow = NamedTempFile::new().unwrap();
		writeln!(old_follow, r#"(disabled_fg: "Red")"#).unwrap();
		let expected_follow = Theme {
			disabled_fg: Color::Red,
			rule_fg: Color::Red,
			..Theme::classic()
		};
		let first = Theme::init_for(old_follow.path(), None);
		assert_theme_eq(&first, &expected_follow);
		let rewritten =
			fs::read_to_string(old_follow.path()).unwrap();
		let parsed: ThemePatchFile =
			ron::de::from_str(&rewritten).unwrap();
		assert_eq!(parsed.rule_fg, None);
		let second = Theme::init_for(old_follow.path(), None);
		assert_theme_eq(&second, &expected_follow);

		let mut old_explicit = NamedTempFile::new().unwrap();
		writeln!(old_explicit, r#"(rule_fg: "Green")"#).unwrap();
		let expected_explicit = Theme {
			rule_fg: Color::Green,
			..Theme::classic()
		};
		let first = Theme::init_for(old_explicit.path(), None);
		assert_theme_eq(&first, &expected_explicit);
		let rewritten =
			fs::read_to_string(old_explicit.path()).unwrap();
		let parsed: ThemePatchFile =
			ron::de::from_str(&rewritten).unwrap();
		assert_eq!(parsed.rule_fg, Some(Color::Green));
		let second = Theme::init_for(old_explicit.path(), None);
		assert_theme_eq(&second, &expected_explicit);
	}
}
