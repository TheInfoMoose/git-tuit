use std::borrow::Cow;

use crate::{
	app::Environment,
	components::{
		commit_details::style::{style_detail, Detail},
		dialog_paragraph,
		utils::time_to_string,
		CommandBlocking, CommandInfo, Component, DrawableComponent,
		EventState,
	},
	strings::{self},
	ui::style::SharedTheme,
};
use anyhow::Result;
use asyncgit::sync::{
	self, commit_files::OldNew, CommitDetails, CommitId, RepoPathRef,
};
use crossterm::event::Event;
use ratatui::{
	layout::{Constraint, Direction, Layout, Rect},
	text::{Line, Span, Text},
	Frame,
};

pub struct CompareDetailsComponent {
	repo: RepoPathRef,
	data: Option<OldNew<CommitDetails>>,
	theme: SharedTheme,
	focused: bool,
}

impl CompareDetailsComponent {
	pub(super) const IDEAL_HEIGHT: u16 = 10;

	///
	pub fn new(env: &Environment, focused: bool) -> Self {
		Self {
			data: None,
			theme: env.theme.clone(),
			focused,
			repo: env.repo.clone(),
		}
	}

	pub fn set_commits(&mut self, ids: Option<OldNew<CommitId>>) {
		self.data = ids.and_then(|ids| {
			let old = sync::get_commit_details(
				&self.repo.borrow(),
				ids.old,
			)
			.ok()?;
			let new = sync::get_commit_details(
				&self.repo.borrow(),
				ids.new,
			)
			.ok()?;

			Some(OldNew { old, new })
		});
	}

	fn get_commit_text(&self, data: &CommitDetails) -> Vec<Line<'_>> {
		let mut res = vec![
			Line::from(vec![
				style_detail(&self.theme, &Detail::Author),
				Span::styled(
					Cow::from(format!(
						"{} <{}>",
						data.author.name, data.author.email
					)),
					self.theme.text(true, false),
				),
			]),
			Line::from(vec![
				style_detail(&self.theme, &Detail::Date),
				Span::styled(
					Cow::from(time_to_string(
						data.author.time,
						false,
					)),
					self.theme.text(true, false),
				),
			]),
		];

		res.push(Line::from(vec![
			style_detail(&self.theme, &Detail::Message),
			Span::styled(
				Cow::from(
					data.message
						.as_ref()
						.map(|msg| msg.subject.clone())
						.unwrap_or_default(),
				),
				self.theme.text(true, false),
			),
		]));

		res
	}

	const fn split_heights(total: u16) -> (u16, u16) {
		const IDEAL_EACH: u16 = 5;
		if total >= IDEAL_EACH * 2 {
			return (IDEAL_EACH, IDEAL_EACH);
		}
		let first = total / 2;
		let second = total.saturating_sub(first);
		(first, second)
	}

	#[cfg(test)]
	pub(super) fn set_test_data(
		&mut self,
		data: Option<OldNew<CommitDetails>>,
	) {
		self.data = data;
	}
}

impl DrawableComponent for CompareDetailsComponent {
	fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
		let (old_height, new_height) =
			Self::split_heights(rect.height);

		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Length(old_height),
					Constraint::Length(new_height),
				]
				.as_ref(),
			)
			.split(rect);

		if let Some(data) = &self.data {
			f.render_widget(
				dialog_paragraph(
					&strings::commit::compare_details_info_title(
						true,
						data.old.short_hash(),
					),
					Text::from(self.get_commit_text(&data.old)),
					&self.theme,
					false,
				),
				chunks[0],
			);

			f.render_widget(
				dialog_paragraph(
					&strings::commit::compare_details_info_title(
						false,
						data.new.short_hash(),
					),
					Text::from(self.get_commit_text(&data.new)),
					&self.theme,
					false,
				),
				chunks[1],
			);
		}

		Ok(())
	}
}

impl Component for CompareDetailsComponent {
	fn commands(
		&self,
		_out: &mut Vec<CommandInfo>,
		_force_all: bool,
	) -> CommandBlocking {
		CommandBlocking::PassingOn
	}

	fn event(&mut self, _event: &Event) -> Result<EventState> {
		Ok(EventState::NotConsumed)
	}

	fn focused(&self) -> bool {
		self.focused
	}

	fn focus(&mut self, focus: bool) {
		self.focused = focus;
	}
}
