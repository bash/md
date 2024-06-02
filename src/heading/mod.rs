use crate::block::prelude::*;
use crate::inline::into_inlines;
use anstyle::AnsiColor::Green;
use anstyle::Style;
use pulldown_cmark::HeadingLevel;

mod decoration;
pub use decoration::*;

pub(crate) struct Heading {
    pub(crate) level: HeadingLevel,
}

impl Block for Heading {
    fn kind(&self) -> BlockKind {
        BlockKind::Heading(self.level)
    }

    fn render<'e>(
        self,
        events: Events<'_, 'e, '_>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut Writer,
    ) -> io::Result<()> {
        ctx.counters().update_section(self.level);

        let style = heading_style(self.level);
        let prefix = ctx
            .options()
            .heading_decoration
            .prefix(self.level, || ctx.counters().section());
        let ctx = ctx.block(prefix, style);

        let writer = w.inline_writer(&ctx);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Heading(..)))
                .flat_map(|event| into_inlines(event, &ctx)),
        )
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Green.on_default().bold().underline(),
        HeadingLevel::H2 => Green.on_default().bold(),
        _ => Green.on_default(),
    }
}
