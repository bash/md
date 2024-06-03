use crate::block::prelude::*;
use crate::inline::into_inlines;
use crate::ThemeProvider;
use pulldown_cmark::HeadingLevel;

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

        let style = ctx.theme().heading_style(self.level, ctx);
        let prefix = ctx.theme().heading_prefix(self.level, ctx);
        let ctx = ctx.block(prefix, style);

        let writer = w.inline_writer(&ctx);
        writer.write_all(
            terminated!(events, Event::End(TagEnd::Heading(..)))
                .flat_map(|event| into_inlines(event, &ctx)),
        )
    }
}
