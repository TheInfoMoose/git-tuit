use unicode_segmentation::UnicodeSegmentation;
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

	if len <= width {
		let mut out = String::with_capacity(s.len() + (width - len));
		out.push_str(s);
		for _ in 0..(width - len) {
			out.push(' ');
		}
		return out;
	}

	if width < POSTFIX.len() {
		return POSTFIX[..width].to_string();
	}

	let width_wo_postfix = width - POSTFIX.len();
	let mut out = String::with_capacity(width);
	let mut used = 0;
	for grapheme in s.graphemes(true) {
		let w = UnicodeWidthStr::width(grapheme);
		if used + w > width_wo_postfix {
			break;
		}
		out.push_str(grapheme);
		used += w;
	}
	out.push_str(POSTFIX);
	let final_width = UnicodeWidthStr::width(out.as_str());
	for _ in 0..width.saturating_sub(final_width) {
		out.push(' ');
	}
	out
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

	#[test]
	fn t8o_align_wide_text() {
		// Padding CJK to exact display width
		let padded_cjk = string_width_align("日本", 6);
		assert_eq!(UnicodeWidthStr::width(padded_cjk.as_str()), 6);
		assert_eq!(padded_cjk, "日本  ");

		// Truncating CJK on an exact boundary
		let trunc_even = string_width_align("日本語テスト", 6);
		assert_eq!(UnicodeWidthStr::width(trunc_even.as_str()), 6);
		assert_eq!(trunc_even, "日本..");

		// Truncating CJK when a wide character straddles the edge
		let trunc_straddle = string_width_align("日本語テスト", 7);
		assert_eq!(
			UnicodeWidthStr::width(trunc_straddle.as_str()),
			7
		);
		assert_eq!(trunc_straddle, "日本.. ");

		// Padding and truncating emoji, including straddling the edge
		let padded_emoji = string_width_align("😀", 4);
		assert_eq!(UnicodeWidthStr::width(padded_emoji.as_str()), 4);
		assert_eq!(padded_emoji, "😀  ");

		let trunc_emoji = string_width_align("😀😀😀", 5);
		assert_eq!(UnicodeWidthStr::width(trunc_emoji.as_str()), 5);
		assert_eq!(trunc_emoji, "😀.. ");

		let mixed_straddle = string_width_align("a😀bc", 4);
		assert_eq!(
			UnicodeWidthStr::width(mixed_straddle.as_str()),
			4
		);
		assert_eq!(mixed_straddle, "a.. ");
	}
}
