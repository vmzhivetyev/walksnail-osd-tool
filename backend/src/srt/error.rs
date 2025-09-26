use parse_display::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SrtFileError {
    #[error("Srt file seem to be empty.")]
    EmptySrtFile {},

    #[error("Failed to parse data from STR file, source: {source}")]
    FailedToParseData {
        #[from]
        source: ParseError,
    },

    #[error("Unable to open SRT file, source: {source}")]
    UnableToOpenFile {
        #[from]
        source: srtparse::ReaderError,
    },

    #[error("Parse error: {message}")]
    ParseError { message: String },
}
