use std::fmt;

use tracing::{level_filters::LevelFilter, Event, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    fmt::{
        format::Writer, writer::BoxMakeWriter, FmtContext, FormatEvent,
        FormatFields, FormattedFields,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    Registry,
};

pub fn install_logger() -> anyhow::Result<()> {
    let writer = BoxMakeWriter::new(
        std::fs::File::options()
            .append(true)
            .create(true)
            .write(true)
            .open("/tmp/ew-log.txt")?,
    );
    let ra_fmt_layer = tracing_subscriber::fmt::layer()
        .event_format(LoggerFormatter)
        .with_writer(writer);
    let filter = LevelFilter::TRACE;

    Registry::default().with(ra_fmt_layer).with(filter).init();

    Ok(())
}

#[derive(Debug)]
struct LoggerFormatter;

impl<S, N> FormatEvent<S, N> for LoggerFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self, ctx: &FmtContext<'_, S, N>, mut writer: Writer,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Write level and target
        let level = *event.metadata().level();

        // If this event is issued from `log` crate, then the value of target is
        // always "log". `tracing-log` has hard coded it for some reason, so we
        // need to extract it using `normalized_metadata` method which is part of
        // `tracing_log::NormalizeEvent`.
        let target = match event.normalized_metadata() {
            // This event is issued from `log` crate
            Some(log) => log.target(),
            None => event.metadata().target(),
        };

        write!(writer, "[{} {}] ", level, target)?;

        // Write spans and fields of each span
        ctx.visit_spans(|span| {
            write!(writer, "{}", span.name())?;

            let ext = span.extensions();

            // `FormattedFields` is a a formatted representation of the span's
            // fields, which is stored in its extensions by the `fmt` layer's
            // `new_span` method. The fields will have been formatted
            // by the same field formatter that's provided to the event
            // formatter in the `FmtContext`.
            let fields = &ext
                .get::<FormattedFields<N>>()
                .expect("will never be `None`");

            if !fields.is_empty() {
                write!(writer, "{{{}}}", fields)?;
            }
            write!(writer, ": ")?;

            Ok(())
        })?;

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
