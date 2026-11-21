use unicode_width::UnicodeWidthStr;

#[cfg(feature = "ghemoji")]
pub mod emoji;
pub mod filetree;
pub mod logitems;
pub mod scroll_horizontal;
pub mod scroll_vertical;
pub mod statustree;

/// macro to simplify running code that might return Err.
/// It will show a popup in that case
#[macro_export]
macro_rules! try_or_popup {
	($self:ident, $msg:expr, $e:expr) => {
		if let Err(err) = $e {
			::log::error!("{} {}", $msg, err);
			$self.queue.push(
				$crate::queue::InternalEvent::ShowErrorMsg(format!(
					"{}\n{}",
					$msg, err
				)),
			);
		}
	};
}

/// helper func to convert unix time since epoch to formatted time string in local timezone
pub fn time_to_string(secs: i64, short: bool) -> String {
	let time = jiff::Timestamp::from_second(secs)
		.unwrap_or(jiff::Timestamp::UNIX_EPOCH)
		.to_zoned(jiff::tz::TimeZone::system());

	time.strftime(if short {
		"%Y-%m-%d"
	} else {
		"%Y-%m-%d %H:%M:%S"
	})
	.to_string()
}

#[inline]
pub fn string_width_align(s: &str, width: usize) -> String {
	static POSTFIX: &str = "..";

	let len = UnicodeWidthStr::width(s);
	let width_wo_postfix = width.saturating_sub(POSTFIX.len());

	if (len >= width_wo_postfix && len <= width)
		|| (len <= width_wo_postfix)
	{
		format!("{s:width$}")
	} else {
		let mut s = s.to_string();
		s.truncate(find_truncate_point(&s, width_wo_postfix));
		format!("{s}{POSTFIX}")
	}
}

#[inline]
fn find_truncate_point(s: &str, chars: usize) -> usize {
	s.chars().take(chars).map(char::len_utf8).sum()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn time_to_string_formats_short_and_long() {
		let short = time_to_string(0, true);
		let long = time_to_string(0, false);
		assert_eq!(short.len(), 10);
		assert_eq!(long.len(), 19);
		assert!(long.starts_with(&short));
	}

	#[test]
	fn time_to_string_out_of_range_falls_back() {
		assert_eq!(
			time_to_string(i64::MAX, false),
			time_to_string(0, false)
		);
	}
}
