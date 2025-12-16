// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chumsky::input::{Input, Stream};
use chumsky::span::SimpleSpan;

use crate::syntax::SpannedSegments;

pub fn make_input(segs: SpannedSegments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    let eoi = match (segs.segments.first(), segs.segments.last()) {
        (Some(first), Some(last)) => first.1.start..last.1.end,
        _ => 0..0,
    };
    Stream::from_iter(segs.into_spanned_chars()).map(eoi.into(), |(t, s)| (t, s.into()))
}
